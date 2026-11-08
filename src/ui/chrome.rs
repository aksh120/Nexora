use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    AlphaBlend, BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, CreateDIBSection, CreateFontW,
    CreatePen, CreateSolidBrush, DeleteDC, DeleteObject, DrawTextW, FillRect, GetDC, LineTo, MoveToEx,
    ReleaseDC, SelectObject, SetBkMode, SetTextColor, AC_SRC_ALPHA, AC_SRC_OVER, BITMAPINFO,
    BITMAPINFOHEADER, BI_RGB, BLENDFUNCTION, DIB_RGB_COLORS, DT_END_ELLIPSIS, DT_LEFT,
    DT_SINGLELINE, DT_VCENTER, FW_NORMAL, HBITMAP, HDC, HFONT, PS_SOLID, SRCCOPY, TRANSPARENT,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::{GetKeyState, VK_CONTROL, VK_RETURN};
use windows::Win32::UI::WindowsAndMessaging::{
    CallWindowProcW, CreateWindowExW, GetWindowLongPtrW, GetWindowTextLengthW, GetWindowTextW,
    MoveWindow, SendMessageW, SetWindowLongPtrW, SetWindowTextW, GWLP_USERDATA, GWLP_WNDPROC, HMENU,
    WINDOW_EX_STYLE, WINDOW_STYLE, WM_CHAR, WM_KEYDOWN, WM_KILLFOCUS, WM_LBUTTONUP, WM_SETFOCUS,
    WM_SETFONT, WNDPROC, WS_CHILD, WS_TABSTOP, WS_VISIBLE,
};

use crate::core::error::{NexoraError, Result};
use crate::tabs::Tab;
use crate::ui::layout::{BrowserLayout, OMNIBOX_HEIGHT, TAB_BAR_HEIGHT, TOP_CHROME_HEIGHT};
use crate::ui::theme::{rgb, ThemePalette};

pub const IDC_OMNIBOX: usize = 1005;
pub const EM_SETSEL: u32 = 0x00B1;

pub const ATLAS_ICON_SIZE: i32 = 24;
pub const ATLAS_TOTAL_WIDTH: i32 = 696;
pub const ATLAS_TOTAL_HEIGHT: i32 = 24;
const ATLAS_BYTES: &[u8] = include_bytes!("../../assets/branding/chrome_icons_atlas.bin");

pub const ICON_LOGO: usize = 0;
pub const ICON_BACK_NORMAL: usize = 1;
pub const ICON_BACK_HOVER: usize = 2;
pub const ICON_FWD_NORMAL: usize = 3;
pub const ICON_FWD_HOVER: usize = 4;
pub const ICON_RELOAD_NORMAL: usize = 5;
pub const ICON_RELOAD_HOVER: usize = 6;
pub const ICON_HOME_NORMAL: usize = 7;
pub const ICON_HOME_HOVER: usize = 8;
pub const ICON_SHIELD_CHECK: usize = 9;
pub const ICON_BM_OUTLINE_NORMAL: usize = 10;
pub const ICON_BM_OUTLINE_HOVER: usize = 11;
pub const ICON_BM_FILLED_NORMAL: usize = 12;
pub const ICON_BM_FILLED_HOVER: usize = 13;
pub const ICON_MENU_NORMAL: usize = 14;
pub const ICON_MENU_HOVER: usize = 15;
pub const ICON_PLUS_NORMAL: usize = 16;
pub const ICON_PLUS_HOVER: usize = 17;
pub const ICON_CLOSE_NORMAL: usize = 18;
pub const ICON_CLOSE_ACTIVE: usize = 19;
pub const ICON_CLOSE_HOVER: usize = 20;
pub const ICON_CAP_MIN_NORMAL: usize = 21;
pub const ICON_CAP_MIN_HOVER: usize = 22;
pub const ICON_CAP_MAX_NORMAL: usize = 23;
pub const ICON_CAP_MAX_HOVER: usize = 24;
pub const ICON_CAP_RESTORE_NORMAL: usize = 25;
pub const ICON_CAP_RESTORE_HOVER: usize = 26;
pub const ICON_CAP_CLOSE_NORMAL: usize = 27;
pub const ICON_CAP_CLOSE_HOVER: usize = 28;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChromeHitResult {
    BrandLogo,
    Tab(usize),
    TabClose(usize),
    NewTab,
    BtnBack,
    BtnForward,
    BtnReload,
    BtnHome,
    BtnBookmark,
    BtnMenu,
    CaptionMinimize,
    CaptionMaximize,
    CaptionClose,
    TitleBarDrag,
    None,
}

pub struct BrowserChrome {
    pub hwnd_parent: HWND,
    pub hwnd_omnibox: HWND,
    pub hovered_item: ChromeHitResult,
    font: HFONT,
    pub theme: ThemePalette,
    pub is_current_page_bookmarked: bool,
    atlas_dc: HDC,
    atlas_bmp: HBITMAP,
}

struct OmniboxSubclassContext {
    orig_proc: WNDPROC,
    on_enter: Box<dyn Fn(String) + 'static>,
    just_got_focus: bool,
}

impl BrowserChrome {
    pub fn new<FNavigate>(
        parent: HWND,
        theme: ThemePalette,
        on_navigate: FNavigate,
    ) -> Result<Self>
    where
        FNavigate: Fn(String) + 'static,
    {
        unsafe {
            let hinstance: HINSTANCE = GetModuleHandleW(None)
                .map_err(|e| NexoraError::Win32(format!("Failed to get module handle: {}", e)))?
                .into();

            let font = CreateFontW(
                -13, // ~10pt Segoe UI
                0,
                0,
                0,
                FW_NORMAL.0 as i32,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                w!("Segoe UI"),
            );

            let edit_style = WINDOW_STYLE((WS_CHILD.0 | WS_VISIBLE.0 | WS_TABSTOP.0 | 0x0080) as u32); // ES_AUTOHSCROLL = 0x0080

            // Omnibox URL / search input field
            let hwnd_omnibox = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                w!("EDIT"),
                w!(""),
                edit_style,
                144, 40, 600, OMNIBOX_HEIGHT,
                parent,
                HMENU(IDC_OMNIBOX as *mut core::ffi::c_void),
                hinstance,
                None,
            ).map_err(|e| NexoraError::Win32(format!("Failed to create Omnibox: {}", e)))?;

            let _ = SendMessageW(
                hwnd_omnibox,
                WM_SETFONT,
                WPARAM(font.0 as usize),
                LPARAM(1),
            );

            // Subclass the Omnibox to intercept Enter key, Ctrl+A, and click selection
            let context = Box::new(OmniboxSubclassContext {
                orig_proc: std::mem::transmute(GetWindowLongPtrW(hwnd_omnibox, GWLP_WNDPROC)),
                on_enter: Box::new(on_navigate),
                just_got_focus: false,
            });
            SetWindowLongPtrW(hwnd_omnibox, GWLP_USERDATA, Box::into_raw(context) as isize);
            SetWindowLongPtrW(hwnd_omnibox, GWLP_WNDPROC, Self::omnibox_subclass_proc as *const () as isize);

            // Initialize Hardware 32-bit Vector Icon Atlas
            let screen_dc = GetDC(None);
            let atlas_dc = CreateCompatibleDC(screen_dc);
            let bmi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: ATLAS_TOTAL_WIDTH,
                    biHeight: -ATLAS_TOTAL_HEIGHT,
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB.0,
                    biSizeImage: (ATLAS_TOTAL_WIDTH * ATLAS_TOTAL_HEIGHT * 4) as u32,
                    ..Default::default()
                },
                ..Default::default()
            };
            let mut bits: *mut core::ffi::c_void = std::ptr::null_mut();
            let atlas_bmp = CreateDIBSection(
                screen_dc,
                &bmi,
                DIB_RGB_COLORS,
                &mut bits,
                None,
                0,
            ).map_err(|e| NexoraError::Win32(format!("Failed to create DIB section for icon atlas: {}", e)))?;

            if !bits.is_null() && !atlas_bmp.is_invalid() {
                std::ptr::copy_nonoverlapping(ATLAS_BYTES.as_ptr(), bits as *mut u8, ATLAS_BYTES.len());
            }
            let _ = SelectObject(atlas_dc, atlas_bmp);
            let _ = ReleaseDC(None, screen_dc);

            Ok(Self {
                hwnd_parent: parent,
                hwnd_omnibox,
                hovered_item: ChromeHitResult::None,
                font,
                theme,
                is_current_page_bookmarked: false,
                atlas_dc,
                atlas_bmp,
            })
        }
    }

    pub fn set_is_bookmarked(&mut self, bookmarked: bool) {
        self.is_current_page_bookmarked = bookmarked;
    }

    pub fn update_hover(&mut self, new_hover: ChromeHitResult) -> bool {
        if self.hovered_item != new_hover {
            self.hovered_item = new_hover;
            true
        } else {
            false
        }
    }

    pub fn set_omnibox_text(&self, text: &str) {
        unsafe {
            let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
            let _ = SetWindowTextW(self.hwnd_omnibox, PCWSTR(wide.as_ptr()));
        }
    }

    pub fn get_omnibox_text(&self) -> String {
        unsafe {
            let len = GetWindowTextLengthW(self.hwnd_omnibox);
            if len <= 0 {
                return String::new();
            }
            let mut buf: Vec<u16> = vec![0; (len + 1) as usize];
            let read = GetWindowTextW(self.hwnd_omnibox, &mut buf);
            if read > 0 {
                String::from_utf16_lossy(&buf[..read as usize])
            } else {
                String::new()
            }
        }
    }

    pub fn layout_controls(&self, layout: &BrowserLayout, _tab_count: usize) {
        unsafe {
            let omnibox = layout.omnibox_rect();
            // Inset edit control inside the rendered omnibox container frame, leaving 28px on left for ShieldCheck
            let _ = MoveWindow(
                self.hwnd_omnibox,
                omnibox.0 + 28,
                omnibox.1 + 4,
                omnibox.2 - 36,
                omnibox.3 - 8,
                true,
            );
        }
    }

    pub fn hit_test(
        &self,
        x: i32,
        y: i32,
        layout: &BrowserLayout,
        tabs: &[Tab],
    ) -> ChromeHitResult {
        // 1. Top Title Bar & Tab Strip
        if y >= 0 && y <= TAB_BAR_HEIGHT {
            // Brand Logo
            let logo = layout.brand_logo_rect();
            if x >= logo.0 && x < logo.0 + logo.2 && y >= logo.1 && y < logo.1 + logo.3 {
                return ChromeHitResult::BrandLogo;
            }

            // Caption Buttons on Top-Right
            let close_rect = layout.caption_close_rect();
            if x >= close_rect.0 && x < close_rect.0 + close_rect.2 {
                return ChromeHitResult::CaptionClose;
            }
            let max_rect = layout.caption_maximize_rect();
            if x >= max_rect.0 && x < max_rect.0 + max_rect.2 {
                return ChromeHitResult::CaptionMaximize;
            }
            let min_rect = layout.caption_minimize_rect();
            if x >= min_rect.0 && x < min_rect.0 + min_rect.2 {
                return ChromeHitResult::CaptionMinimize;
            }

            // Tabs
            let rects = layout.calculate_tab_rects(tabs.len());
            for (i, (tx, ty, tw, th)) in rects.iter().enumerate() {
                if x >= *tx && x < tx + tw && y >= *ty && y < ty + th {
                    let tab_id = tabs[i].id;
                    // Close button is in rightmost 24px of tab
                    if x >= (tx + tw - 24) {
                        return ChromeHitResult::TabClose(tab_id);
                    }
                    return ChromeHitResult::Tab(tab_id);
                }
            }

            // New Tab button hit
            let new_tab_rect = layout.new_tab_button_rect(tabs.len());
            if x >= new_tab_rect.0 && x < new_tab_rect.0 + new_tab_rect.2
                && y >= new_tab_rect.1 && y < new_tab_rect.1 + new_tab_rect.3
            {
                return ChromeHitResult::NewTab;
            }

            // Empty space in tab bar / title bar -> draggable area
            if x < min_rect.0 {
                return ChromeHitResult::TitleBarDrag;
            }
        }

        // 2. Navigation Toolbar
        if y > TAB_BAR_HEIGHT && y <= TOP_CHROME_HEIGHT {
            let back = layout.back_button_rect();
            if x >= back.0 && x < back.0 + back.2 && y >= back.1 && y < back.1 + back.3 {
                return ChromeHitResult::BtnBack;
            }

            let fwd = layout.forward_button_rect();
            if x >= fwd.0 && x < fwd.0 + fwd.2 && y >= fwd.1 && y < fwd.1 + fwd.3 {
                return ChromeHitResult::BtnForward;
            }

            let reload = layout.reload_button_rect();
            if x >= reload.0 && x < reload.0 + reload.2 && y >= reload.1 && y < reload.1 + reload.3 {
                return ChromeHitResult::BtnReload;
            }

            let home = layout.home_button_rect();
            if x >= home.0 && x < home.0 + home.2 && y >= home.1 && y < home.1 + home.3 {
                return ChromeHitResult::BtnHome;
            }

            let bm = layout.bookmark_button_rect();
            if x >= bm.0 && x < bm.0 + bm.2 && y >= bm.1 && y < bm.1 + bm.3 {
                return ChromeHitResult::BtnBookmark;
            }

            let menu = layout.menu_button_rect();
            if x >= menu.0 && x < menu.0 + menu.2 && y >= menu.1 && y < menu.1 + menu.3 {
                return ChromeHitResult::BtnMenu;
            }
        }

        ChromeHitResult::None
    }

    pub fn render(
        &self,
        hdc: HDC,
        layout: &BrowserLayout,
        tabs: &[Tab],
        active_tab_id: Option<usize>,
        is_maximized: bool,
    ) {
        unsafe {
            let mem_dc = CreateCompatibleDC(hdc);
            let mem_bmp = CreateCompatibleBitmap(hdc, layout.window_width, TOP_CHROME_HEIGHT);
            let old_bmp = SelectObject(mem_dc, mem_bmp);
            let old_font = SelectObject(mem_dc, self.font);

            // 1. Draw Tab Bar Background
            let tab_bar_brush = CreateSolidBrush(self.theme.tab_bar_bg);
            let tab_bar_rect = RECT {
                left: 0,
                top: 0,
                right: layout.window_width,
                bottom: TAB_BAR_HEIGHT,
            };
            FillRect(mem_dc, &tab_bar_rect, tab_bar_brush);
            let _ = DeleteObject(tab_bar_brush);

            // 2. Draw Toolbar Background
            let toolbar_brush = CreateSolidBrush(self.theme.toolbar_bg);
            let toolbar_rect = RECT {
                left: 0,
                top: TAB_BAR_HEIGHT,
                right: layout.window_width,
                bottom: TOP_CHROME_HEIGHT,
            };
            FillRect(mem_dc, &toolbar_rect, toolbar_brush);
            let _ = DeleteObject(toolbar_brush);

            // 3. Draw Divider Line below toolbar
            let div_pen = CreatePen(PS_SOLID, 1, self.theme.divider_color);
            let old_pen = SelectObject(mem_dc, div_pen);
            let _ = MoveToEx(mem_dc, 0, TOP_CHROME_HEIGHT - 1, None);
            let _ = LineTo(mem_dc, layout.window_width, TOP_CHROME_HEIGHT - 1);

            // 4. Render Brand Logo (Official Orbital Ribbon N from Master Vector Atlas)
            let logo = layout.brand_logo_rect();
            Self::draw_atlas_icon(mem_dc, self.atlas_dc, ICON_LOGO, logo.0, logo.1, logo.2, logo.3);

            // 5. Render Tabs
            let tab_rects = layout.calculate_tab_rects(tabs.len());
            SetBkMode(mem_dc, TRANSPARENT);

            for (i, tab) in tabs.iter().enumerate() {
                if i >= tab_rects.len() {
                    break;
                }
                let (tx, ty, tw, th) = tab_rects[i];
                let is_active = Some(tab.id) == active_tab_id;
                let is_hovered = self.hovered_item == ChromeHitResult::Tab(tab.id);

                let tab_bg = if is_active {
                    self.theme.tab_active_bg
                } else if is_hovered {
                    self.theme.tab_hover_bg
                } else {
                    self.theme.tab_inactive_bg
                };

                let brush = CreateSolidBrush(tab_bg);
                let r = RECT {
                    left: tx,
                    top: ty,
                    right: tx + tw,
                    bottom: ty + th,
                };
                FillRect(mem_dc, &r, brush);
                let _ = DeleteObject(brush);

                // Active tab top accent line
                if is_active {
                    let accent_pen = CreatePen(PS_SOLID, 2, self.theme.accent_color);
                    let _ = SelectObject(mem_dc, accent_pen);
                    let _ = MoveToEx(mem_dc, tx, ty, None);
                    let _ = LineTo(mem_dc, tx + tw, ty);
                    let _ = SelectObject(mem_dc, div_pen);
                    let _ = DeleteObject(accent_pen);
                }

                // Tab Title Text
                let text_color = if is_active {
                    self.theme.tab_text_active
                } else {
                    self.theme.tab_text_inactive
                };
                SetTextColor(mem_dc, text_color);

                let mut title = tab.title.clone();
                if tab.is_playing_audio {
                    title = format!("\u{266A} {}", title);
                } else if tab.is_suspended() {
                    title = format!("[Zz] {}", title);
                }

                let mut wide_title: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
                let mut text_rect = RECT {
                    left: tx + 10,
                    top: ty,
                    right: tx + tw - 24, // Space for close button
                    bottom: ty + th,
                };
                let title_slice_len = wide_title.len() - 1;
                DrawTextW(
                    mem_dc,
                    &mut wide_title[..title_slice_len],
                    &mut text_rect,
                    DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS,
                );

                // Tab Close Button (Lucide X with Smooth Subpixel Antialiasing)
                let is_close_hovered = self.hovered_item == ChromeHitResult::TabClose(tab.id);
                let close_cx = tx + tw - 13;
                let close_cy = ty + th / 2;
                if is_close_hovered {
                    let hover_brush = CreateSolidBrush(self.theme.button_hover_bg);
                    let r = RECT {
                        left: close_cx - 8,
                        top: close_cy - 8,
                        right: close_cx + 8,
                        bottom: close_cy + 8,
                    };
                    FillRect(mem_dc, &r, hover_brush);
                    let _ = DeleteObject(hover_brush);
                }
                let close_idx = if is_close_hovered {
                    ICON_CLOSE_HOVER
                } else if is_active {
                    ICON_CLOSE_ACTIVE
                } else {
                    ICON_CLOSE_NORMAL
                };
                Self::draw_atlas_icon(mem_dc, self.atlas_dc, close_idx, close_cx - 7, close_cy - 7, 14, 14);
            }

            // 6. Render New Tab "+" Button (Lucide Plus)
            let new_tab = layout.new_tab_button_rect(tabs.len());
            let is_new_tab_hovered = self.hovered_item == ChromeHitResult::NewTab;
            if is_new_tab_hovered {
                let hover_brush = CreateSolidBrush(self.theme.button_hover_bg);
                let r = RECT {
                    left: new_tab.0,
                    top: new_tab.1,
                    right: new_tab.0 + new_tab.2,
                    bottom: new_tab.1 + new_tab.3,
                };
                FillRect(mem_dc, &r, hover_brush);
                let _ = DeleteObject(hover_brush);
            }
            let plus_idx = if is_new_tab_hovered { ICON_PLUS_HOVER } else { ICON_PLUS_NORMAL };
            let plus_x = new_tab.0 + (new_tab.2 - 16) / 2;
            let plus_y = new_tab.1 + (new_tab.3 - 16) / 2;
            Self::draw_atlas_icon(mem_dc, self.atlas_dc, plus_idx, plus_x, plus_y, 16, 16);

            // 7. Render Custom Caption Buttons on Top-Right
            let min_rect = layout.caption_minimize_rect();
            let max_rect = layout.caption_maximize_rect();
            let close_rect = layout.caption_close_rect();

            // Minimize
            let is_min_hover = self.hovered_item == ChromeHitResult::CaptionMinimize;
            if is_min_hover {
                let h_brush = CreateSolidBrush(self.theme.button_hover_bg);
                let r = RECT { left: min_rect.0, top: min_rect.1, right: min_rect.0 + min_rect.2, bottom: min_rect.1 + min_rect.3 };
                FillRect(mem_dc, &r, h_brush);
                let _ = DeleteObject(h_brush);
            }
            let min_idx = if is_min_hover { ICON_CAP_MIN_HOVER } else { ICON_CAP_MIN_NORMAL };
            Self::draw_atlas_icon(mem_dc, self.atlas_dc, min_idx, min_rect.0 + (min_rect.2 - 14) / 2, min_rect.1 + (min_rect.3 - 14) / 2, 14, 14);

            // Maximize / Restore
            let is_max_hover = self.hovered_item == ChromeHitResult::CaptionMaximize;
            if is_max_hover {
                let h_brush = CreateSolidBrush(self.theme.button_hover_bg);
                let r = RECT { left: max_rect.0, top: max_rect.1, right: max_rect.0 + max_rect.2, bottom: max_rect.1 + max_rect.3 };
                FillRect(mem_dc, &r, h_brush);
                let _ = DeleteObject(h_brush);
            }
            let max_idx = if is_maximized {
                if is_max_hover { ICON_CAP_RESTORE_HOVER } else { ICON_CAP_RESTORE_NORMAL }
            } else {
                if is_max_hover { ICON_CAP_MAX_HOVER } else { ICON_CAP_MAX_NORMAL }
            };
            Self::draw_atlas_icon(mem_dc, self.atlas_dc, max_idx, max_rect.0 + (max_rect.2 - 14) / 2, max_rect.1 + (max_rect.3 - 14) / 2, 14, 14);

            // Close
            let is_close_hover = self.hovered_item == ChromeHitResult::CaptionClose;
            if is_close_hover {
                let red_brush = CreateSolidBrush(rgb(225, 29, 72)); // Eclipse Crimson red
                let r = RECT { left: close_rect.0, top: close_rect.1, right: close_rect.0 + close_rect.2, bottom: close_rect.1 + close_rect.3 };
                FillRect(mem_dc, &r, red_brush);
                let _ = DeleteObject(red_brush);
            }
            let cap_close_idx = if is_close_hover { ICON_CAP_CLOSE_HOVER } else { ICON_CAP_CLOSE_NORMAL };
            Self::draw_atlas_icon(mem_dc, self.atlas_dc, cap_close_idx, close_rect.0 + (close_rect.2 - 14) / 2, close_rect.1 + (close_rect.3 - 14) / 2, 14, 14);

            // 8. Render Navigation Toolbar Buttons with Antialiased Vector Lucide Icons
            // Back Button
            let back = layout.back_button_rect();
            let is_back_hover = self.hovered_item == ChromeHitResult::BtnBack;
            Self::render_toolbar_button(mem_dc, self.atlas_dc, back, is_back_hover, self.theme.button_hover_bg, if is_back_hover { ICON_BACK_HOVER } else { ICON_BACK_NORMAL });

            // Forward Button
            let fwd = layout.forward_button_rect();
            let is_fwd_hover = self.hovered_item == ChromeHitResult::BtnForward;
            Self::render_toolbar_button(mem_dc, self.atlas_dc, fwd, is_fwd_hover, self.theme.button_hover_bg, if is_fwd_hover { ICON_FWD_HOVER } else { ICON_FWD_NORMAL });

            // Reload Button
            let reload = layout.reload_button_rect();
            let is_reload_hover = self.hovered_item == ChromeHitResult::BtnReload;
            Self::render_toolbar_button(mem_dc, self.atlas_dc, reload, is_reload_hover, self.theme.button_hover_bg, if is_reload_hover { ICON_RELOAD_HOVER } else { ICON_RELOAD_NORMAL });

            // Home Button
            let home = layout.home_button_rect();
            let is_home_hover = self.hovered_item == ChromeHitResult::BtnHome;
            Self::render_toolbar_button(mem_dc, self.atlas_dc, home, is_home_hover, self.theme.button_hover_bg, if is_home_hover { ICON_HOME_HOVER } else { ICON_HOME_NORMAL });

            // Bookmark Button (Dynamic Fill Status)
            let bm = layout.bookmark_button_rect();
            let is_bm_hover = self.hovered_item == ChromeHitResult::BtnBookmark;
            let bm_idx = if self.is_current_page_bookmarked {
                if is_bm_hover { ICON_BM_FILLED_HOVER } else { ICON_BM_FILLED_NORMAL }
            } else {
                if is_bm_hover { ICON_BM_OUTLINE_HOVER } else { ICON_BM_OUTLINE_NORMAL }
            };
            Self::render_toolbar_button(mem_dc, self.atlas_dc, bm, is_bm_hover, self.theme.button_hover_bg, bm_idx);

            // Menu Button
            let menu = layout.menu_button_rect();
            let is_menu_hover = self.hovered_item == ChromeHitResult::BtnMenu;
            Self::render_toolbar_button(mem_dc, self.atlas_dc, menu, is_menu_hover, self.theme.button_hover_bg, if is_menu_hover { ICON_MENU_HOVER } else { ICON_MENU_NORMAL });

            // 9. Render Omnibox Container Frame
            let omnibox = layout.omnibox_rect();
            let omni_bg_brush = CreateSolidBrush(self.theme.omnibox_bg);
            let omni_rect = RECT {
                left: omnibox.0,
                top: omnibox.1,
                right: omnibox.0 + omnibox.2,
                bottom: omnibox.1 + omnibox.3,
            };
            FillRect(mem_dc, &omni_rect, omni_bg_brush);
            let _ = DeleteObject(omni_bg_brush);

            let border_pen = CreatePen(PS_SOLID, 1, self.theme.omnibox_border);
            let _ = SelectObject(mem_dc, border_pen);
            let _ = MoveToEx(mem_dc, omnibox.0, omnibox.1, None);
            let _ = LineTo(mem_dc, omnibox.0 + omnibox.2, omnibox.1);
            let _ = LineTo(mem_dc, omnibox.0 + omnibox.2, omnibox.1 + omnibox.3);
            let _ = LineTo(mem_dc, omnibox.0, omnibox.1 + omnibox.3);
            let _ = LineTo(mem_dc, omnibox.0, omnibox.1);
            let _ = SelectObject(mem_dc, div_pen);
            let _ = DeleteObject(border_pen);

            // Render Lucide ShieldCheck security icon on the left of omnibox
            Self::draw_atlas_icon(mem_dc, self.atlas_dc, ICON_SHIELD_CHECK, omnibox.0 + 8, omnibox.1 + (omnibox.3 - 16) / 2, 16, 16);

            // Clean up GDI objects
            let _ = SelectObject(mem_dc, old_pen);
            let _ = DeleteObject(div_pen);

            // BitBlt double buffer to screen HDC
            let _ = BitBlt(
                hdc,
                0,
                0,
                layout.window_width,
                TOP_CHROME_HEIGHT,
                mem_dc,
                0,
                0,
                SRCCOPY,
            );

            let _ = SelectObject(mem_dc, old_bmp);
            let _ = SelectObject(mem_dc, old_font);
            let _ = DeleteObject(mem_bmp);
            let _ = DeleteDC(mem_dc);
        }
    }

    // --- Vector Icon Atlas Drawing Helpers ---

    unsafe fn draw_atlas_icon(
        mem_dc: HDC,
        atlas_dc: HDC,
        icon_idx: usize,
        dest_x: i32,
        dest_y: i32,
        dest_w: i32,
        dest_h: i32,
    ) {
        let src_x = (icon_idx as i32) * ATLAS_ICON_SIZE;
        let blend = BLENDFUNCTION {
            BlendOp: AC_SRC_OVER as u8,
            BlendFlags: 0,
            SourceConstantAlpha: 255,
            AlphaFormat: AC_SRC_ALPHA as u8,
        };
        let _ = AlphaBlend(
            mem_dc,
            dest_x,
            dest_y,
            dest_w,
            dest_h,
            atlas_dc,
            src_x,
            0,
            ATLAS_ICON_SIZE,
            ATLAS_ICON_SIZE,
            blend,
        );
    }

    unsafe fn render_toolbar_button(
        mem_dc: HDC,
        atlas_dc: HDC,
        rect: (i32, i32, i32, i32),
        is_hovered: bool,
        hover_bg: windows::Win32::Foundation::COLORREF,
        icon_idx: usize,
    ) {
        if is_hovered {
            let h_brush = CreateSolidBrush(hover_bg);
            let r = RECT {
                left: rect.0 + 2,
                top: rect.1 + 2,
                right: rect.0 + rect.2 - 2,
                bottom: rect.1 + rect.3 - 2,
            };
            FillRect(mem_dc, &r, h_brush);
            let _ = DeleteObject(h_brush);
        }

        let icon_w = 20;
        let icon_h = 20;
        let icon_x = rect.0 + (rect.2 - icon_w) / 2;
        let icon_y = rect.1 + (rect.3 - icon_h) / 2;
        Self::draw_atlas_icon(mem_dc, atlas_dc, icon_idx, icon_x, icon_y, icon_w, icon_h);
    }

    unsafe extern "system" fn omnibox_subclass_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut OmniboxSubclassContext;
        if ptr.is_null() {
            return LRESULT(0);
        }

        match msg {
            WM_SETFOCUS => {
                (*ptr).just_got_focus = true;
            }
            WM_KILLFOCUS => {
                (*ptr).just_got_focus = false;
            }
            WM_LBUTTONUP => {
                let res = CallWindowProcW((*ptr).orig_proc, hwnd, msg, wparam, lparam);
                if (*ptr).just_got_focus {
                    (*ptr).just_got_focus = false;
                    let _ = SendMessageW(hwnd, EM_SETSEL, WPARAM(0), LPARAM(-1));
                }
                return res;
            }
            WM_KEYDOWN => {
                if wparam.0 == VK_RETURN.0 as usize {
                    let len = GetWindowTextLengthW(hwnd);
                    if len > 0 {
                        let mut buf: Vec<u16> = vec![0; (len + 1) as usize];
                        let read = GetWindowTextW(hwnd, &mut buf);
                        if read > 0 {
                            let text = String::from_utf16_lossy(&buf[..read as usize]);
                            ((*ptr).on_enter)(text);
                        }
                    }
                    return LRESULT(0);
                }

                // Handle Ctrl + A (Select All)
                if wparam.0 == 0x41 || wparam.0 == b'a' as usize || wparam.0 == b'A' as usize {
                    let ctrl_pressed = (GetKeyState(VK_CONTROL.0 as i32) as u16 & 0x8000) != 0;
                    if ctrl_pressed {
                        let _ = SendMessageW(hwnd, EM_SETSEL, WPARAM(0), LPARAM(-1));
                        return LRESULT(0);
                    }
                }
            }
            WM_CHAR => {
                if wparam.0 == VK_RETURN.0 as usize {
                    // Suppress ding sound on Enter
                    return LRESULT(0);
                }
                if wparam.0 == 1 {
                    // Suppress ding/beep sound on Ctrl + A
                    return LRESULT(0);
                }
            }
            _ => {}
        }

        CallWindowProcW((*ptr).orig_proc, hwnd, msg, wparam, lparam)
    }
}

impl Drop for BrowserChrome {
    fn drop(&mut self) {
        unsafe {
            let ptr = GetWindowLongPtrW(self.hwnd_omnibox, GWLP_USERDATA) as *mut OmniboxSubclassContext;
            if !ptr.is_null() {
                let _ = Box::from_raw(ptr);
                SetWindowLongPtrW(self.hwnd_omnibox, GWLP_USERDATA, 0);
            }
            let _ = DeleteDC(self.atlas_dc);
            let _ = DeleteObject(self.atlas_bmp);
            let _ = DeleteObject(self.font);
        }
    }
}
