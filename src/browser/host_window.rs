use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, CreateSolidBrush, EndPaint, InvalidateRect, HBRUSH, PAINTSTRUCT,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetClientRect, GetMessageW,
    GetWindowLongPtrW, KillTimer, PostQuitMessage, RegisterClassExW, SetTimer, SetWindowLongPtrW,
    ShowWindow, TranslateMessage, CW_USEDEFAULT, GWLP_USERDATA, MSG, SW_SHOW, WINDOW_EX_STYLE,
    WM_CLOSE, WM_COMMAND, WM_DESTROY, WM_DPICHANGED, WM_ERASEBKGND, WM_LBUTTONDOWN,
    WM_PAINT, WM_SIZE, WM_TIMER, WNDCLASSEXW, WS_OVERLAPPEDWINDOW,
};

use crate::core::error::{NexoraError, Result};
use crate::engine::WebViewEngine;
use crate::navigation::Omnibox;
use crate::performance::SmartSuspensionEngine;
use crate::tabs::TabManager;
use crate::ui::chrome::{
    BrowserChrome, ChromeHitResult, IDC_BTN_BACK, IDC_BTN_FORWARD, IDC_BTN_HOME,
    IDC_BTN_MENU, IDC_BTN_NEW_TAB, IDC_BTN_RELOAD,
};
use crate::ui::layout::{BrowserLayout, TAB_BAR_HEIGHT};
use crate::ui::theme::ThemePalette;

pub const IDT_SUSPENSION_TIMER: usize = 1001;

pub struct WindowContext {
    pub layout: BrowserLayout,
    pub theme: ThemePalette,
    pub tab_manager: Rc<RefCell<TabManager>>,
    pub chrome: Option<BrowserChrome>,
    pub webview: Option<Rc<WebViewEngine>>,
    pub homepage: String,
    pub default_search_engine: String,
    pub is_closing: Arc<AtomicBool>,
    pub suspension_engine: SmartSuspensionEngine,
    pub recovery_manager: Option<Arc<crate::storage::CrashRecoveryManager>>,
    pub session_manager: Option<Arc<crate::storage::SessionManager>>,
}

pub struct HostWindow {
    hwnd: HWND,
    is_closing: Arc<AtomicBool>,
}

impl HostWindow {
    pub fn new(
        title: &str,
        width: i32,
        height: i32,
        theme: ThemePalette,
        homepage: &str,
        default_search: &str,
        tab_manager: Rc<RefCell<TabManager>>,
        smart_suspension: bool,
        suspension_timeout_minutes: u32,
    ) -> Result<Self> {
        let is_closing = Arc::new(AtomicBool::new(false));

        unsafe {
            let hinstance: HINSTANCE = GetModuleHandleW(None)
                .map_err(|e| NexoraError::Win32(format!("Failed to get module handle: {}", e)))?
                .into();

            let class_name = w!("NexoraBrowserHostWindowClass");

            // Window background brush
            let bg_brush: HBRUSH = CreateSolidBrush(theme.window_bg);

            let wnd_class = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                style: windows::Win32::UI::WindowsAndMessaging::WNDCLASS_STYLES(0),
                lpfnWndProc: Some(Self::wnd_proc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: hinstance,
                hIcon: windows::Win32::UI::WindowsAndMessaging::HICON::default(),
                hCursor: windows::Win32::UI::WindowsAndMessaging::LoadCursorW(
                    None,
                    windows::Win32::UI::WindowsAndMessaging::IDC_ARROW,
                )
                .unwrap_or_default(),
                hbrBackground: bg_brush,
                lpszMenuName: PCWSTR::null(),
                lpszClassName: class_name,
                hIconSm: windows::Win32::UI::WindowsAndMessaging::HICON::default(),
            };

            let _ = RegisterClassExW(&wnd_class);

            let wide_title: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();

            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                class_name,
                PCWSTR(wide_title.as_ptr()),
                WS_OVERLAPPEDWINDOW,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                width,
                height,
                None,
                None,
                hinstance,
                None,
            )
            .map_err(|e| NexoraError::Win32(format!("Failed to create host window: {}", e)))?;

            // Apply native dark title bar if theme is dark
            ThemePalette::apply_to_window(hwnd, theme.is_dark);

            let layout = BrowserLayout::new(width, height);
            let suspension_engine = SmartSuspensionEngine::new(suspension_timeout_minutes, smart_suspension);

            let context = Box::new(WindowContext {
                layout,
                theme,
                tab_manager,
                chrome: None,
                webview: None,
                homepage: homepage.to_string(),
                default_search_engine: default_search.to_string(),
                is_closing: is_closing.clone(),
                suspension_engine,
                recovery_manager: None,
                session_manager: None,
            });

            SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(context) as isize);

            // Start 30-second periodic timer for memory pressure & smart tab suspension
            let _ = SetTimer(hwnd, IDT_SUSPENSION_TIMER, 30_000, None);

            log::info!("Native Win32 Host Window created (HWND: {:?}, size: {}x{})", hwnd.0, width, height);

            Ok(Self { hwnd, is_closing })
        }
    }

    pub fn set_recovery_manager(
        &self,
        recovery: Arc<crate::storage::CrashRecoveryManager>,
        session: Arc<crate::storage::SessionManager>,
    ) {
        unsafe {
            let ptr = GetWindowLongPtrW(self.hwnd, GWLP_USERDATA) as *mut WindowContext;
            if !ptr.is_null() {
                (*ptr).recovery_manager = Some(recovery);
                (*ptr).session_manager = Some(session);
            }
        }
    }

    pub fn hwnd(&self) -> HWND {
        self.hwnd
    }

    pub fn show(&self) {
        unsafe {
            let _ = ShowWindow(self.hwnd, SW_SHOW);
        }
    }

    pub fn init_chrome(&self) -> Result<()> {
        unsafe {
            let ptr = GetWindowLongPtrW(self.hwnd, GWLP_USERDATA) as *mut WindowContext;
            if ptr.is_null() {
                return Err(NexoraError::Other("Window context is null".into()));
            }

            let hwnd_parent = self.hwnd;
            let chrome = BrowserChrome::new(
                self.hwnd,
                (*ptr).theme,
                move |text| {
                    let ptr = GetWindowLongPtrW(hwnd_parent, GWLP_USERDATA) as *mut WindowContext;
                    if !ptr.is_null() {
                        let target = Omnibox::parse_input(&text, &(*ptr).default_search_engine);
                        let dest_url = match target {
                            crate::navigation::NavigationTarget::Url(u) => u,
                            crate::navigation::NavigationTarget::Search { url, .. } => url,
                            crate::navigation::NavigationTarget::Internal(u) => u,
                        };

                        if let Some(ref webview) = (*ptr).webview {
                            let _ = webview.navigate(&dest_url);
                        }

                        // Update active tab URL
                        let mut tabs = (*ptr).tab_manager.borrow_mut();
                        if let Some(active) = tabs.get_active_tab_mut() {
                            active.url = dest_url;
                        }
                        drop(tabs);

                        let _ = InvalidateRect(hwnd_parent, None, false);
                    }
                },
            )?;

            let tab_count = (*ptr).tab_manager.borrow().tab_count();
            chrome.layout_controls(&(*ptr).layout, tab_count);
            (*ptr).chrome = Some(chrome);

            Ok(())
        }
    }

    pub fn set_webview(&self, webview: Rc<WebViewEngine>) {
        unsafe {
            let ptr = GetWindowLongPtrW(self.hwnd, GWLP_USERDATA) as *mut WindowContext;
            if !ptr.is_null() {
                let (x, y, w, h) = (*ptr).layout.webview_rect();
                let _ = webview.set_bounds(x, y, w, h);
                (*ptr).webview = Some(webview);
            }
        }
    }

    pub fn update_omnibox_url(&self, url: &str) {
        unsafe {
            let ptr = GetWindowLongPtrW(self.hwnd, GWLP_USERDATA) as *mut WindowContext;
            if !ptr.is_null() {
                if let Some(ref chrome) = (*ptr).chrome {
                    chrome.set_omnibox_text(url);
                }
            }
        }
    }

    pub fn update_active_tab_title(&self, title: &str) {
        unsafe {
            let ptr = GetWindowLongPtrW(self.hwnd, GWLP_USERDATA) as *mut WindowContext;
            if !ptr.is_null() {
                let mut tabs = (*ptr).tab_manager.borrow_mut();
                if let Some(active_id) = tabs.active_tab_id() {
                    tabs.update_title(active_id, title);
                }
                drop(tabs);
                let _ = InvalidateRect(self.hwnd, None, false);
            }
        }
    }

    pub fn refresh_chrome(&self) {
        unsafe {
            let _ = InvalidateRect(self.hwnd, None, false);
        }
    }

    pub fn get_client_bounds(&self) -> (i32, i32, i32, i32) {
        unsafe {
            let mut rect = RECT::default();
            let _ = GetClientRect(self.hwnd, &mut rect);
            (rect.left, rect.top, rect.right - rect.left, rect.bottom - rect.top)
        }
    }

    pub fn is_closing(&self) -> bool {
        self.is_closing.load(Ordering::SeqCst)
    }

    pub fn run_message_loop(&self) {
        unsafe {
            let mut msg = MSG::default();
            log::info!("Starting native Win32 message pump...");
            while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
            log::info!("Native Win32 message pump exited.");
        }
    }

    unsafe extern "system" fn wnd_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut WindowContext;

        match msg {
            WM_PAINT => {
                let mut ps = PAINTSTRUCT::default();
                let hdc = BeginPaint(hwnd, &mut ps);
                if !ptr.is_null() {
                    if let Some(ref chrome) = (*ptr).chrome {
                        let tabs = (*ptr).tab_manager.borrow();
                        chrome.render(hdc, &(*ptr).layout, tabs.tabs(), tabs.active_tab_id());
                    }
                }
                let _ = EndPaint(hwnd, &ps);
                LRESULT(0)
            }
            WM_SIZE => {
                if !ptr.is_null() {
                    let mut rect = RECT::default();
                    if GetClientRect(hwnd, &mut rect).is_ok() {
                        let width = rect.right - rect.left;
                        let height = rect.bottom - rect.top;
                        (*ptr).layout = BrowserLayout::new(width, height);

                        let tab_count = (*ptr).tab_manager.borrow().tab_count();
                        if let Some(ref chrome) = (*ptr).chrome {
                            chrome.layout_controls(&(*ptr).layout, tab_count);
                        }

                        if let Some(ref webview) = (*ptr).webview {
                            let (wx, wy, ww, wh) = (*ptr).layout.webview_rect();
                            let _ = webview.set_bounds(wx, wy, ww, wh);
                        }

                        let _ = InvalidateRect(hwnd, None, false);
                    }
                }
                LRESULT(0)
            }
            WM_LBUTTONDOWN => {
                if !ptr.is_null() {
                    let x = (lparam.0 & 0xffff) as i16 as i32;
                    let y = ((lparam.0 >> 16) & 0xffff) as i16 as i32;

                    if y <= TAB_BAR_HEIGHT {
                        let hit = if let Some(ref chrome) = (*ptr).chrome {
                            let tabs = (*ptr).tab_manager.borrow();
                            chrome.hit_test(x, y, &(*ptr).layout, tabs.tabs())
                        } else {
                            ChromeHitResult::None
                        };

                        match hit {
                            ChromeHitResult::Tab(id) => {
                                let mut tabs = (*ptr).tab_manager.borrow_mut();
                                let was_suspended = tabs.get_tab(id).map(|t| t.is_suspended()).unwrap_or(false);
                                if tabs.switch_tab(id) {
                                    if let Some(active) = tabs.get_active_tab() {
                                        let url = active.url.clone();
                                        drop(tabs);
                                        if was_suspended {
                                            log::info!("Waking suspended/lazy tab #{} and navigating to {}", id, url);
                                        }
                                        if let Some(ref webview) = (*ptr).webview {
                                            let _ = webview.navigate(&url);
                                        }
                                        if let Some(ref chrome) = (*ptr).chrome {
                                            chrome.set_omnibox_text(&url);
                                        }
                                        let _ = InvalidateRect(hwnd, None, false);
                                    }
                                }
                            }
                            ChromeHitResult::TabClose(id) => {
                                let mut tabs = (*ptr).tab_manager.borrow_mut();
                                let new_active = tabs.close_tab(id);
                                let maybe_url = new_active.and_then(|new_id| tabs.get_tab(new_id).map(|t| t.url.clone()));
                                drop(tabs);

                                if let Some(url) = maybe_url {
                                    if let Some(ref webview) = (*ptr).webview {
                                        let _ = webview.navigate(&url);
                                    }
                                    if let Some(ref chrome) = (*ptr).chrome {
                                        chrome.set_omnibox_text(&url);
                                    }
                                }

                                if let Some(ref chrome) = (*ptr).chrome {
                                    let count = (*ptr).tab_manager.borrow().tab_count();
                                    chrome.layout_controls(&(*ptr).layout, count);
                                }
                                let _ = InvalidateRect(hwnd, None, false);
                            }
                            ChromeHitResult::NewTab => {
                                let homepage = (*ptr).homepage.clone();
                                let mut tabs = (*ptr).tab_manager.borrow_mut();
                                let _ = tabs.create_tab(&homepage, Some("New Tab"));
                                drop(tabs);
                                if let Some(ref webview) = (*ptr).webview {
                                    let _ = webview.navigate(&homepage);
                                }
                                if let Some(ref chrome) = (*ptr).chrome {
                                    chrome.set_omnibox_text(&homepage);
                                    let count = (*ptr).tab_manager.borrow().tab_count();
                                    chrome.layout_controls(&(*ptr).layout, count);
                                }
                                let _ = InvalidateRect(hwnd, None, false);
                            }
                            ChromeHitResult::None => {}
                        }
                    }
                }
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
            WM_COMMAND => {
                if !ptr.is_null() {
                    let ctrl_id = (wparam.0 & 0xffff) as usize;
                    match ctrl_id {
                        IDC_BTN_BACK => {
                            if let Some(ref webview) = (*ptr).webview {
                                let _ = webview.go_back();
                            }
                        }
                        IDC_BTN_FORWARD => {
                            if let Some(ref webview) = (*ptr).webview {
                                let _ = webview.go_forward();
                            }
                        }
                        IDC_BTN_RELOAD => {
                            if let Some(ref webview) = (*ptr).webview {
                                let _ = webview.reload();
                            }
                        }
                        IDC_BTN_HOME => {
                            let homepage = (*ptr).homepage.clone();
                            if let Some(ref webview) = (*ptr).webview {
                                let _ = webview.navigate(&homepage);
                            }
                            if let Some(ref chrome) = (*ptr).chrome {
                                chrome.set_omnibox_text(&homepage);
                            }
                        }
                        IDC_BTN_NEW_TAB => {
                            let homepage = (*ptr).homepage.clone();
                            let mut tabs = (*ptr).tab_manager.borrow_mut();
                            let _ = tabs.create_tab(&homepage, Some("New Tab"));
                            drop(tabs);
                            if let Some(ref webview) = (*ptr).webview {
                                let _ = webview.navigate(&homepage);
                            }
                            if let Some(ref chrome) = (*ptr).chrome {
                                chrome.set_omnibox_text(&homepage);
                                let count = (*ptr).tab_manager.borrow().tab_count();
                                chrome.layout_controls(&(*ptr).layout, count);
                            }
                            let _ = InvalidateRect(hwnd, None, false);
                        }
                        IDC_BTN_MENU => {
                            log::info!("Toolbar Menu clicked. Navigating to nexora://settings");
                            if let Some(ref webview) = (*ptr).webview {
                                let _ = webview.navigate("nexora://settings");
                            }
                            if let Some(ref chrome) = (*ptr).chrome {
                                chrome.set_omnibox_text("nexora://settings");
                            }
                            let mut tabs = (*ptr).tab_manager.borrow_mut();
                            if let Some(active) = tabs.get_active_tab_mut() {
                                active.url = "nexora://settings".to_string();
                                active.title = "Settings - Nexora".to_string();
                            }
                            let _ = InvalidateRect(hwnd, None, false);
                        }
                        _ => {}
                    }
                }
                LRESULT(0)
            }
            WM_DPICHANGED => {
                let suggested_rect = *(lparam.0 as *const RECT);
                let _ = windows::Win32::UI::WindowsAndMessaging::SetWindowPos(
                    hwnd,
                    HWND::default(),
                    suggested_rect.left,
                    suggested_rect.top,
                    suggested_rect.right - suggested_rect.left,
                    suggested_rect.bottom - suggested_rect.top,
                    windows::Win32::UI::WindowsAndMessaging::SWP_NOZORDER
                        | windows::Win32::UI::WindowsAndMessaging::SWP_NOACTIVATE,
                );
                LRESULT(0)
            }
            WM_ERASEBKGND => {
                // Prevent flicker
                LRESULT(1)
            }
            WM_TIMER => {
                if !ptr.is_null() && wparam.0 == IDT_SUSPENSION_TIMER {
                    let mut tabs = (*ptr).tab_manager.borrow_mut();
                    let suspended = (*ptr).suspension_engine.sweep_and_suspend(&mut tabs);
                    if !suspended.is_empty() {
                        log::info!("SmartSuspension: Suspended {} idle tabs via background sweep", suspended.len());
                        let _ = InvalidateRect(hwnd, None, false);
                    }

                    // Periodic session checkpointing (TASK-801)
                    if let Some(ref recovery) = (*ptr).recovery_manager {
                        let _ = recovery.save_checkpoint(tabs.tabs(), tabs.active_tab_id());
                    }
                    if let Some(ref session_mgr) = (*ptr).session_manager {
                        let _ = session_mgr.save_session(tabs.tabs(), tabs.active_tab_id());
                    }
                }
                LRESULT(0)
            }
            WM_CLOSE => {
                let _ = KillTimer(hwnd, IDT_SUSPENSION_TIMER);
                if !ptr.is_null() {
                    (*ptr).is_closing.store(true, Ordering::SeqCst);

                    // Final atomic checkpoint and clean exit marker (TASK-801)
                    let tabs = (*ptr).tab_manager.borrow();
                    if let Some(ref recovery) = (*ptr).recovery_manager {
                        let _ = recovery.save_checkpoint(tabs.tabs(), tabs.active_tab_id());
                        let _ = recovery.mark_clean_exit();
                    }
                    if let Some(ref session_mgr) = (*ptr).session_manager {
                        let _ = session_mgr.save_session(tabs.tabs(), tabs.active_tab_id());
                    }
                }
                let _ = windows::Win32::UI::WindowsAndMessaging::DestroyWindow(hwnd);
                LRESULT(0)
            }
            WM_DESTROY => {
                let _ = KillTimer(hwnd, IDT_SUSPENSION_TIMER);
                if !ptr.is_null() {
                    let _ = Box::from_raw(ptr);
                    SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
                }
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_host_window_with_theme_and_tabs() {
        let tab_mgr = Rc::new(RefCell::new(TabManager::new()));
        let theme = ThemePalette::dark();
        let window = HostWindow::new(
            "Nexora UI Test Window",
            1000,
            700,
            theme,
            "https://www.google.com",
            "Google",
            tab_mgr.clone(),
            true,
            10,
        );
        assert!(window.is_ok(), "HostWindow creation should succeed");
        let window = window.unwrap();
        assert!(!window.hwnd().0.is_null(), "HWND should be valid and non-null");

        // Initialize chrome controls
        let chrome_res = window.init_chrome();
        assert!(chrome_res.is_ok(), "BrowserChrome initialization should succeed");

        // Cleanly close
        unsafe {
            let _ = windows::Win32::UI::WindowsAndMessaging::DestroyWindow(window.hwnd());
        }
    }
}
