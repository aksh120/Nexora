use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    Arc, BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, CreateFontW, CreatePen, CreateSolidBrush,
    DeleteDC, DeleteObject, DrawTextW, Ellipse, FillRect, LineTo, MoveToEx, Polygon, SelectObject,
    SetBkMode, SetTextColor, DT_CENTER, DT_END_ELLIPSIS, DT_LEFT, DT_SINGLELINE, DT_VCENTER,
    FW_NORMAL, HDC, HFONT, PS_NULL, PS_SOLID, SRCCOPY, TRANSPARENT,
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

            Ok(Self {
                hwnd_parent: parent,
                hwnd_omnibox,
                hovered_item: ChromeHitResult::None,
                font,
                theme,
                is_current_page_bookmarked: false,
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

            // 4. Render Brand Logo (Official Orbital Ribbon N)
            let logo = layout.brand_logo_rect();
            let is_logo_hovered = self.hovered_item == ChromeHitResult::BrandLogo;
            let logo_cx = logo.0 + logo.2 / 2;
            let logo_cy = logo.1 + logo.3 / 2;
            Self::draw_brand_logo(mem_dc, logo_cx, logo_cy, is_logo_hovered);

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

                // Close Button (Lucide X)
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
                let close_color = if is_close_hovered {
                    self.theme.accent_color
                } else if is_active {
                    rgb(180, 180, 180)
                } else {
                    rgb(120, 120, 120)
                };
                Self::draw_vector_x(mem_dc, close_cx, close_cy, close_color);
            }

            // 5. Render New Tab "+" Button
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
            let plus_cx = new_tab.0 + new_tab.2 / 2;
            let plus_cy = new_tab.1 + new_tab.3 / 2;
            let plus_color = if is_new_tab_hovered { self.theme.accent_color } else { rgb(180, 165, 170) };
            Self::draw_vector_plus(mem_dc, plus_cx, plus_cy, plus_color);

            // 6. Render Custom Caption Buttons on Top-Right
            let min_rect = layout.caption_minimize_rect();
            let max_rect = layout.caption_maximize_rect();
            let close_rect = layout.caption_close_rect();

            // Minimize
            if self.hovered_item == ChromeHitResult::CaptionMinimize {
                let h_brush = CreateSolidBrush(self.theme.button_hover_bg);
                let r = RECT { left: min_rect.0, top: min_rect.1, right: min_rect.0 + min_rect.2, bottom: min_rect.1 + min_rect.3 };
                FillRect(mem_dc, &r, h_brush);
                let _ = DeleteObject(h_brush);
            }
            Self::draw_caption_minimize(mem_dc, min_rect.0 + min_rect.2 / 2, min_rect.1 + min_rect.3 / 2, rgb(190, 175, 180));

            // Maximize
            if self.hovered_item == ChromeHitResult::CaptionMaximize {
                let h_brush = CreateSolidBrush(self.theme.button_hover_bg);
                let r = RECT { left: max_rect.0, top: max_rect.1, right: max_rect.0 + max_rect.2, bottom: max_rect.1 + max_rect.3 };
                FillRect(mem_dc, &r, h_brush);
                let _ = DeleteObject(h_brush);
            }
            Self::draw_caption_maximize(mem_dc, max_rect.0 + max_rect.2 / 2, max_rect.1 + max_rect.3 / 2, is_maximized, rgb(190, 175, 180));

            // Close
            let is_close_hover = self.hovered_item == ChromeHitResult::CaptionClose;
            if is_close_hover {
                let red_brush = CreateSolidBrush(rgb(225, 29, 72)); // Eclipse Crimson red
                let r = RECT { left: close_rect.0, top: close_rect.1, right: close_rect.0 + close_rect.2, bottom: close_rect.1 + close_rect.3 };
                FillRect(mem_dc, &r, red_brush);
                let _ = DeleteObject(red_brush);
            }
            let close_icon_color = if is_close_hover { rgb(255, 255, 255) } else { rgb(190, 175, 180) };
            Self::draw_caption_close(mem_dc, close_rect.0 + close_rect.2 / 2, close_rect.1 + close_rect.3 / 2, close_icon_color);

            // 7. Render Navigation Toolbar Buttons with Clean Vector Icons
            // Back Button
            let back = layout.back_button_rect();
            Self::render_nav_button(mem_dc, back, self.hovered_item == ChromeHitResult::BtnBack, self.theme.button_hover_bg, |dc, cx, cy, col| {
                Self::draw_vector_back(dc, cx, cy, col);
            });

            // Forward Button
            let fwd = layout.forward_button_rect();
            Self::render_nav_button(mem_dc, fwd, self.hovered_item == ChromeHitResult::BtnForward, self.theme.button_hover_bg, |dc, cx, cy, col| {
                Self::draw_vector_forward(dc, cx, cy, col);
            });

            // Reload Button
            let reload = layout.reload_button_rect();
            Self::render_nav_button(mem_dc, reload, self.hovered_item == ChromeHitResult::BtnReload, self.theme.button_hover_bg, |dc, cx, cy, col| {
                Self::draw_vector_reload(dc, cx, cy, col);
            });

            // Home Button
            let home = layout.home_button_rect();
            Self::render_nav_button(mem_dc, home, self.hovered_item == ChromeHitResult::BtnHome, self.theme.button_hover_bg, |dc, cx, cy, col| {
                Self::draw_vector_home(dc, cx, cy, col);
            });

            // Bookmark Button (Lucide Bookmark Ribbon)
            let bm = layout.bookmark_button_rect();
            let is_bm_hovered = self.hovered_item == ChromeHitResult::BtnBookmark;
            let is_bm_filled = self.is_current_page_bookmarked;
            Self::render_nav_button(mem_dc, bm, is_bm_hovered, self.theme.button_hover_bg, |dc, cx, cy, col| {
                let bm_color = if is_bm_hovered {
                    rgb(244, 63, 94)
                } else if is_bm_filled {
                    rgb(225, 29, 72)
                } else {
                    col
                };
                Self::draw_vector_bookmark(dc, cx, cy, bm_color, is_bm_filled);
            });

            // Menu Button
            let menu = layout.menu_button_rect();
            Self::render_nav_button(mem_dc, menu, self.hovered_item == ChromeHitResult::BtnMenu, self.theme.button_hover_bg, |dc, cx, cy, col| {
                Self::draw_vector_menu(dc, cx, cy, col);
            });

            // 8. Render Omnibox Container Frame
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
            Self::draw_vector_shield_check(
                mem_dc,
                omnibox.0 + 14,
                omnibox.1 + omnibox.3 / 2,
                rgb(16, 185, 129), // Restrained emerald green
            );

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

    // --- Vector Drawing Helpers ---

    unsafe fn render_nav_button<F>(hdc: HDC, rect: (i32, i32, i32, i32), is_hovered: bool, hover_bg: windows::Win32::Foundation::COLORREF, draw_icon: F)
    where
        F: FnOnce(HDC, i32, i32, windows::Win32::Foundation::COLORREF),
    {
        if is_hovered {
            let h_brush = CreateSolidBrush(hover_bg);
            let r = RECT { left: rect.0, top: rect.1, right: rect.0 + rect.2, bottom: rect.1 + rect.3 };
            FillRect(hdc, &r, h_brush);
            let _ = DeleteObject(h_brush);
        }

        let cx = rect.0 + rect.2 / 2;
        let cy = rect.1 + rect.3 / 2;
        let icon_color = if is_hovered { rgb(225, 29, 72) } else { rgb(190, 175, 180) };
        draw_icon(hdc, cx, cy, icon_color);
    }

    unsafe fn draw_vector_back(hdc: HDC, cx: i32, cy: i32, color: windows::Win32::Foundation::COLORREF) {
        let pen = CreatePen(PS_SOLID, 2, color);
        let old_pen = SelectObject(hdc, pen);

        // Shadcn/Lucide ArrowLeft: horizontal shaft + 45-degree arrow wings
        let _ = MoveToEx(hdc, cx + 6, cy, None);
        let _ = LineTo(hdc, cx - 6, cy);

        let _ = MoveToEx(hdc, cx - 1, cy - 5, None);
        let _ = LineTo(hdc, cx - 6, cy);
        let _ = LineTo(hdc, cx - 1, cy + 5);

        let _ = SelectObject(hdc, old_pen);
        let _ = DeleteObject(pen);
    }

    unsafe fn draw_vector_forward(hdc: HDC, cx: i32, cy: i32, color: windows::Win32::Foundation::COLORREF) {
        let pen = CreatePen(PS_SOLID, 2, color);
        let old_pen = SelectObject(hdc, pen);

        // Shadcn/Lucide ArrowRight: horizontal shaft + 45-degree arrow wings
        let _ = MoveToEx(hdc, cx - 6, cy, None);
        let _ = LineTo(hdc, cx + 6, cy);

        let _ = MoveToEx(hdc, cx + 1, cy - 5, None);
        let _ = LineTo(hdc, cx + 6, cy);
        let _ = LineTo(hdc, cx + 1, cy + 5);

        let _ = SelectObject(hdc, old_pen);
        let _ = DeleteObject(pen);
    }

    unsafe fn draw_vector_reload(hdc: HDC, cx: i32, cy: i32, color: windows::Win32::Foundation::COLORREF) {
        let pen = CreatePen(PS_SOLID, 2, color);
        let old_pen = SelectObject(hdc, pen);

        // Shadcn/Lucide RotateCw: smooth circular arc
        let arc_pts = [
            (cx + 4, cy - 3),
            (cx + 1, cy - 5),
            (cx - 3, cy - 4),
            (cx - 5, cy - 1),
            (cx - 5, cy + 2),
            (cx - 3, cy + 5),
            (cx + 1, cy + 5),
            (cx + 4, cy + 3),
            (cx + 5, cy),
        ];

        let _ = MoveToEx(hdc, arc_pts[0].0, arc_pts[0].1, None);
        for pt in &arc_pts[1..] {
            let _ = LineTo(hdc, pt.0, pt.1);
        }

        // Arrow tip at top right pointing clockwise
        let _ = MoveToEx(hdc, cx + 1, cy - 5, None);
        let _ = LineTo(hdc, cx + 5, cy - 3);
        let _ = LineTo(hdc, cx + 4, cy + 1);

        let _ = SelectObject(hdc, old_pen);
        let _ = DeleteObject(pen);
    }

    unsafe fn draw_vector_home(hdc: HDC, cx: i32, cy: i32, color: windows::Win32::Foundation::COLORREF) {
        let pen = CreatePen(PS_SOLID, 2, color);
        let old_pen = SelectObject(hdc, pen);

        // Shadcn/Lucide Home: crisp pitched roof
        let _ = MoveToEx(hdc, cx - 6, cy - 1, None);
        let _ = LineTo(hdc, cx, cy - 7);
        let _ = LineTo(hdc, cx + 6, cy - 1);

        // Walls with bottom doorway opening
        let _ = MoveToEx(hdc, cx - 5, cy, None);
        let _ = LineTo(hdc, cx - 5, cy + 6);
        let _ = LineTo(hdc, cx - 2, cy + 6);
        let _ = LineTo(hdc, cx - 2, cy + 2);
        let _ = LineTo(hdc, cx + 2, cy + 2);
        let _ = LineTo(hdc, cx + 2, cy + 6);
        let _ = LineTo(hdc, cx + 5, cy + 6);
        let _ = LineTo(hdc, cx + 5, cy);

        let _ = SelectObject(hdc, old_pen);
        let _ = DeleteObject(pen);
    }

    unsafe fn draw_vector_bookmark(hdc: HDC, cx: i32, cy: i32, color: windows::Win32::Foundation::COLORREF, is_filled: bool) {
        // Lucide Bookmark ribbon icon with bottom notch
        let pts = [
            POINT { x: cx - 5, y: cy - 7 },
            POINT { x: cx + 5, y: cy - 7 },
            POINT { x: cx + 5, y: cy + 7 },
            POINT { x: cx, y: cy + 3 },
            POINT { x: cx - 5, y: cy + 7 },
        ];

        if is_filled {
            let brush = CreateSolidBrush(color);
            let old_brush = SelectObject(hdc, brush);
            let pen = CreatePen(PS_SOLID, 1, color);
            let old_pen = SelectObject(hdc, pen);
            let _ = windows::Win32::Graphics::Gdi::Polygon(hdc, &pts);
            let _ = SelectObject(hdc, old_brush);
            let _ = SelectObject(hdc, old_pen);
            let _ = DeleteObject(brush);
            let _ = DeleteObject(pen);
        } else {
            let pen = CreatePen(PS_SOLID, 2, color);
            let old_pen = SelectObject(hdc, pen);
            let stock_brush = windows::Win32::Graphics::Gdi::GetStockObject(windows::Win32::Graphics::Gdi::HOLLOW_BRUSH);
            let old_brush = SelectObject(hdc, stock_brush);
            let _ = windows::Win32::Graphics::Gdi::Polygon(hdc, &pts);
            let _ = SelectObject(hdc, old_brush);
            let _ = SelectObject(hdc, old_pen);
            let _ = DeleteObject(pen);
        }
    }

    unsafe fn draw_vector_shield_check(hdc: HDC, cx: i32, cy: i32, color: windows::Win32::Foundation::COLORREF) {
        let pen = CreatePen(PS_SOLID, 1, color);
        let old_pen = SelectObject(hdc, pen);

        // Shield contour
        let pts = [
            POINT { x: cx - 5, y: cy - 6 },
            POINT { x: cx + 5, y: cy - 6 },
            POINT { x: cx + 5, y: cy + 1 },
            POINT { x: cx, y: cy + 6 },
            POINT { x: cx - 5, y: cy + 1 },
        ];
        let stock_brush = windows::Win32::Graphics::Gdi::GetStockObject(windows::Win32::Graphics::Gdi::HOLLOW_BRUSH);
        let old_brush = SelectObject(hdc, stock_brush);
        let _ = windows::Win32::Graphics::Gdi::Polygon(hdc, &pts);
        let _ = SelectObject(hdc, old_brush);

        // Checkmark inside shield
        let _ = MoveToEx(hdc, cx - 3, cy, None);
        let _ = LineTo(hdc, cx - 1, cy + 2);
        let _ = LineTo(hdc, cx + 3, cy - 2);

        let _ = SelectObject(hdc, old_pen);
        let _ = DeleteObject(pen);
    }

    unsafe fn draw_vector_x(hdc: HDC, cx: i32, cy: i32, color: windows::Win32::Foundation::COLORREF) {
        let pen = CreatePen(PS_SOLID, 1, color);
        let old_pen = SelectObject(hdc, pen);

        let _ = MoveToEx(hdc, cx - 4, cy - 4, None);
        let _ = LineTo(hdc, cx + 5, cy + 5);

        let _ = MoveToEx(hdc, cx + 4, cy - 4, None);
        let _ = LineTo(hdc, cx - 5, cy + 5);

        let _ = SelectObject(hdc, old_pen);
        let _ = DeleteObject(pen);
    }

    unsafe fn draw_vector_menu(hdc: HDC, cx: i32, cy: i32, color: windows::Win32::Foundation::COLORREF) {
        let pen = CreatePen(PS_SOLID, 2, color);
        let old_pen = SelectObject(hdc, pen);

        // Shadcn/Lucide Menu: 3 crisp horizontal bars
        let _ = MoveToEx(hdc, cx - 6, cy - 5, None);
        let _ = LineTo(hdc, cx + 6, cy - 5);

        let _ = MoveToEx(hdc, cx - 6, cy, None);
        let _ = LineTo(hdc, cx + 6, cy);

        let _ = MoveToEx(hdc, cx - 6, cy + 5, None);
        let _ = LineTo(hdc, cx + 6, cy + 5);

        let _ = SelectObject(hdc, old_pen);
        let _ = DeleteObject(pen);
    }

    unsafe fn draw_vector_plus(hdc: HDC, cx: i32, cy: i32, color: windows::Win32::Foundation::COLORREF) {
        let pen = CreatePen(PS_SOLID, 2, color);
        let old_pen = SelectObject(hdc, pen);

        let _ = MoveToEx(hdc, cx, cy - 5, None);
        let _ = LineTo(hdc, cx, cy + 5);

        let _ = MoveToEx(hdc, cx - 5, cy, None);
        let _ = LineTo(hdc, cx + 5, cy);

        let _ = SelectObject(hdc, old_pen);
        let _ = DeleteObject(pen);
    }

    unsafe fn draw_caption_minimize(hdc: HDC, cx: i32, cy: i32, color: windows::Win32::Foundation::COLORREF) {
        let pen = CreatePen(PS_SOLID, 1, color);
        let old_pen = SelectObject(hdc, pen);

        let _ = MoveToEx(hdc, cx - 5, cy, None);
        let _ = LineTo(hdc, cx + 5, cy);

        let _ = SelectObject(hdc, old_pen);
        let _ = DeleteObject(pen);
    }

    unsafe fn draw_caption_maximize(hdc: HDC, cx: i32, cy: i32, is_maximized: bool, color: windows::Win32::Foundation::COLORREF) {
        let pen = CreatePen(PS_SOLID, 1, color);
        let old_pen = SelectObject(hdc, pen);

        if !is_maximized {
            // Single square box
            let _ = MoveToEx(hdc, cx - 5, cy - 5, None);
            let _ = LineTo(hdc, cx + 5, cy - 5);
            let _ = LineTo(hdc, cx + 5, cy + 5);
            let _ = LineTo(hdc, cx - 5, cy + 5);
            let _ = LineTo(hdc, cx - 5, cy - 5);
        } else {
            // Restore overlapping squares
            // Back square
            let _ = MoveToEx(hdc, cx - 2, cy - 5, None);
            let _ = LineTo(hdc, cx + 5, cy - 5);
            let _ = LineTo(hdc, cx + 5, cy + 2);

            // Front square
            let _ = MoveToEx(hdc, cx - 5, cy - 2, None);
            let _ = LineTo(hdc, cx + 2, cy - 2);
            let _ = LineTo(hdc, cx + 2, cy + 5);
            let _ = LineTo(hdc, cx - 5, cy + 5);
            let _ = LineTo(hdc, cx - 5, cy - 2);
        }

        let _ = SelectObject(hdc, old_pen);
        let _ = DeleteObject(pen);
    }

    unsafe fn draw_caption_close(hdc: HDC, cx: i32, cy: i32, color: windows::Win32::Foundation::COLORREF) {
        let pen = CreatePen(PS_SOLID, 1, color);
        let old_pen = SelectObject(hdc, pen);

        let _ = MoveToEx(hdc, cx - 5, cy - 5, None);
        let _ = LineTo(hdc, cx + 5, cy + 5);

        let _ = MoveToEx(hdc, cx + 5, cy - 5, None);
        let _ = LineTo(hdc, cx - 5, cy + 5);

        let _ = SelectObject(hdc, old_pen);
        let _ = DeleteObject(pen);
    }

    unsafe fn draw_brand_logo(hdc: HDC, cx: i32, cy: i32, is_hovered: bool) {
        let null_pen = CreatePen(PS_NULL, 0, rgb(0, 0, 0));
        let old_pen = SelectObject(hdc, null_pen);

        // 1. Back arc of silver orbital ring
        let silver_pen = CreatePen(PS_SOLID, 1, if is_hovered { rgb(255, 255, 255) } else { rgb(212, 212, 216) });
        let old_silver = SelectObject(hdc, silver_pen);
        let _ = Arc(hdc, cx - 11, cy - 8, cx + 11, cy + 8, cx - 7, cy + 6, cx + 8, cy - 5);
        let _ = SelectObject(hdc, old_silver);
        let _ = DeleteObject(silver_pen);

        // 2. Left slanted leg - Crimson
        let left_brush = CreateSolidBrush(if is_hovered { rgb(255, 60, 95) } else { rgb(225, 29, 72) });
        let old_brush = SelectObject(hdc, left_brush);
        let left_pts = [
            POINT { x: cx - 6, y: cy + 7 },
            POINT { x: cx - 3, y: cy - 7 },
            POINT { x: cx - 1, y: cy - 7 },
            POINT { x: cx - 4, y: cy + 7 },
        ];
        let _ = Polygon(hdc, &left_pts);

        // 3. Fold crease top accent - Coral/Pink
        let crease_brush = CreateSolidBrush(if is_hovered { rgb(255, 185, 195) } else { rgb(253, 164, 175) });
        let _ = SelectObject(hdc, crease_brush);
        let crease_pts = [
            POINT { x: cx - 3, y: cy - 7 },
            POINT { x: cx, y: cy - 7 },
            POINT { x: cx - 2, y: cy - 3 },
            POINT { x: cx - 4, y: cy - 3 },
        ];
        let _ = Polygon(hdc, &crease_pts);

        // 4. Center bold diagonal - Deep Crimson
        let diag_brush = CreateSolidBrush(if is_hovered { rgb(244, 63, 94) } else { rgb(180, 15, 55) });
        let _ = SelectObject(hdc, diag_brush);
        let diag_pts = [
            POINT { x: cx - 2, y: cy - 7 },
            POINT { x: cx + 1, y: cy - 7 },
            POINT { x: cx + 6, y: cy + 7 },
            POINT { x: cx + 3, y: cy + 7 },
        ];
        let _ = Polygon(hdc, &diag_pts);

        // 5. Right slanted leg - Crimson
        let right_brush = CreateSolidBrush(if is_hovered { rgb(255, 60, 95) } else { rgb(225, 29, 72) });
        let _ = SelectObject(hdc, right_brush);
        let right_pts = [
            POINT { x: cx + 4, y: cy - 7 },
            POINT { x: cx + 7, y: cy - 7 },
            POINT { x: cx + 5, y: cy + 7 },
            POINT { x: cx + 2, y: cy + 7 },
        ];
        let _ = Polygon(hdc, &right_pts);

        // 6. Front arc of silver orbital ring (sweeping across the right leg)
        let front_silver_pen = CreatePen(PS_SOLID, 1, if is_hovered { rgb(255, 255, 255) } else { rgb(240, 240, 245) });
        let _ = SelectObject(hdc, front_silver_pen);
        let _ = Arc(hdc, cx - 11, cy - 8, cx + 11, cy + 8, cx + 8, cy - 5, cx - 7, cy + 6);
        let _ = DeleteObject(front_silver_pen);

        // Clean up GDI objects
        let _ = SelectObject(hdc, old_brush);
        let _ = SelectObject(hdc, old_pen);
        let _ = DeleteObject(left_brush);
        let _ = DeleteObject(crease_brush);
        let _ = DeleteObject(diag_brush);
        let _ = DeleteObject(right_brush);
        let _ = DeleteObject(null_pen);
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
            let _ = DeleteObject(self.font);
        }
    }
}
