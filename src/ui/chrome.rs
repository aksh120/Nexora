use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, CreateFontW, CreatePen, CreateSolidBrush,
    DeleteDC, DeleteObject, DrawTextW, FillRect, LineTo, MoveToEx, SelectObject,
    SetBkMode, SetTextColor, DT_CENTER, DT_END_ELLIPSIS, DT_LEFT, DT_SINGLELINE, DT_VCENTER,
    FW_NORMAL, HDC, HFONT, PS_SOLID, SRCCOPY, TRANSPARENT,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_RETURN;
use windows::Win32::UI::WindowsAndMessaging::{
    CallWindowProcW, CreateWindowExW, GetWindowLongPtrW, GetWindowTextLengthW, GetWindowTextW,
    MoveWindow, SendMessageW, SetWindowLongPtrW, SetWindowTextW, GWLP_USERDATA, GWLP_WNDPROC,
    HMENU, WINDOW_EX_STYLE, WINDOW_STYLE, WM_CHAR, WM_KEYDOWN, WM_SETFONT, WNDPROC, WS_CHILD,
    WS_TABSTOP, WS_VISIBLE,
};

use crate::core::error::{NexoraError, Result};
use crate::tabs::Tab;
use crate::ui::layout::{BrowserLayout, BUTTON_GAP, BUTTON_HEIGHT, BUTTON_WIDTH, OMNIBOX_HEIGHT, TAB_BAR_HEIGHT, TOP_CHROME_HEIGHT};
use crate::ui::theme::{rgb, ThemePalette};

pub const IDC_BTN_BACK: usize = 1001;
pub const IDC_BTN_FORWARD: usize = 1002;
pub const IDC_BTN_RELOAD: usize = 1003;
pub const IDC_BTN_HOME: usize = 1004;
pub const IDC_OMNIBOX: usize = 1005;
pub const IDC_BTN_NEW_TAB: usize = 1006;
pub const IDC_BTN_MENU: usize = 1007;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChromeHitResult {
    Tab(usize),
    TabClose(usize),
    NewTab,
    None,
}

pub struct BrowserChrome {
    pub hwnd_parent: HWND,
    pub hwnd_back: HWND,
    pub hwnd_forward: HWND,
    pub hwnd_reload: HWND,
    pub hwnd_home: HWND,
    pub hwnd_omnibox: HWND,
    pub hwnd_new_tab: HWND,
    pub hwnd_menu: HWND,
    font: HFONT,
    pub theme: ThemePalette,
}

struct OmniboxSubclassContext {
    orig_proc: WNDPROC,
    on_enter: Box<dyn Fn(String) + 'static>,
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

            let btn_style = WINDOW_STYLE((WS_CHILD.0 | WS_VISIBLE.0 | WS_TABSTOP.0) as u32);
            let edit_style = WINDOW_STYLE((WS_CHILD.0 | WS_VISIBLE.0 | WS_TABSTOP.0 | 0x0080) as u32); // ES_AUTOHSCROLL = 0x0080

            // 1. Back button
            let hwnd_back = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                w!("BUTTON"),
                w!(" \u{25C0} "),
                btn_style,
                8, 38, BUTTON_WIDTH, BUTTON_HEIGHT,
                parent,
                HMENU(IDC_BTN_BACK as *mut core::ffi::c_void),
                hinstance,
                None,
            ).map_err(|e| NexoraError::Win32(format!("Failed to create Back button: {}", e)))?;

            // 2. Forward button
            let hwnd_forward = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                w!("BUTTON"),
                w!(" \u{25B6} "),
                btn_style,
                8 + BUTTON_WIDTH + BUTTON_GAP, 38, BUTTON_WIDTH, BUTTON_HEIGHT,
                parent,
                HMENU(IDC_BTN_FORWARD as *mut core::ffi::c_void),
                hinstance,
                None,
            ).map_err(|e| NexoraError::Win32(format!("Failed to create Forward button: {}", e)))?;

            // 3. Reload button
            let hwnd_reload = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                w!("BUTTON"),
                w!(" \u{21BB} "),
                btn_style,
                8 + (BUTTON_WIDTH + BUTTON_GAP) * 2, 38, BUTTON_WIDTH, BUTTON_HEIGHT,
                parent,
                HMENU(IDC_BTN_RELOAD as *mut core::ffi::c_void),
                hinstance,
                None,
            ).map_err(|e| NexoraError::Win32(format!("Failed to create Reload button: {}", e)))?;

            // 4. Home button
            let hwnd_home = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                w!("BUTTON"),
                w!(" \u{2302} "),
                btn_style,
                8 + (BUTTON_WIDTH + BUTTON_GAP) * 3, 38, BUTTON_WIDTH, BUTTON_HEIGHT,
                parent,
                HMENU(IDC_BTN_HOME as *mut core::ffi::c_void),
                hinstance,
                None,
            ).map_err(|e| NexoraError::Win32(format!("Failed to create Home button: {}", e)))?;

            // 5. New Tab button in tab bar
            let hwnd_new_tab = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                w!("BUTTON"),
                w!("+"),
                btn_style,
                240, 5, 26, 28,
                parent,
                HMENU(IDC_BTN_NEW_TAB as *mut core::ffi::c_void),
                hinstance,
                None,
            ).map_err(|e| NexoraError::Win32(format!("Failed to create New Tab button: {}", e)))?;

            // 6. Omnibox Edit control
            let hwnd_omnibox = CreateWindowExW(
                WINDOW_EX_STYLE(0x00000200), // WS_EX_CLIENTEDGE
                w!("EDIT"),
                PCWSTR::null(),
                edit_style,
                160, 42, 600, OMNIBOX_HEIGHT,
                parent,
                HMENU(IDC_OMNIBOX as *mut core::ffi::c_void),
                hinstance,
                None,
            ).map_err(|e| NexoraError::Win32(format!("Failed to create Omnibox edit: {}", e)))?;

            // 7. Menu / Diagnostics button
            let hwnd_menu = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                w!("BUTTON"),
                w!(" \u{2630} "),
                btn_style,
                770, 38, BUTTON_WIDTH, BUTTON_HEIGHT,
                parent,
                HMENU(IDC_BTN_MENU as *mut core::ffi::c_void),
                hinstance,
                None,
            ).map_err(|e| NexoraError::Win32(format!("Failed to create Menu button: {}", e)))?;

            // Set Segoe UI font on all child controls
            let wparam_font = WPARAM(font.0 as usize);
            let lparam_redraw = LPARAM(1);
            let _ = SendMessageW(hwnd_back, WM_SETFONT, wparam_font, lparam_redraw);
            let _ = SendMessageW(hwnd_forward, WM_SETFONT, wparam_font, lparam_redraw);
            let _ = SendMessageW(hwnd_reload, WM_SETFONT, wparam_font, lparam_redraw);
            let _ = SendMessageW(hwnd_home, WM_SETFONT, wparam_font, lparam_redraw);
            let _ = SendMessageW(hwnd_new_tab, WM_SETFONT, wparam_font, lparam_redraw);
            let _ = SendMessageW(hwnd_omnibox, WM_SETFONT, wparam_font, lparam_redraw);
            let _ = SendMessageW(hwnd_menu, WM_SETFONT, wparam_font, lparam_redraw);

            // Subclass the Omnibox to intercept Enter key (VK_RETURN)
            let orig_proc = GetWindowLongPtrW(hwnd_omnibox, GWLP_WNDPROC);
            let ctx = Box::new(OmniboxSubclassContext {
                orig_proc: std::mem::transmute(orig_proc),
                on_enter: Box::new(on_navigate),
            });
            SetWindowLongPtrW(hwnd_omnibox, GWLP_USERDATA, Box::into_raw(ctx) as isize);
            SetWindowLongPtrW(hwnd_omnibox, GWLP_WNDPROC, Self::omnibox_subclass_proc as *const () as isize);

            Ok(Self {
                hwnd_parent: parent,
                hwnd_back,
                hwnd_forward,
                hwnd_reload,
                hwnd_home,
                hwnd_omnibox,
                hwnd_new_tab,
                hwnd_menu,
                font,
                theme,
            })
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

    pub fn layout_controls(&self, layout: &BrowserLayout, tab_count: usize) {
        unsafe {
            let back = layout.back_button_rect();
            let _ = MoveWindow(self.hwnd_back, back.0, back.1, back.2, back.3, true);

            let fwd = layout.forward_button_rect();
            let _ = MoveWindow(self.hwnd_forward, fwd.0, fwd.1, fwd.2, fwd.3, true);

            let reload = layout.reload_button_rect();
            let _ = MoveWindow(self.hwnd_reload, reload.0, reload.1, reload.2, reload.3, true);

            let home = layout.home_button_rect();
            let _ = MoveWindow(self.hwnd_home, home.0, home.1, home.2, home.3, true);

            let menu = layout.menu_button_rect();
            let _ = MoveWindow(self.hwnd_menu, menu.0, menu.1, menu.2, menu.3, true);

            let omnibox = layout.omnibox_rect();
            let _ = MoveWindow(self.hwnd_omnibox, omnibox.0, omnibox.1, omnibox.2, omnibox.3, true);

            let new_tab = layout.new_tab_button_rect(tab_count);
            let _ = MoveWindow(self.hwnd_new_tab, new_tab.0, new_tab.1, new_tab.2, new_tab.3, true);
        }
    }

    pub fn hit_test(
        &self,
        x: i32,
        y: i32,
        layout: &BrowserLayout,
        tabs: &[Tab],
    ) -> ChromeHitResult {
        if y < 0 || y > TAB_BAR_HEIGHT {
            return ChromeHitResult::None;
        }

        let rects = layout.calculate_tab_rects(tabs.len());
        for (i, (tx, ty, tw, th)) in rects.iter().enumerate() {
            if x >= *tx && x <= tx + tw && y >= *ty && y <= ty + th {
                let tab_id = tabs[i].id;
                // Close button is in rightmost 22px
                if x >= (tx + tw - 22) {
                    return ChromeHitResult::TabClose(tab_id);
                }
                return ChromeHitResult::Tab(tab_id);
            }
        }

        // New tab button hit
        let new_tab_rect = layout.new_tab_button_rect(tabs.len());
        if x >= new_tab_rect.0 && x <= new_tab_rect.0 + new_tab_rect.2
            && y >= new_tab_rect.1 && y <= new_tab_rect.1 + new_tab_rect.3
        {
            return ChromeHitResult::NewTab;
        }

        ChromeHitResult::None
    }

    pub fn render(
        &self,
        hdc: HDC,
        layout: &BrowserLayout,
        tabs: &[Tab],
        active_tab_id: Option<usize>,
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

            // 3. Draw Divider Line
            let div_pen = CreatePen(PS_SOLID, 1, self.theme.divider_color);
            let old_pen = SelectObject(mem_dc, div_pen);
            let _ = MoveToEx(mem_dc, 0, TOP_CHROME_HEIGHT - 1, None);
            let _ = LineTo(mem_dc, layout.window_width, TOP_CHROME_HEIGHT - 1);

            // 4. Draw Tabs
            let tab_rects = layout.calculate_tab_rects(tabs.len());
            SetBkMode(mem_dc, TRANSPARENT);

            for (i, tab) in tabs.iter().enumerate() {
                if i >= tab_rects.len() {
                    break;
                }
                let (tx, ty, tw, th) = tab_rects[i];
                let is_active = Some(tab.id) == active_tab_id;

                let tab_bg = if is_active {
                    self.theme.tab_active_bg
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
                    left: tx + 8,
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

                // Close Button ("\u{00D7}" = ×)
                let mut close_str: [u16; 2] = [0x00D7, 0];
                let mut close_rect = RECT {
                    left: tx + tw - 20,
                    top: ty,
                    right: tx + tw - 4,
                    bottom: ty + th,
                };
                SetTextColor(mem_dc, if is_active { rgb(180, 180, 180) } else { rgb(120, 120, 120) });
                DrawTextW(
                    mem_dc,
                    &mut close_str[..1],
                    &mut close_rect,
                    DT_CENTER | DT_VCENTER | DT_SINGLELINE,
                );
            }

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
            }
            WM_CHAR => {
                if wparam.0 == VK_RETURN.0 as usize {
                    // Suppress ding sound on Enter
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

