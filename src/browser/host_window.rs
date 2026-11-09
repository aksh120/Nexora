use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Dwm::DwmExtendFrameIntoClientArea;
use windows::Win32::Graphics::Gdi::{
    BeginPaint, ClientToScreen, CreateSolidBrush, DeleteObject, EndPaint, InvalidateRect, SetBkColor, SetTextColor,
    HBRUSH, HDC, PAINTSTRUCT,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Controls::MARGINS;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyState, ReleaseCapture, SetCapture, SetFocus, TrackMouseEvent, TME_LEAVE, TRACKMOUSEEVENT,
    VK_ADD, VK_CONTROL, VK_ESCAPE, VK_F11, VK_F5, VK_LEFT, VK_MENU, VK_RIGHT, VK_SHIFT, VK_SUBTRACT, VK_TAB,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu, DispatchMessageW,
    GetClientRect, GetMessageW, GetWindowLongPtrW, GetWindowRect, IsZoomed, KillTimer,
    PostQuitMessage, RegisterClassExW, SendMessageW, SetTimer, SetWindowLongPtrW, SetWindowPos, ShowWindow,
    TrackPopupMenuEx, TranslateMessage, CW_USEDEFAULT, GWLP_USERDATA, MSG, SW_MAXIMIZE,
    SW_MINIMIZE, SW_RESTORE, SW_SHOW, SWP_FRAMECHANGED, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER,
    TPM_NONOTIFY, TPM_RETURNCMD, TPM_RIGHTALIGN, TPM_TOPALIGN,
    WINDOW_EX_STYLE, WM_CLOSE, WM_CTLCOLOREDIT, WM_CTLCOLORSTATIC, WM_DESTROY, WM_DPICHANGED,
    WM_ERASEBKGND, WM_KEYDOWN, WM_LBUTTONDBLCLK, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE,
    WM_MOUSEWHEEL, WM_MOVE, WM_NCCALCSIZE, WM_NCHITTEST, WM_NCLBUTTONDOWN, WM_PAINT, WM_RBUTTONUP, WM_SIZE,
    WM_TIMER, WNDCLASSEXW, WS_CLIPCHILDREN, WS_OVERLAPPEDWINDOW,
    HTBOTTOM, HTBOTTOMLEFT, HTBOTTOMRIGHT, HTCAPTION, HTCLIENT, HTLEFT, HTRIGHT, HTTOP,
    HTTOPLEFT, HTTOPRIGHT, MENU_ITEM_FLAGS, MF_CHECKED, MF_POPUP, MF_SEPARATOR, MF_STRING, MF_UNCHECKED,
};

pub const WM_MOUSELEAVE: u32 = 0x02A3;
pub const EM_SETSEL: u32 = 0x00B1;
pub const WM_SETICON: u32 = 0x0080;
pub const ICON_BIG: usize = 1;
pub const ICON_SMALL: usize = 0;

pub const VK_OEM_PLUS: u16 = 0xBB;
pub const VK_OEM_MINUS: u16 = 0xBD;

pub const IDM_NEW_TAB: usize = 2001;
pub const IDM_BOOKMARKS: usize = 2002;
pub const IDM_HISTORY: usize = 2003;
pub const IDM_SETTINGS: usize = 2004;
pub const IDM_DIAGNOSTICS: usize = 2005;
pub const IDM_EXIT: usize = 2006;

pub const IDM_TAB_DUPLICATE: usize = 3001;
pub const IDM_TAB_PIN: usize = 3002;
pub const IDM_TAB_UNPIN: usize = 3003;
pub const IDM_TAB_MUTE: usize = 3004;
pub const IDM_TAB_UNMUTE: usize = 3005;
pub const IDM_TAB_MUTE_OTHERS: usize = 3006;
pub const IDM_TAB_UNMUTE_ALL: usize = 3007;
pub const IDM_TAB_CLOSE_TAB: usize = 3008;
pub const IDM_TAB_CLOSE_OTHERS: usize = 3009;
pub const IDM_TAB_CLOSE_RIGHT: usize = 3010;

pub const IDM_PERM_CAMERA_ALLOW: usize = 4001;
pub const IDM_PERM_CAMERA_ASK: usize = 4002;
pub const IDM_PERM_CAMERA_BLOCK: usize = 4003;
pub const IDM_PERM_MIC_ALLOW: usize = 4004;
pub const IDM_PERM_MIC_ASK: usize = 4005;
pub const IDM_PERM_MIC_BLOCK: usize = 4006;
pub const IDM_PERM_GEO_ALLOW: usize = 4007;
pub const IDM_PERM_GEO_ASK: usize = 4008;
pub const IDM_PERM_GEO_BLOCK: usize = 4009;
pub const IDM_PERM_NOTIF_ALLOW: usize = 4010;
pub const IDM_PERM_NOTIF_ASK: usize = 4011;
pub const IDM_PERM_NOTIF_BLOCK: usize = 4012;
pub const IDM_PERM_AUTOPLAY_ALLOW: usize = 4013;
pub const IDM_PERM_AUTOPLAY_ASK: usize = 4014;
pub const IDM_PERM_AUTOPLAY_BLOCK: usize = 4015;
pub const IDM_PERM_RESET_SITE: usize = 4016;

use crate::core::error::{NexoraError, Result};
use crate::engine::WebViewEngine;
use crate::navigation::Omnibox;
use crate::performance::SmartSuspensionEngine;
use crate::tabs::TabManager;
use crate::ui::chrome::{BrowserChrome, ChromeHitResult};
use crate::ui::layout::{BrowserLayout, TAB_BAR_HEIGHT};
use crate::ui::theme::ThemePalette;

pub const IDT_SUSPENSION_TIMER: usize = 1001;
pub const IDT_TAB_HOVER_PREVIEW: usize = 1002;
pub const IDT_PAGE_LOADER: usize = 1003;

const ZOOM_LEVELS: [f64; 11] = [0.50, 0.67, 0.75, 0.80, 0.90, 1.00, 1.10, 1.25, 1.50, 1.75, 2.00];

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
    pub omnibox_brush: HBRUSH,
    pub fullscreen: crate::media::FullscreenController,
    pub adblock_engine: Arc<crate::privacy::AdBlockEngine>,
    pub drag_start_x: i32,
    pub drag_start_y: i32,
    pub is_dragging_tab: bool,
    pub drag_tab_idx: Option<usize>,
    pub hovered_tab_id: Option<usize>,
    pub hover_timer_active: bool,
    pub streaming_engine_mode: crate::app::config::StreamingEngineMode,
}

impl WindowContext {
    pub fn current_webview_rect(&self) -> (i32, i32, i32, i32) {
        self.layout.webview_rect_for(self.fullscreen.is_fullscreen())
    }

    pub fn start_loader_timer(&mut self, hwnd: HWND) {
        if let Some(ref mut chrome) = self.chrome {
            chrome.start_loading();
        }
        unsafe {
            let _ = windows::Win32::UI::WindowsAndMessaging::SetTimer(
                hwnd,
                IDT_PAGE_LOADER,
                16,
                None,
            );
        }
    }

    pub fn stop_loader_timer(&mut self, hwnd: HWND) {
        if let Some(ref mut chrome) = self.chrome {
            chrome.finish_loading();
        }
        unsafe {
            let _ = windows::Win32::UI::WindowsAndMessaging::SetTimer(
                hwnd,
                IDT_PAGE_LOADER,
                16,
                None,
            );
        }
    }

    pub fn reset_loader(&mut self, hwnd: HWND) {
        if let Some(ref mut chrome) = self.chrome {
            chrome.reset_loader();
        }
        unsafe {
            let _ = windows::Win32::UI::WindowsAndMessaging::KillTimer(hwnd, IDT_PAGE_LOADER);
            let rect = RECT {
                left: 0,
                top: crate::ui::layout::TOP_CHROME_HEIGHT - 4,
                right: self.layout.window_width,
                bottom: crate::ui::layout::TOP_CHROME_HEIGHT,
            };
            let _ = windows::Win32::Graphics::Gdi::InvalidateRect(hwnd, Some(&rect), false);
        }
    }

    pub fn sync_tab_loader(&mut self, hwnd: HWND, tab_id: usize) {
        let is_loading = self.tab_manager.borrow().get_tab(tab_id).map(|t| t.is_loading).unwrap_or(false);
        if is_loading {
            self.start_loader_timer(hwnd);
        } else {
            self.reset_loader(hwnd);
        }
    }
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
        streaming_engine_mode: crate::app::config::StreamingEngineMode,
    ) -> Result<Self> {
        let is_closing = Arc::new(AtomicBool::new(false));

        unsafe {
            let hinstance: HINSTANCE = GetModuleHandleW(None)
                .map_err(|e| NexoraError::Win32(format!("Failed to get module handle: {}", e)))?
                .into();

            let class_name = w!("NexoraBrowserHostWindowClass");

            // Window background brush
            let bg_brush: HBRUSH = CreateSolidBrush(theme.window_bg);

            // Load application icon (Official Orbital Ribbon N)
            let mut hicon = windows::Win32::UI::WindowsAndMessaging::LoadIconW(
                hinstance,
                PCWSTR(1 as *const u16),
            )
            .unwrap_or_default();

            if hicon.is_invalid() {
                if let Ok(exe_path) = std::env::current_exe() {
                    let candidates = [
                        std::path::PathBuf::from("assets/branding/icon.ico"),
                        exe_path.parent().map(|p| p.join("assets/branding/icon.ico")).unwrap_or_default(),
                        exe_path.parent().and_then(|p| p.parent()).and_then(|p| p.parent()).map(|p| p.join("assets/branding/icon.ico")).unwrap_or_default(),
                    ];
                    for path in &candidates {
                        if path.exists() {
                            let path_wide: Vec<u16> = path.to_string_lossy().encode_utf16().chain(std::iter::once(0)).collect();
                            if let Ok(handle) = windows::Win32::UI::WindowsAndMessaging::LoadImageW(
                                None,
                                PCWSTR(path_wide.as_ptr()),
                                windows::Win32::UI::WindowsAndMessaging::IMAGE_ICON,
                                0,
                                0,
                                windows::Win32::UI::WindowsAndMessaging::LR_LOADFROMFILE | windows::Win32::UI::WindowsAndMessaging::LR_DEFAULTSIZE,
                            ) {
                                hicon = windows::Win32::UI::WindowsAndMessaging::HICON(handle.0);
                                break;
                            }
                        }
                    }
                }
            }

            let hicon_sm = if !hicon.is_invalid() {
                windows::Win32::UI::WindowsAndMessaging::CopyImage(
                    windows::Win32::Foundation::HANDLE(hicon.0),
                    windows::Win32::UI::WindowsAndMessaging::IMAGE_ICON,
                    windows::Win32::UI::WindowsAndMessaging::GetSystemMetrics(windows::Win32::UI::WindowsAndMessaging::SM_CXSMICON),
                    windows::Win32::UI::WindowsAndMessaging::GetSystemMetrics(windows::Win32::UI::WindowsAndMessaging::SM_CYSMICON),
                    windows::Win32::UI::WindowsAndMessaging::IMAGE_FLAGS(0),
                ).map(|h| windows::Win32::UI::WindowsAndMessaging::HICON(h.0)).unwrap_or(hicon)
            } else {
                hicon
            };

            let wnd_class = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                style: windows::Win32::UI::WindowsAndMessaging::WNDCLASS_STYLES(0),
                lpfnWndProc: Some(Self::wnd_proc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: hinstance,
                hIcon: hicon,
                hCursor: windows::Win32::UI::WindowsAndMessaging::LoadCursorW(
                    None,
                    windows::Win32::UI::WindowsAndMessaging::IDC_ARROW,
                )
                .unwrap_or_default(),
                hbrBackground: bg_brush,
                lpszMenuName: PCWSTR::null(),
                lpszClassName: class_name,
                hIconSm: hicon_sm,
            };

            let _ = RegisterClassExW(&wnd_class);

            let wide_title: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();

            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                class_name,
                PCWSTR(wide_title.as_ptr()),
                WS_OVERLAPPEDWINDOW | WS_CLIPCHILDREN,
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

            // Set window and taskbar icons (Big for Alt+Tab/Taskbar, Small for Titlebar)
            if !hicon.is_invalid() {
                let _ = SendMessageW(hwnd, WM_SETICON, WPARAM(ICON_BIG), LPARAM(hicon.0 as isize));
                let _ = SendMessageW(hwnd, WM_SETICON, WPARAM(ICON_SMALL), LPARAM(hicon_sm.0 as isize));
            }

            // Apply native dark title bar if theme is dark
            ThemePalette::apply_to_window(hwnd, theme.is_dark);

            // Extend frame into client area by 1px at the top so DWM handles frameless window shadows and transitions properly
            let margins = MARGINS {
                cxLeftWidth: 0,
                cxRightWidth: 0,
                cyTopHeight: 1,
                cyBottomHeight: 0,
            };
            let _ = DwmExtendFrameIntoClientArea(hwnd, &margins);

            // Recompute frame immediately so default Windows caption is completely stripped
            let _ = SetWindowPos(
                hwnd,
                None,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED,
            );

            let layout = BrowserLayout::new(width, height);
            let suspension_engine = SmartSuspensionEngine::new(suspension_timeout_minutes, smart_suspension);
            let omnibox_brush = CreateSolidBrush(theme.omnibox_bg);

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
                omnibox_brush,
                fullscreen: crate::media::FullscreenController::new(),
                adblock_engine: Arc::new(crate::privacy::AdBlockEngine::new(true, true)),
                drag_start_x: 0,
                drag_start_y: 0,
                is_dragging_tab: false,
                drag_tab_idx: None,
                hovered_tab_id: None,
                hover_timer_active: false,
                streaming_engine_mode,
            });

            SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(context) as isize);

            // Start 5-second periodic timer for memory pressure, working set trimming & smart tab suspension
            let _ = SetTimer(hwnd, IDT_SUSPENSION_TIMER, 5_000, None);

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
                        let mut dest_url = match target {
                            crate::navigation::NavigationTarget::Url(u) => u,
                            crate::navigation::NavigationTarget::Search { url, .. } => url,
                            crate::navigation::NavigationTarget::Internal(u) => u,
                        };

                        if let Some(intercepted) = crate::media::stream_engine::should_intercept_video_stream(&dest_url, (*ptr).streaming_engine_mode) {
                            dest_url = intercepted;
                        }

                        let active_id = (*ptr).tab_manager.borrow().active_tab_id().unwrap_or(1);
                        if !dest_url.starts_with("nexora://") {
                            (*ptr).start_loader_timer(hwnd_parent);
                        }
                        if let Some(ref webview) = (*ptr).webview {
                            let _ = webview.navigate_tab(active_id, &dest_url);
                        }

                        // Update active tab URL
                        let mut tabs = (*ptr).tab_manager.borrow_mut();
                        if let Some(active) = tabs.get_active_tab_mut() {
                            let clean_history_url = if dest_url.starts_with("nexora://player") {
                                if let Some(vid) = crate::media::stream_engine::extract_youtube_video_id(&dest_url) {
                                    format!("https://www.youtube.com/watch?v={}", vid)
                                } else {
                                    dest_url.clone()
                                }
                            } else {
                                dest_url.clone()
                            };
                            active.url = clean_history_url.clone();
                            active.push_history(&clean_history_url);
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
                let (x, y, w, h) = (*ptr).current_webview_rect();
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
            WM_NCCALCSIZE => {
                if wparam.0 != 0 {
                    // Stripping default OS caption, title bar, and OS minimize/maximize/close buttons.
                    return LRESULT(0);
                }
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
            WM_NCHITTEST => {
                if !ptr.is_null() && (*ptr).fullscreen.is_fullscreen() {
                    return LRESULT(HTCLIENT as isize);
                }
                let sx = (lparam.0 & 0xffff) as i16 as i32;
                let sy = ((lparam.0 >> 16) & 0xffff) as i16 as i32;

                let is_zoomed = IsZoomed(hwnd).as_bool();
                if !is_zoomed {
                    let mut rect = RECT::default();
                    if GetWindowRect(hwnd, &mut rect).is_ok() {
                        const BORDER: i32 = 6;
                        let on_left = sx < rect.left + BORDER;
                        let on_right = sx >= rect.right - BORDER;
                        let on_top = sy < rect.top + BORDER;
                        let on_bottom = sy >= rect.bottom - BORDER;

                        if on_top && on_left {
                            return LRESULT(HTTOPLEFT as isize);
                        }
                        if on_top && on_right {
                            return LRESULT(HTTOPRIGHT as isize);
                        }
                        if on_bottom && on_left {
                            return LRESULT(HTBOTTOMLEFT as isize);
                        }
                        if on_bottom && on_right {
                            return LRESULT(HTBOTTOMRIGHT as isize);
                        }
                        if on_top {
                            return LRESULT(HTTOP as isize);
                        }
                        if on_bottom {
                            return LRESULT(HTBOTTOM as isize);
                        }
                        if on_left {
                            return LRESULT(HTLEFT as isize);
                        }
                        if on_right {
                            return LRESULT(HTRIGHT as isize);
                        }
                    }
                }
                LRESULT(HTCLIENT as isize)
            }
            WM_PAINT => {
                let mut ps = PAINTSTRUCT::default();
                let hdc = BeginPaint(hwnd, &mut ps);
                if !ptr.is_null() {
                    let is_fs = (*ptr).fullscreen.is_fullscreen();
                    if !is_fs {
                        if let Some(ref chrome) = (*ptr).chrome {
                            let tabs = (*ptr).tab_manager.borrow();
                            let is_maximized = IsZoomed(hwnd).as_bool();
                            chrome.render(hdc, &(*ptr).layout, tabs.tabs(), tabs.active_tab_id(), is_maximized);
                        }
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

                        let is_fs = (*ptr).fullscreen.is_fullscreen();
                        let tab_count = (*ptr).tab_manager.borrow().tab_count();

                        if let Some(ref chrome) = (*ptr).chrome {
                            if is_fs {
                                chrome.set_omnibox_visible(false);
                            } else {
                                chrome.set_omnibox_visible(true);
                                chrome.layout_controls(&(*ptr).layout, tab_count);
                            }
                        }

                        if let Some(ref webview) = (*ptr).webview {
                            if is_fs {
                                let _ = webview.set_bounds(0, 0, width, height);
                            } else {
                                let (wx, wy, ww, wh) = (*ptr).layout.webview_rect();
                                let _ = webview.set_bounds(wx, wy, ww, wh);
                            }
                            webview.notify_parent_position_changed();
                        }

                        let _ = InvalidateRect(hwnd, None, false);
                    }
                }
                LRESULT(0)
            }
            WM_MOVE => {
                if !ptr.is_null() {
                    if let Some(ref webview) = (*ptr).webview {
                        webview.notify_parent_position_changed();
                    }
                }
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
            WM_MOUSEMOVE => {
                let mut tme = TRACKMOUSEEVENT {
                    cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
                    dwFlags: TME_LEAVE,
                    hwndTrack: hwnd,
                    dwHoverTime: 0,
                };
                let _ = TrackMouseEvent(&mut tme);

                if !ptr.is_null() {
                    if (*ptr).fullscreen.is_fullscreen() {
                        return LRESULT(0);
                    }
                    let x = (lparam.0 & 0xffff) as i16 as i32;
                    let y = ((lparam.0 >> 16) & 0xffff) as i16 as i32;

                    // Tab Drag and Drop handling
                    if let Some(from_idx) = (*ptr).drag_tab_idx {
                        let dx = (x - (*ptr).drag_start_x).abs();
                        if dx > 6 || (*ptr).is_dragging_tab {
                            (*ptr).is_dragging_tab = true;
                            let tabs = (*ptr).tab_manager.borrow();
                            let tab_rects = (*ptr).layout.calculate_tab_rects_for_tabs(tabs.tabs());
                            let pinned_count = tabs.pinned_tab_count();
                            let is_source_pinned = from_idx < pinned_count;

                            let mut insert_idx = from_idx;
                            for (i, (tx, _ty, tw, _th)) in tab_rects.iter().enumerate() {
                                let mid = tx + tw / 2;
                                if x < mid {
                                    insert_idx = i;
                                    break;
                                } else {
                                    insert_idx = i + 1;
                                }
                            }

                            if is_source_pinned {
                                if insert_idx > pinned_count {
                                    insert_idx = pinned_count;
                                }
                            } else {
                                if insert_idx < pinned_count {
                                    insert_idx = pinned_count;
                                }
                            }

                            if let Some(ref mut chrome) = (*ptr).chrome {
                                chrome.drag_source_idx = Some(from_idx);
                                chrome.drag_insert_idx = Some(insert_idx);
                            }
                            let _ = InvalidateRect(hwnd, None, false);
                            return LRESULT(0);
                        }
                    }

                    if let Some(ref mut chrome) = (*ptr).chrome {
                        let tabs = (*ptr).tab_manager.borrow();
                        let hit = chrome.hit_test(x, y, &(*ptr).layout, tabs.tabs());

                        // Tab Hover Preview timer
                        if let ChromeHitResult::Tab(tab_id) = hit {
                            if (*ptr).hovered_tab_id != Some(tab_id) {
                                (*ptr).hovered_tab_id = Some(tab_id);
                                let _ = SetTimer(hwnd, IDT_TAB_HOVER_PREVIEW, 450, None);
                                (*ptr).hover_timer_active = true;
                            }
                        } else if (*ptr).hover_timer_active {
                            let _ = KillTimer(hwnd, IDT_TAB_HOVER_PREVIEW);
                            (*ptr).hover_timer_active = false;
                            (*ptr).hovered_tab_id = None;
                        }

                        if chrome.update_hover(hit) {
                            let _ = InvalidateRect(hwnd, None, false);
                        }
                    }
                }
                LRESULT(0)
            }
            WM_MOUSELEAVE => {
                if !ptr.is_null() {
                    if (*ptr).hover_timer_active {
                        let _ = KillTimer(hwnd, IDT_TAB_HOVER_PREVIEW);
                        (*ptr).hover_timer_active = false;
                        (*ptr).hovered_tab_id = None;
                    }
                    if let Some(ref mut chrome) = (*ptr).chrome {
                        if chrome.update_hover(ChromeHitResult::None) {
                            let _ = InvalidateRect(hwnd, None, false);
                        }
                    }
                }
                LRESULT(0)
            }
            WM_LBUTTONDOWN => {
                if !ptr.is_null() {
                    if (*ptr).fullscreen.is_fullscreen() {
                        return LRESULT(0);
                    }
                    let x = (lparam.0 & 0xffff) as i16 as i32;
                    let y = ((lparam.0 >> 16) & 0xffff) as i16 as i32;

                    let hit = if let Some(ref chrome) = (*ptr).chrome {
                        let tabs = (*ptr).tab_manager.borrow();
                        chrome.hit_test(x, y, &(*ptr).layout, tabs.tabs())
                    } else {
                        ChromeHitResult::None
                    };

                    match hit {
                        ChromeHitResult::BrandLogo => {
                            let homepage = (*ptr).homepage.clone();
                            let title = if homepage == "nexora://home" || homepage == "nexora://newtab" {
                                "Nexora Home"
                            } else {
                                "New Tab"
                            };
                            let full_title = format!("{} - Nexora", title);
                            let wide_title: Vec<u16> = full_title.encode_utf16().chain(std::iter::once(0)).collect();
                            let _ = windows::Win32::UI::WindowsAndMessaging::SetWindowTextW(
                                hwnd,
                                windows::core::PCWSTR(wide_title.as_ptr()),
                            );

                            let mut tabs = (*ptr).tab_manager.borrow_mut();
                            if let Some(active) = tabs.get_active_tab_mut() {
                                active.url = homepage.clone();
                                active.title = title.to_string();
                            }
                            drop(tabs);

                            if let Some(ref mut chrome) = (*ptr).chrome {
                                chrome.set_omnibox_text(&homepage);
                                chrome.set_is_bookmarked(false);
                            }

                            if let Some(ref webview) = (*ptr).webview {
                                let _ = webview.navigate(&homepage);
                            }

                            let _ = InvalidateRect(hwnd, None, false);
                        }
                        ChromeHitResult::CaptionMinimize => {
                            let _ = ShowWindow(hwnd, SW_MINIMIZE);
                        }
                        ChromeHitResult::CaptionMaximize => {
                            let is_zoomed = IsZoomed(hwnd).as_bool();
                            if is_zoomed {
                                let _ = ShowWindow(hwnd, SW_RESTORE);
                            } else {
                                let _ = ShowWindow(hwnd, SW_MAXIMIZE);
                            }
                            let _ = InvalidateRect(hwnd, None, false);
                        }
                        ChromeHitResult::CaptionClose => {
                            let _ = SendMessageW(hwnd, WM_CLOSE, WPARAM(0), LPARAM(0));
                        }
                        ChromeHitResult::TitleBarDrag => {
                            let _ = ReleaseCapture();
                            let _ = SendMessageW(
                                hwnd,
                                WM_NCLBUTTONDOWN,
                                WPARAM(HTCAPTION as usize),
                                LPARAM(0),
                            );
                        }
                        ChromeHitResult::BtnBack => {
                            let mut tabs = (*ptr).tab_manager.borrow_mut();
                            if let Some(active) = tabs.get_active_tab_mut() {
                                let active_id = active.id;
                                if let Some(prev_url) = active.go_back() {
                                    drop(tabs);
                                    if !prev_url.starts_with("nexora://") {
                                        (*ptr).start_loader_timer(hwnd);
                                    }
                                    if let Some(ref webview) = (*ptr).webview {
                                        let _ = webview.navigate_tab(active_id, &prev_url);
                                    }
                                    if let Some(ref chrome) = (*ptr).chrome {
                                        chrome.set_omnibox_text(&prev_url);
                                    }
                                    let _ = InvalidateRect(hwnd, None, false);
                                }
                            }
                        }
                        ChromeHitResult::BtnForward => {
                            let mut tabs = (*ptr).tab_manager.borrow_mut();
                            if let Some(active) = tabs.get_active_tab_mut() {
                                let active_id = active.id;
                                if let Some(next_url) = active.go_forward() {
                                    drop(tabs);
                                    if !next_url.starts_with("nexora://") {
                                        (*ptr).start_loader_timer(hwnd);
                                    }
                                    if let Some(ref webview) = (*ptr).webview {
                                        let _ = webview.navigate_tab(active_id, &next_url);
                                    }
                                    if let Some(ref chrome) = (*ptr).chrome {
                                        chrome.set_omnibox_text(&next_url);
                                    }
                                    let _ = InvalidateRect(hwnd, None, false);
                                }
                            }
                        }
                        ChromeHitResult::BtnReload => {
                            let active_id = (*ptr).tab_manager.borrow().active_tab_id().unwrap_or(1);
                            (*ptr).start_loader_timer(hwnd);
                            if let Some(ref webview) = (*ptr).webview {
                                let _ = webview.reload_tab(active_id);
                            }
                        }
                        ChromeHitResult::BtnHome => {
                            let homepage = (*ptr).homepage.clone();
                            let title = if homepage == "nexora://home" || homepage == "nexora://newtab" {
                                "Nexora Home"
                            } else {
                                "Home"
                            };
                            let mut tabs = (*ptr).tab_manager.borrow_mut();
                            if let Some(active) = tabs.get_active_tab_mut() {
                                let active_id = active.id;
                                active.url = homepage.clone();
                                active.title = title.to_string();
                                active.push_history(&homepage);
                                drop(tabs);
                                if !homepage.starts_with("nexora://") {
                                    (*ptr).start_loader_timer(hwnd);
                                } else {
                                    (*ptr).reset_loader(hwnd);
                                }
                                if let Some(ref webview) = (*ptr).webview {
                                    let _ = webview.navigate_tab(active_id, &homepage);
                                }
                                if let Some(ref chrome) = (*ptr).chrome {
                                    chrome.set_omnibox_text(&homepage);
                                }
                                let _ = InvalidateRect(hwnd, None, false);
                            }
                        }
                        ChromeHitResult::BtnBookmark => {
                            let tabs = (*ptr).tab_manager.borrow();
                            let (current_url, current_title) = if let Some(active) = tabs.get_active_tab() {
                                (active.url.clone(), active.title.clone())
                            } else {
                                (String::new(), String::new())
                            };
                            drop(tabs);

                            if !current_url.is_empty() && !current_url.starts_with("nexora://") && current_url != "about:blank" {
                                if let Ok(paths) = crate::app::application::ApplicationPaths::resolve() {
                                    let db_path = paths.data_dir.join("nexora.db");
                                    if let Ok(conn) = crate::storage::SqliteConnection::open(&db_path) {
                                        let bm_mgr = crate::storage::BookmarkManager::new(std::sync::Arc::new(conn));
                                        let is_bm = bm_mgr.is_bookmarked(&current_url).unwrap_or(false);
                                        if is_bm {
                                            let _ = bm_mgr.delete_bookmark_by_url(&current_url);
                                            if let Some(ref mut chrome) = (*ptr).chrome {
                                                chrome.set_is_bookmarked(false);
                                            }
                                            log::info!("Toggled bookmark OFF: {}", current_url);
                                        } else {
                                            let _ = bm_mgr.add_bookmark(&current_title, &current_url, None);
                                            if let Some(ref mut chrome) = (*ptr).chrome {
                                                chrome.set_is_bookmarked(true);
                                            }
                                            log::info!("Toggled bookmark ON: {} ({})", current_title, current_url);
                                        }
                                        let _ = InvalidateRect(hwnd, None, false);
                                    }
                                }
                            }
                        }
                        ChromeHitResult::BtnMenu => {
                            let menu_rect = (*ptr).layout.menu_button_rect();
                            let mut pt = POINT {
                                x: menu_rect.0,
                                y: menu_rect.1 + menu_rect.3,
                            };
                            let _ = ClientToScreen(hwnd, &mut pt);

                            if let Ok(hmenu) = CreatePopupMenu() {
                                let _ = AppendMenuW(hmenu, MF_STRING, IDM_NEW_TAB, w!("New Tab\tCtrl+T"));
                                let _ = AppendMenuW(hmenu, MF_SEPARATOR, 0, PCWSTR::null());
                                let _ = AppendMenuW(hmenu, MF_STRING, IDM_BOOKMARKS, w!("Bookmarks\tCtrl+B"));
                                let _ = AppendMenuW(hmenu, MF_STRING, IDM_HISTORY, w!("History\tCtrl+H"));
                                let _ = AppendMenuW(hmenu, MF_SEPARATOR, 0, PCWSTR::null());
                                let _ = AppendMenuW(hmenu, MF_STRING, IDM_SETTINGS, w!("Settings\tCtrl+,"));
                                let _ = AppendMenuW(hmenu, MF_STRING, IDM_DIAGNOSTICS, w!("Diagnostics\tCtrl+Shift+D"));
                                let _ = AppendMenuW(hmenu, MF_SEPARATOR, 0, PCWSTR::null());
                                let _ = AppendMenuW(hmenu, MF_STRING, IDM_EXIT, w!("Exit\tAlt+F4"));

                                let cmd = TrackPopupMenuEx(
                                    hmenu,
                                    (TPM_RIGHTALIGN.0 | TPM_TOPALIGN.0 | TPM_RETURNCMD.0 | TPM_NONOTIFY.0) as u32,
                                    pt.x + menu_rect.2,
                                    pt.y,
                                    hwnd,
                                    None,
                                );
                                let _ = DestroyMenu(hmenu);

                                match cmd.0 as usize {
                                    IDM_NEW_TAB => {
                                        let homepage = (*ptr).homepage.clone();
                                        let title = if homepage == "nexora://home" || homepage == "nexora://newtab" {
                                            "Nexora Home"
                                        } else {
                                            "New Tab"
                                        };
                                        let mut tabs = (*ptr).tab_manager.borrow_mut();
                                        let _ = tabs.create_tab(&homepage, Some(title));
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
                                    IDM_BOOKMARKS => {
                                        if let Some(ref webview) = (*ptr).webview {
                                            let _ = webview.navigate("nexora://bookmarks");
                                        }
                                        if let Some(ref chrome) = (*ptr).chrome {
                                            chrome.set_omnibox_text("nexora://bookmarks");
                                        }
                                        let mut tabs = (*ptr).tab_manager.borrow_mut();
                                        if let Some(active) = tabs.get_active_tab_mut() {
                                            active.url = "nexora://bookmarks".to_string();
                                            active.title = "Bookmarks - Nexora".to_string();
                                        }
                                        drop(tabs);
                                        let wide_title: Vec<u16> = "Bookmarks - Nexora".encode_utf16().chain(std::iter::once(0)).collect();
                                        let _ = windows::Win32::UI::WindowsAndMessaging::SetWindowTextW(
                                            hwnd,
                                            windows::core::PCWSTR(wide_title.as_ptr()),
                                        );
                                        let _ = InvalidateRect(hwnd, None, false);
                                    }
                                    IDM_HISTORY => {
                                        if let Some(ref webview) = (*ptr).webview {
                                            let _ = webview.navigate("nexora://history");
                                        }
                                        if let Some(ref chrome) = (*ptr).chrome {
                                            chrome.set_omnibox_text("nexora://history");
                                        }
                                        let mut tabs = (*ptr).tab_manager.borrow_mut();
                                        if let Some(active) = tabs.get_active_tab_mut() {
                                            active.url = "nexora://history".to_string();
                                            active.title = "History - Nexora".to_string();
                                        }
                                        drop(tabs);
                                        let wide_title: Vec<u16> = "History - Nexora".encode_utf16().chain(std::iter::once(0)).collect();
                                        let _ = windows::Win32::UI::WindowsAndMessaging::SetWindowTextW(
                                            hwnd,
                                            windows::core::PCWSTR(wide_title.as_ptr()),
                                        );
                                        let _ = InvalidateRect(hwnd, None, false);
                                    }
                                    IDM_SETTINGS => {
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
                                        drop(tabs);
                                        let wide_title: Vec<u16> = "Settings - Nexora".encode_utf16().chain(std::iter::once(0)).collect();
                                        let _ = windows::Win32::UI::WindowsAndMessaging::SetWindowTextW(
                                            hwnd,
                                            windows::core::PCWSTR(wide_title.as_ptr()),
                                        );
                                        let _ = InvalidateRect(hwnd, None, false);
                                    }
                                    IDM_DIAGNOSTICS => {
                                        if let Some(ref webview) = (*ptr).webview {
                                            let _ = webview.navigate("nexora://diagnostics");
                                        }
                                        if let Some(ref chrome) = (*ptr).chrome {
                                            chrome.set_omnibox_text("nexora://diagnostics");
                                        }
                                        let mut tabs = (*ptr).tab_manager.borrow_mut();
                                        if let Some(active) = tabs.get_active_tab_mut() {
                                            active.url = "nexora://diagnostics".to_string();
                                            active.title = "Diagnostics - Nexora".to_string();
                                        }
                                        drop(tabs);
                                        let wide_title: Vec<u16> = "Diagnostics - Nexora".encode_utf16().chain(std::iter::once(0)).collect();
                                        let _ = windows::Win32::UI::WindowsAndMessaging::SetWindowTextW(
                                            hwnd,
                                            windows::core::PCWSTR(wide_title.as_ptr()),
                                        );
                                        let _ = InvalidateRect(hwnd, None, false);
                                    }
                                    IDM_EXIT => {
                                        let _ = SendMessageW(hwnd, WM_CLOSE, WPARAM(0), LPARAM(0));
                                    }
                                    _ => {}
                                }
                            }
                        }
                        ChromeHitResult::Tab(id) => {
                            let (x, y, w, h) = (*ptr).current_webview_rect();
                            let bounds = RECT { left: x, top: y, right: x + w, bottom: y + h };

                            // Setup drag start parameters
                            {
                                let tabs = (*ptr).tab_manager.borrow();
                                if let Some(idx) = tabs.tabs().iter().position(|t| t.id == id) {
                                    (*ptr).drag_start_x = x;
                                    (*ptr).drag_start_y = y;
                                    (*ptr).drag_tab_idx = Some(idx);
                                    (*ptr).is_dragging_tab = false;
                                    let _ = SetCapture(hwnd);
                                }
                            }

                            let mut tabs = (*ptr).tab_manager.borrow_mut();
                            let current_active = tabs.active_tab_id();
                            if current_active == Some(id) {
                                drop(tabs);
                                if let Some(ref webview) = (*ptr).webview {
                                    let _ = webview.switch_to_tab(id, bounds);
                                }
                                return LRESULT(0);
                            }
                            let was_suspended = tabs.get_tab(id).map(|t| t.is_suspended()).unwrap_or(false);
                            if tabs.switch_tab(id) {
                                if let Some(active) = tabs.get_active_tab() {
                                    let url = active.url.clone();
                                    let title = active.title.clone();
                                    let tab_zoom = active.zoom_factor;
                                    drop(tabs);
                                    if was_suspended {
                                        log::info!("Waking suspended/lazy tab #{} and navigating to {}", id, url);
                                    }

                                    // Update bookmark star state for switched tab
                                    let is_bm = if !url.starts_with("nexora://") && url != "about:blank" && !url.is_empty() {
                                        if let Ok(paths) = crate::app::application::ApplicationPaths::resolve() {
                                            let db_path = paths.data_dir.join("nexora.db");
                                            if let Ok(conn) = crate::storage::SqliteConnection::open(&db_path) {
                                                let bm_mgr = crate::storage::BookmarkManager::new(std::sync::Arc::new(conn));
                                                bm_mgr.is_bookmarked(&url).unwrap_or(false)
                                            } else { false }
                                        } else { false }
                                    } else { false };

                                    if let Some(ref mut chrome) = (*ptr).chrome {
                                        chrome.set_omnibox_text(&url);
                                        chrome.set_is_bookmarked(is_bm);
                                    }
                                    (*ptr).sync_tab_loader(hwnd, id);

                                    // Update window title immediately to this tab's own title
                                    let full_title = format!("{} - Nexora", title);
                                    let wide_title: Vec<u16> = full_title.encode_utf16().chain(std::iter::once(0)).collect();
                                    let _ = windows::Win32::UI::WindowsAndMessaging::SetWindowTextW(
                                        hwnd,
                                        windows::core::PCWSTR(wide_title.as_ptr()),
                                    );

                                    // Instant multi-controller visibility switch (< 15ms)
                                    let (x, y, w, h) = (*ptr).current_webview_rect();
                                    let bounds = RECT { left: x, top: y, right: x + w, bottom: y + h };
                                    let t0 = std::time::Instant::now();
                                    if let Some(ref webview) = (*ptr).webview {
                                        let _ = webview.set_tab_zoom(id, tab_zoom);
                                        let _ = webview.switch_to_tab(id, bounds);
                                    }
                                    let elapsed_ms = t0.elapsed().as_secs_f64() * 1000.0;
                                    log::info!("[Tab Switch] Switched to tab #{} in {:.2} ms (instant persistent view)", id, elapsed_ms);

                                    let _ = InvalidateRect(hwnd, None, false);
                                }
                            }
                        }
                        ChromeHitResult::TabAudio(id) => {
                            let mut tabs = (*ptr).tab_manager.borrow_mut();
                            let new_mute = tabs.toggle_tab_mute(id);
                            drop(tabs);

                            if let (Some(ref webview), Some(m)) = (&(*ptr).webview, new_mute) {
                                webview.set_tab_muted(id, m);
                            }
                            let _ = InvalidateRect(hwnd, None, false);
                        }
                        ChromeHitResult::BtnShield => {
                            Self::show_permissions_menu(hwnd, ptr);
                        }
                        ChromeHitResult::TabClose(id) => {
                            let mut tabs = (*ptr).tab_manager.borrow_mut();
                            let new_active = tabs.close_tab(id);
                            let maybe_tab_info = new_active.and_then(|new_id| tabs.get_tab(new_id).map(|t| (new_id, t.url.clone(), t.title.clone())));
                            drop(tabs);

                            let (x, y, w, h) = (*ptr).current_webview_rect();
                            let bounds = RECT { left: x, top: y, right: x + w, bottom: y + h };

                            if let Some(ref webview) = (*ptr).webview {
                                webview.close_tab_view(id);
                            }

                            if let Some((active_id, url, title)) = maybe_tab_info {
                                let is_bm = if !url.starts_with("nexora://") && url != "about:blank" && !url.is_empty() {
                                    if let Ok(paths) = crate::app::application::ApplicationPaths::resolve() {
                                        let db_path = paths.data_dir.join("nexora.db");
                                        if let Ok(conn) = crate::storage::SqliteConnection::open(&db_path) {
                                            let bm_mgr = crate::storage::BookmarkManager::new(std::sync::Arc::new(conn));
                                            bm_mgr.is_bookmarked(&url).unwrap_or(false)
                                        } else { false }
                                    } else { false }
                                } else { false };

                                if let Some(ref mut chrome) = (*ptr).chrome {
                                    chrome.set_omnibox_text(&url);
                                    chrome.set_is_bookmarked(is_bm);
                                }

                                let full_title = format!("{} - Nexora", title);
                                let wide_title: Vec<u16> = full_title.encode_utf16().chain(std::iter::once(0)).collect();
                                let _ = windows::Win32::UI::WindowsAndMessaging::SetWindowTextW(
                                    hwnd,
                                    windows::core::PCWSTR(wide_title.as_ptr()),
                                );

                                if let Some(ref webview) = (*ptr).webview {
                                    let _ = webview.switch_to_tab(active_id, bounds);
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
                            let title = if homepage == "nexora://home" || homepage == "nexora://newtab" {
                                "Nexora Home"
                            } else {
                                "New Tab"
                            };
                            let new_id = {
                                let mut tabs = (*ptr).tab_manager.borrow_mut();
                                let id = tabs.create_tab(&homepage, Some(title));
                                tabs.switch_tab(id);
                                id
                            };

                            let full_title = format!("{} - Nexora", title);
                            let wide_title: Vec<u16> = full_title.encode_utf16().chain(std::iter::once(0)).collect();
                            let _ = windows::Win32::UI::WindowsAndMessaging::SetWindowTextW(
                                hwnd,
                                windows::core::PCWSTR(wide_title.as_ptr()),
                            );

                            let (x, y, w, h) = (*ptr).current_webview_rect();
                            let bounds = RECT { left: x, top: y, right: x + w, bottom: y + h };

                            if let Some(ref mut chrome) = (*ptr).chrome {
                                chrome.set_omnibox_text(&homepage);
                                chrome.set_is_bookmarked(false);
                                let count = (*ptr).tab_manager.borrow().tab_count();
                                chrome.layout_controls(&(*ptr).layout, count);
                            }
                            (*ptr).reset_loader(hwnd);

                            if let Some(ref webview) = (*ptr).webview {
                                let _ = webview.create_tab_view(new_id, &homepage, bounds, true);
                                let _ = webview.switch_to_tab(new_id, bounds);
                            }

                            let _ = InvalidateRect(hwnd, None, false);
                        }
                        ChromeHitResult::None => {}
                    }
                }
                LRESULT(0)
            }
            WM_LBUTTONUP => {
                if !ptr.is_null() {
                    let _ = ReleaseCapture();
                    if (*ptr).is_dragging_tab {
                        let from_opt = (*ptr).drag_tab_idx;
                        let to_opt = (*ptr).chrome.as_ref().and_then(|c| c.drag_insert_idx);

                        if let (Some(from), Some(to)) = (from_opt, to_opt) {
                            let mut tabs = (*ptr).tab_manager.borrow_mut();
                            tabs.move_tab(from, to);
                            drop(tabs);
                        }

                        (*ptr).is_dragging_tab = false;
                        (*ptr).drag_tab_idx = None;
                        if let Some(ref mut chrome) = (*ptr).chrome {
                            chrome.drag_source_idx = None;
                            chrome.drag_insert_idx = None;
                        }
                        let _ = InvalidateRect(hwnd, None, false);
                    } else {
                        (*ptr).drag_tab_idx = None;
                    }
                }
                LRESULT(0)
            }
            WM_RBUTTONUP => {
                if !ptr.is_null() {
                    if (*ptr).fullscreen.is_fullscreen() {
                        return LRESULT(0);
                    }
                    let x = (lparam.0 & 0xffff) as i16 as i32;
                    let y = ((lparam.0 >> 16) & 0xffff) as i16 as i32;
                    if y <= TAB_BAR_HEIGHT {
                        let hit = if let Some(ref chrome) = (*ptr).chrome {
                            let tabs = (*ptr).tab_manager.borrow();
                            chrome.hit_test(x, y, &(*ptr).layout, tabs.tabs())
                        } else {
                            ChromeHitResult::None
                        };

                        if let ChromeHitResult::Tab(target_tab_id) = hit {
                            Self::show_tab_context_menu(hwnd, ptr, target_tab_id, x, y);
                        }
                    }
                }
                LRESULT(0)
            }
            WM_MOUSEWHEEL => {
                if !ptr.is_null() {
                    let ctrl_pressed = (GetKeyState(VK_CONTROL.0 as i32) as u16 & 0x8000) != 0;
                    if ctrl_pressed {
                        let delta = ((wparam.0 >> 16) & 0xffff) as i16;
                        if delta > 0 {
                            Self::apply_zoom_step(hwnd, ptr, true);
                        } else if delta < 0 {
                            Self::apply_zoom_step(hwnd, ptr, false);
                        }
                        return LRESULT(0);
                    }
                }
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
            WM_LBUTTONDBLCLK => {
                if !ptr.is_null() {
                    if (*ptr).fullscreen.is_fullscreen() {
                        return LRESULT(0);
                    }
                    let x = (lparam.0 & 0xffff) as i16 as i32;
                    let y = ((lparam.0 >> 16) & 0xffff) as i16 as i32;

                    if y <= TAB_BAR_HEIGHT {
                        let hit = if let Some(ref chrome) = (*ptr).chrome {
                            let tabs = (*ptr).tab_manager.borrow();
                            chrome.hit_test(x, y, &(*ptr).layout, tabs.tabs())
                        } else {
                            ChromeHitResult::None
                        };

                        if hit == ChromeHitResult::TitleBarDrag || hit == ChromeHitResult::None {
                            let is_zoomed = IsZoomed(hwnd).as_bool();
                            if is_zoomed {
                                let _ = ShowWindow(hwnd, SW_RESTORE);
                            } else {
                                let _ = ShowWindow(hwnd, SW_MAXIMIZE);
                            }
                            let _ = InvalidateRect(hwnd, None, false);
                        }
                    }
                }
                LRESULT(0)
            }
            WM_KEYDOWN => {
                if !ptr.is_null() {
                    let ctrl_pressed = (GetKeyState(VK_CONTROL.0 as i32) as u16 & 0x8000) != 0;
                    let shift_pressed = (GetKeyState(VK_SHIFT.0 as i32) as u16 & 0x8000) != 0;
                    let alt_pressed = (GetKeyState(VK_MENU.0 as i32) as u16 & 0x8000) != 0;
                    let vk = wparam.0 as u16;

                    // 1. F11 Fullscreen Toggle
                    if vk == VK_F11.0 {
                        let is_fs = (*ptr).fullscreen.is_fullscreen();
                        if is_fs {
                            let (_, _, w, h) = (*ptr).fullscreen.exit_fullscreen(hwnd);
                            (*ptr).layout = BrowserLayout::new(w, h);
                            if let Some(ref chrome) = (*ptr).chrome {
                                chrome.set_omnibox_visible(true);
                                let count = (*ptr).tab_manager.borrow().tab_count();
                                chrome.layout_controls(&(*ptr).layout, count);
                            }
                            if let Some(ref webview) = (*ptr).webview {
                                let (wx, wy, ww, wh) = (*ptr).layout.webview_rect();
                                let _ = webview.set_bounds(wx, wy, ww, wh);
                                webview.notify_parent_position_changed();
                            }
                        } else {
                            if let Some(ref chrome) = (*ptr).chrome {
                                chrome.set_omnibox_visible(false);
                            }
                            let (_, _, w, h) = (*ptr).fullscreen.enter_fullscreen(hwnd);
                            (*ptr).layout = BrowserLayout::new(w, h);
                            if let Some(ref chrome) = (*ptr).chrome {
                                chrome.set_omnibox_visible(false);
                            }
                            if let Some(ref webview) = (*ptr).webview {
                                let _ = webview.set_bounds(0, 0, w, h);
                                webview.notify_parent_position_changed();
                            }
                        }
                        let _ = InvalidateRect(hwnd, None, false);
                        return LRESULT(0);
                    }

                    // 2. Escape: Exit Fullscreen
                    if vk == VK_ESCAPE.0 && (*ptr).fullscreen.is_fullscreen() {
                        let (_, _, w, h) = (*ptr).fullscreen.exit_fullscreen(hwnd);
                        (*ptr).layout = BrowserLayout::new(w, h);
                        if let Some(ref chrome) = (*ptr).chrome {
                            chrome.set_omnibox_visible(true);
                            let count = (*ptr).tab_manager.borrow().tab_count();
                            chrome.layout_controls(&(*ptr).layout, count);
                        }
                        if let Some(ref webview) = (*ptr).webview {
                            let (wx, wy, ww, wh) = (*ptr).layout.webview_rect();
                            let _ = webview.set_bounds(wx, wy, ww, wh);
                            webview.notify_parent_position_changed();
                        }
                        let _ = InvalidateRect(hwnd, None, false);
                        return LRESULT(0);
                    }

                    // 3. F5 Reload
                    if vk == VK_F5.0 {
                        if let Some(ref webview) = (*ptr).webview {
                            let _ = webview.reload();
                        }
                        return LRESULT(0);
                    }

                    // 4. Alt shortcuts (Back, Forward, Focus Omnibox)
                    if alt_pressed {
                        if vk == VK_LEFT.0 {
                            let mut tabs = (*ptr).tab_manager.borrow_mut();
                            if let Some(active) = tabs.get_active_tab_mut() {
                                if let Some(prev_url) = active.go_back() {
                                    drop(tabs);
                                    if let Some(ref webview) = (*ptr).webview {
                                        let _ = webview.navigate(&prev_url);
                                    }
                                    if let Some(ref chrome) = (*ptr).chrome {
                                        chrome.set_omnibox_text(&prev_url);
                                    }
                                    let _ = InvalidateRect(hwnd, None, false);
                                }
                            }
                            return LRESULT(0);
                        }
                        if vk == VK_RIGHT.0 {
                            let mut tabs = (*ptr).tab_manager.borrow_mut();
                            if let Some(active) = tabs.get_active_tab_mut() {
                                if let Some(next_url) = active.go_forward() {
                                    drop(tabs);
                                    if let Some(ref webview) = (*ptr).webview {
                                        let _ = webview.navigate(&next_url);
                                    }
                                    if let Some(ref chrome) = (*ptr).chrome {
                                        chrome.set_omnibox_text(&next_url);
                                    }
                                    let _ = InvalidateRect(hwnd, None, false);
                                }
                            }
                            return LRESULT(0);
                        }
                        if vk == b'D' as u16 || vk == b'd' as u16 {
                            if let Some(ref chrome) = (*ptr).chrome {
                                let _ = SetFocus(chrome.hwnd_omnibox);
                                let _ = SendMessageW(chrome.hwnd_omnibox, EM_SETSEL, WPARAM(0), LPARAM(-1));
                            }
                            return LRESULT(0);
                        }
                    }

                    // 5. Ctrl shortcuts
                    if ctrl_pressed {
                        if vk == VK_TAB.0 {
                            let mut tabs = (*ptr).tab_manager.borrow_mut();
                            let active_id = tabs.active_tab_id().unwrap_or(0);
                            let all_ids: Vec<usize> = tabs.tabs().iter().map(|t| t.id).collect();
                            if !all_ids.is_empty() {
                                let cur_idx = all_ids.iter().position(|&id| id == active_id).unwrap_or(0);
                                let next_idx = if shift_pressed {
                                    if cur_idx == 0 { all_ids.len() - 1 } else { cur_idx - 1 }
                                } else {
                                    (cur_idx + 1) % all_ids.len()
                                };
                                let target_id = all_ids[next_idx];
                                if tabs.switch_tab(target_id) {
                                    if let Some(active) = tabs.get_active_tab() {
                                        let url = active.url.clone();
                                        let title = active.title.clone();
                                        drop(tabs);

                                        let full_title = format!("{} - Nexora", title);
                                        let wide_title: Vec<u16> = full_title.encode_utf16().chain(std::iter::once(0)).collect();
                                        let _ = windows::Win32::UI::WindowsAndMessaging::SetWindowTextW(
                                            hwnd,
                                            windows::core::PCWSTR(wide_title.as_ptr()),
                                        );

                                        let (x, y, w, h) = (*ptr).current_webview_rect();
                                        let bounds = RECT { left: x, top: y, right: x + w, bottom: y + h };
                                        if let Some(ref webview) = (*ptr).webview {
                                            let _ = webview.switch_to_tab(target_id, bounds);
                                        }
                                        if let Some(ref chrome) = (*ptr).chrome {
                                            chrome.set_omnibox_text(&url);
                                        }
                                        let _ = InvalidateRect(hwnd, None, false);
                                    }
                                }
                            }
                            return LRESULT(0);
                        }

                        match vk as u8 as char {
                            'H' | 'h' => {
                                if let Some(ref webview) = (*ptr).webview {
                                    let _ = webview.navigate("nexora://history");
                                }
                                if let Some(ref chrome) = (*ptr).chrome {
                                    chrome.set_omnibox_text("nexora://history");
                                }
                                let mut tabs = (*ptr).tab_manager.borrow_mut();
                                if let Some(active) = tabs.get_active_tab_mut() {
                                    active.url = "nexora://history".to_string();
                                    active.title = "History - Nexora".to_string();
                                }
                                let _ = InvalidateRect(hwnd, None, false);
                                return LRESULT(0);
                            }
                            'B' | 'b' => {
                                if let Some(ref webview) = (*ptr).webview {
                                    let _ = webview.navigate("nexora://bookmarks");
                                }
                                if let Some(ref chrome) = (*ptr).chrome {
                                    chrome.set_omnibox_text("nexora://bookmarks");
                                }
                                let mut tabs = (*ptr).tab_manager.borrow_mut();
                                if let Some(active) = tabs.get_active_tab_mut() {
                                    active.url = "nexora://bookmarks".to_string();
                                    active.title = "Bookmarks - Nexora".to_string();
                                }
                                let _ = InvalidateRect(hwnd, None, false);
                                return LRESULT(0);
                            }
                            'D' | 'd' => {
                                let tabs = (*ptr).tab_manager.borrow();
                                let (current_url, current_title) = if let Some(active) = tabs.get_active_tab() {
                                    (active.url.clone(), active.title.clone())
                                } else {
                                    (String::new(), String::new())
                                };
                                drop(tabs);

                                if !current_url.is_empty() && !current_url.starts_with("nexora://") && current_url != "about:blank" {
                                    if let Ok(paths) = crate::app::application::ApplicationPaths::resolve() {
                                        let db_path = paths.data_dir.join("nexora.db");
                                        if let Ok(conn) = crate::storage::SqliteConnection::open(&db_path) {
                                            let bm_mgr = crate::storage::BookmarkManager::new(std::sync::Arc::new(conn));
                                            let is_bm = bm_mgr.is_bookmarked(&current_url).unwrap_or(false);
                                            if is_bm {
                                                let _ = bm_mgr.delete_bookmark_by_url(&current_url);
                                                if let Some(ref mut chrome) = (*ptr).chrome {
                                                    chrome.set_is_bookmarked(false);
                                                }
                                                log::info!("Ctrl+D toggled bookmark OFF: {}", current_url);
                                            } else {
                                                let _ = bm_mgr.add_bookmark(&current_title, &current_url, None);
                                                if let Some(ref mut chrome) = (*ptr).chrome {
                                                    chrome.set_is_bookmarked(true);
                                                }
                                                log::info!("Ctrl+D toggled bookmark ON: {} ({})", current_title, current_url);
                                            }
                                            let _ = InvalidateRect(hwnd, None, false);
                                        }
                                    }
                                }
                                return LRESULT(0);
                            }
                            'T' | 't' => {
                                let homepage = (*ptr).homepage.clone();
                                let title = if homepage == "nexora://home" || homepage == "nexora://newtab" {
                                    "Nexora Home"
                                } else {
                                    "New Tab"
                                };
                                let new_id = {
                                    let mut tabs = (*ptr).tab_manager.borrow_mut();
                                    let id = tabs.create_tab(&homepage, Some(title));
                                    tabs.switch_tab(id);
                                    id
                                };

                                let (x, y, w, h) = (*ptr).current_webview_rect();
                                let bounds = RECT { left: x, top: y, right: x + w, bottom: y + h };

                                if let Some(ref webview) = (*ptr).webview {
                                    let _ = webview.create_tab_view(new_id, &homepage, bounds, true);
                                    let _ = webview.switch_to_tab(new_id, bounds);
                                }
                                if let Some(ref chrome) = (*ptr).chrome {
                                    chrome.set_omnibox_text(&homepage);
                                    let count = (*ptr).tab_manager.borrow().tab_count();
                                    chrome.layout_controls(&(*ptr).layout, count);
                                    let _ = SetFocus(chrome.hwnd_omnibox);
                                    let _ = SendMessageW(chrome.hwnd_omnibox, EM_SETSEL, WPARAM(0), LPARAM(-1));
                                }
                                let _ = InvalidateRect(hwnd, None, false);
                                return LRESULT(0);
                            }
                            'W' | 'w' => {
                                let (maybe_info, closed_id) = {
                                    let mut tabs = (*ptr).tab_manager.borrow_mut();
                                    if let Some(active_id) = tabs.active_tab_id() {
                                        let new_active = tabs.close_tab(active_id);
                                        let info = new_active.and_then(|new_id| tabs.get_tab(new_id).map(|t| (new_id, t.url.clone(), t.title.clone())));
                                        (info, Some(active_id))
                                    } else {
                                        (None, None)
                                    }
                                };

                                if let Some(id) = closed_id {
                                    if let Some(ref webview) = (*ptr).webview {
                                        webview.close_tab_view(id);
                                    }
                                }

                                if let Some((new_id, url, title)) = maybe_info {
                                    let (x, y, w, h) = (*ptr).current_webview_rect();
                                    let bounds = RECT { left: x, top: y, right: x + w, bottom: y + h };
                                    if let Some(ref webview) = (*ptr).webview {
                                        let _ = webview.switch_to_tab(new_id, bounds);
                                    }
                                    if let Some(ref chrome) = (*ptr).chrome {
                                        chrome.set_omnibox_text(&url);
                                    }
                                    let full_title = format!("{} - Nexora", title);
                                    let wide_title: Vec<u16> = full_title.encode_utf16().chain(std::iter::once(0)).collect();
                                    let _ = windows::Win32::UI::WindowsAndMessaging::SetWindowTextW(
                                        hwnd,
                                        windows::core::PCWSTR(wide_title.as_ptr()),
                                    );
                                } else if closed_id.is_some() {
                                    let _ = SendMessageW(hwnd, WM_CLOSE, WPARAM(0), LPARAM(0));
                                }

                                if let Some(ref chrome) = (*ptr).chrome {
                                    let count = (*ptr).tab_manager.borrow().tab_count();
                                    chrome.layout_controls(&(*ptr).layout, count);
                                }
                                let _ = InvalidateRect(hwnd, None, false);
                                return LRESULT(0);
                            }
                            'R' | 'r' => {
                                if let Some(ref webview) = (*ptr).webview {
                                    let _ = webview.reload();
                                }
                                return LRESULT(0);
                            }
                            'L' | 'l' => {
                                if let Some(ref chrome) = (*ptr).chrome {
                                    let _ = SetFocus(chrome.hwnd_omnibox);
                                    let _ = SendMessageW(chrome.hwnd_omnibox, EM_SETSEL, WPARAM(0), LPARAM(-1));
                                }
                                return LRESULT(0);
                            }
                            'P' | 'p' => {
                                if shift_pressed {
                                    if let Some(ref webview) = (*ptr).webview {
                                        let _ = webview.trigger_pip();
                                    }
                                    return LRESULT(0);
                                }
                            }
                            'F' | 'f' => {
                                if let Some(ref webview) = (*ptr).webview {
                                    let _ = webview.execute_script(crate::navigation::FindInPage::script_open());
                                }
                                return LRESULT(0);
                            }
                            'G' | 'g' => {
                                if let Some(ref webview) = (*ptr).webview {
                                    let backwards = shift_pressed;
                                    let script = format!("try {{ window.find('', false, {}, true, false, false, false); }} catch(e) {{}}", backwards);
                                    let _ = webview.execute_script(&script);
                                }
                                return LRESULT(0);
                            }
                            _ => {}
                        }

                        if vk == VK_ADD.0 || vk == VK_OEM_PLUS || vk == b'=' as u16 {
                            Self::apply_zoom_step(hwnd, ptr, true);
                            return LRESULT(0);
                        }
                        if vk == VK_SUBTRACT.0 || vk == VK_OEM_MINUS || vk == b'-' as u16 {
                            Self::apply_zoom_step(hwnd, ptr, false);
                            return LRESULT(0);
                        }
                        if vk == 0x30 || vk == 0x60 {
                            Self::reset_zoom(hwnd, ptr);
                            return LRESULT(0);
                        }
                    }
                }
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
            WM_CTLCOLOREDIT | WM_CTLCOLORSTATIC => {
                if !ptr.is_null() {
                    let hdc = HDC(wparam.0 as *mut core::ffi::c_void);
                    let theme = (*ptr).theme;
                    let _ = SetTextColor(hdc, theme.omnibox_text);
                    let _ = SetBkColor(hdc, theme.omnibox_bg);
                    return LRESULT((*ptr).omnibox_brush.0 as isize);
                }
                DefWindowProcW(hwnd, msg, wparam, lparam)
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
                    drop(tabs);

                    // Periodic Working Set Trimming for Nexora and child WebView2 processes
                    let _ = crate::performance::memory::trim_working_set();
                } else if !ptr.is_null() && wparam.0 == IDT_TAB_HOVER_PREVIEW {
                    let _ = KillTimer(hwnd, IDT_TAB_HOVER_PREVIEW);
                    (*ptr).hover_timer_active = false;
                    if let Some(hovered_id) = (*ptr).hovered_tab_id {
                        let tabs = (*ptr).tab_manager.borrow();
                        if let Some(t) = tabs.get_tab(hovered_id) {
                            log::debug!("[Tab Preview Card] Hovered Tab #{}: '{}' ({})", t.id, t.title, t.url);
                        }
                    }
                } else if !ptr.is_null() && wparam.0 == IDT_PAGE_LOADER {
                    let mut should_continue = false;
                    if let Some(ref mut chrome) = (*ptr).chrome {
                        should_continue = chrome.step_loader_animation();
                    }
                    if !should_continue {
                        let _ = KillTimer(hwnd, IDT_PAGE_LOADER);
                    }
                    let rect = RECT {
                        left: 0,
                        top: crate::ui::layout::TOP_CHROME_HEIGHT - 4,
                        right: (*ptr).layout.window_width,
                        bottom: crate::ui::layout::TOP_CHROME_HEIGHT,
                    };
                    let _ = InvalidateRect(hwnd, Some(&rect), false);
                }
                LRESULT(0)
            }
            WM_CLOSE => {
                let _ = KillTimer(hwnd, IDT_SUSPENSION_TIMER);
                let _ = KillTimer(hwnd, IDT_TAB_HOVER_PREVIEW);
                let _ = KillTimer(hwnd, IDT_PAGE_LOADER);
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
                let _ = KillTimer(hwnd, IDT_TAB_HOVER_PREVIEW);
                let _ = KillTimer(hwnd, IDT_PAGE_LOADER);
                if !ptr.is_null() {
                    let ctx = Box::from_raw(ptr);
                    let _ = DeleteObject(ctx.omnibox_brush);
                    SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
                }
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }

    fn apply_zoom_step(_hwnd: HWND, ptr: *mut WindowContext, zoom_in: bool) {
        unsafe {
            if ptr.is_null() { return; }
            let mut tabs = (*ptr).tab_manager.borrow_mut();
            if let Some(active) = tabs.get_active_tab_mut() {
                let current_zoom = active.zoom_factor;
                let active_id = active.id;
                let active_url = active.url.clone();

                let mut new_zoom = current_zoom;
                if zoom_in {
                    for &level in &ZOOM_LEVELS {
                        if level > current_zoom + 0.01 {
                            new_zoom = level;
                            break;
                        }
                    }
                    if (new_zoom - current_zoom).abs() < 0.001 {
                        new_zoom = *ZOOM_LEVELS.last().unwrap();
                    }
                } else {
                    for &level in ZOOM_LEVELS.iter().rev() {
                        if level < current_zoom - 0.01 {
                            new_zoom = level;
                            break;
                        }
                    }
                    if (new_zoom - current_zoom).abs() < 0.001 {
                        new_zoom = *ZOOM_LEVELS.first().unwrap();
                    }
                }

                active.set_zoom(new_zoom);
                drop(tabs);

                if let Some(ref webview) = (*ptr).webview {
                    let _ = webview.set_tab_zoom(active_id, new_zoom);
                }

                // Persist zoom for origin
                if !active_url.starts_with("nexora://") && !active_url.is_empty() {
                    let origin = crate::storage::ZoomManager::extract_origin(&active_url);
                    if let Ok(paths) = crate::app::application::ApplicationPaths::resolve() {
                        let db_path = paths.data_dir.join("nexora.db");
                        if let Ok(conn) = crate::storage::SqliteConnection::open(&db_path) {
                            let zm = crate::storage::ZoomManager::new(std::sync::Arc::new(conn));
                            let _ = zm.set_zoom(&origin, new_zoom);
                            log::info!("[Per-Site Zoom] Persisted zoom {:.0}% for origin {}", new_zoom * 100.0, origin);
                        }
                    }
                }
            }
        }
    }

    fn reset_zoom(_hwnd: HWND, ptr: *mut WindowContext) {
        unsafe {
            if ptr.is_null() { return; }
            let mut tabs = (*ptr).tab_manager.borrow_mut();
            if let Some(active) = tabs.get_active_tab_mut() {
                let active_id = active.id;
                let active_url = active.url.clone();
                active.set_zoom(1.0);
                drop(tabs);

                if let Some(ref webview) = (*ptr).webview {
                    let _ = webview.set_tab_zoom(active_id, 1.0);
                }

                if !active_url.starts_with("nexora://") && !active_url.is_empty() {
                    let origin = crate::storage::ZoomManager::extract_origin(&active_url);
                    if let Ok(paths) = crate::app::application::ApplicationPaths::resolve() {
                        let db_path = paths.data_dir.join("nexora.db");
                        if let Ok(conn) = crate::storage::SqliteConnection::open(&db_path) {
                            let zm = crate::storage::ZoomManager::new(std::sync::Arc::new(conn));
                            let _ = zm.remove_zoom(&origin);
                            log::info!("[Per-Site Zoom] Reset zoom to 100% for origin {}", origin);
                        }
                    }
                }
            }
        }
    }

    unsafe fn show_tab_context_menu(hwnd: HWND, ptr: *mut WindowContext, tab_id: usize, x: i32, y: i32) {
        if ptr.is_null() { return; }
        let mut pt = POINT { x, y };
        let _ = ClientToScreen(hwnd, &mut pt);

        let (is_pinned, is_muted) = {
            let tabs = (*ptr).tab_manager.borrow();
            if let Some(t) = tabs.get_tab(tab_id) {
                (t.is_pinned, t.is_muted)
            } else {
                return;
            }
        };

        if let Ok(hmenu) = CreatePopupMenu() {
            let _ = AppendMenuW(hmenu, MF_STRING, IDM_NEW_TAB, w!("New Tab\tCtrl+T"));
            let _ = AppendMenuW(hmenu, MF_STRING, IDM_TAB_DUPLICATE, w!("Duplicate Tab"));
            let pin_text = if is_pinned { w!("Unpin Tab") } else { w!("Pin Tab") };
            let _ = AppendMenuW(hmenu, MF_STRING, if is_pinned { IDM_TAB_UNPIN } else { IDM_TAB_PIN }, pin_text);
            let _ = AppendMenuW(hmenu, MF_SEPARATOR, 0, PCWSTR::null());

            let mute_text = if is_muted { w!("Unmute Tab") } else { w!("Mute Tab") };
            let _ = AppendMenuW(hmenu, MF_STRING, if is_muted { IDM_TAB_UNMUTE } else { IDM_TAB_MUTE }, mute_text);
            let _ = AppendMenuW(hmenu, MF_STRING, IDM_TAB_MUTE_OTHERS, w!("Mute Other Tabs"));
            let _ = AppendMenuW(hmenu, MF_STRING, IDM_TAB_UNMUTE_ALL, w!("Unmute All Tabs"));
            let _ = AppendMenuW(hmenu, MF_SEPARATOR, 0, PCWSTR::null());

            let _ = AppendMenuW(hmenu, MF_STRING, IDM_TAB_CLOSE_TAB, w!("Close Tab\tCtrl+W"));
            let _ = AppendMenuW(hmenu, MF_STRING, IDM_TAB_CLOSE_OTHERS, w!("Close Other Tabs"));
            let _ = AppendMenuW(hmenu, MF_STRING, IDM_TAB_CLOSE_RIGHT, w!("Close Tabs to the Right"));

            let cmd = TrackPopupMenuEx(
                hmenu,
                (TPM_RIGHTALIGN.0 | TPM_TOPALIGN.0 | TPM_RETURNCMD.0 | TPM_NONOTIFY.0) as u32,
                pt.x,
                pt.y,
                hwnd,
                None,
            );
            let _ = DestroyMenu(hmenu);

            match cmd.0 as usize {
                IDM_NEW_TAB => {
                    let homepage = (*ptr).homepage.clone();
                    let title = if homepage == "nexora://home" || homepage == "nexora://newtab" {
                        "Nexora Home"
                    } else {
                        "New Tab"
                    };
                    let new_id = {
                        let mut tabs = (*ptr).tab_manager.borrow_mut();
                        let id = tabs.create_tab(&homepage, Some(title));
                        tabs.switch_tab(id);
                        id
                    };
                    let (x, y, w, h) = (*ptr).current_webview_rect();
                    let bounds = RECT { left: x, top: y, right: x + w, bottom: y + h };
                    if let Some(ref webview) = (*ptr).webview {
                        let _ = webview.create_tab_view(new_id, &homepage, bounds, true);
                        let _ = webview.switch_to_tab(new_id, bounds);
                    }
                    if let Some(ref chrome) = (*ptr).chrome {
                        chrome.set_omnibox_text(&homepage);
                        let count = (*ptr).tab_manager.borrow().tab_count();
                        chrome.layout_controls(&(*ptr).layout, count);
                    }
                    let _ = InvalidateRect(hwnd, None, false);
                }
                IDM_TAB_DUPLICATE => {
                    let mut tabs = (*ptr).tab_manager.borrow_mut();
                    if let Some(new_id) = tabs.duplicate_tab(tab_id) {
                        let dup_url = tabs.get_tab(new_id).map(|t| t.url.clone()).unwrap_or_default();
                        let dup_title = tabs.get_tab(new_id).map(|t| t.title.clone()).unwrap_or_default();
                        let dup_zoom = tabs.get_tab(new_id).map(|t| t.zoom_factor).unwrap_or(1.0);
                        tabs.switch_tab(new_id);
                        drop(tabs);

                        let (x, y, w, h) = (*ptr).current_webview_rect();
                        let bounds = RECT { left: x, top: y, right: x + w, bottom: y + h };
                        if let Some(ref webview) = (*ptr).webview {
                            let _ = webview.create_tab_view(new_id, &dup_url, bounds, true);
                            let _ = webview.set_tab_zoom(new_id, dup_zoom);
                            let _ = webview.switch_to_tab(new_id, bounds);
                        }
                        if let Some(ref chrome) = (*ptr).chrome {
                            chrome.set_omnibox_text(&dup_url);
                            let count = (*ptr).tab_manager.borrow().tab_count();
                            chrome.layout_controls(&(*ptr).layout, count);
                        }
                        let full_title = format!("{} - Nexora", dup_title);
                        let wide_title: Vec<u16> = full_title.encode_utf16().chain(std::iter::once(0)).collect();
                        let _ = windows::Win32::UI::WindowsAndMessaging::SetWindowTextW(
                            hwnd,
                            windows::core::PCWSTR(wide_title.as_ptr()),
                        );
                        let _ = InvalidateRect(hwnd, None, false);
                    }
                }
                IDM_TAB_PIN => {
                    let mut tabs = (*ptr).tab_manager.borrow_mut();
                    tabs.pin_tab(tab_id);
                    drop(tabs);
                    let _ = InvalidateRect(hwnd, None, false);
                }
                IDM_TAB_UNPIN => {
                    let mut tabs = (*ptr).tab_manager.borrow_mut();
                    tabs.unpin_tab(tab_id);
                    drop(tabs);
                    let _ = InvalidateRect(hwnd, None, false);
                }
                IDM_TAB_MUTE => {
                    let mut tabs = (*ptr).tab_manager.borrow_mut();
                    tabs.set_tab_muted(tab_id, true);
                    drop(tabs);
                    if let Some(ref webview) = (*ptr).webview {
                        webview.set_tab_muted(tab_id, true);
                    }
                    let _ = InvalidateRect(hwnd, None, false);
                }
                IDM_TAB_UNMUTE => {
                    let mut tabs = (*ptr).tab_manager.borrow_mut();
                    tabs.set_tab_muted(tab_id, false);
                    drop(tabs);
                    if let Some(ref webview) = (*ptr).webview {
                        webview.set_tab_muted(tab_id, false);
                    }
                    let _ = InvalidateRect(hwnd, None, false);
                }
                IDM_TAB_MUTE_OTHERS => {
                    let mut tabs = (*ptr).tab_manager.borrow_mut();
                    tabs.mute_other_tabs(tab_id);
                    let other_ids: Vec<usize> = tabs.tabs().iter().filter(|t| t.id != tab_id).map(|t| t.id).collect();
                    drop(tabs);
                    if let Some(ref webview) = (*ptr).webview {
                        for oid in other_ids {
                            webview.set_tab_muted(oid, true);
                        }
                    }
                    let _ = InvalidateRect(hwnd, None, false);
                }
                IDM_TAB_UNMUTE_ALL => {
                    let mut tabs = (*ptr).tab_manager.borrow_mut();
                    tabs.unmute_all_tabs();
                    let all_ids: Vec<usize> = tabs.tabs().iter().map(|t| t.id).collect();
                    drop(tabs);
                    if let Some(ref webview) = (*ptr).webview {
                        for aid in all_ids {
                            webview.set_tab_muted(aid, false);
                        }
                    }
                    let _ = InvalidateRect(hwnd, None, false);
                }
                IDM_TAB_CLOSE_TAB => {
                    let mut tabs = (*ptr).tab_manager.borrow_mut();
                    let new_active = tabs.close_tab(tab_id);
                    let maybe_tab_info = new_active.and_then(|new_id| tabs.get_tab(new_id).map(|t| (new_id, t.url.clone(), t.title.clone())));
                    drop(tabs);

                    let (x, y, w, h) = (*ptr).current_webview_rect();
                    let bounds = RECT { left: x, top: y, right: x + w, bottom: y + h };

                    if let Some(ref webview) = (*ptr).webview {
                        webview.close_tab_view(tab_id);
                    }

                    if let Some((active_id, url, title)) = maybe_tab_info {
                        if let Some(ref mut chrome) = (*ptr).chrome {
                            chrome.set_omnibox_text(&url);
                        }
                        let full_title = format!("{} - Nexora", title);
                        let wide_title: Vec<u16> = full_title.encode_utf16().chain(std::iter::once(0)).collect();
                        let _ = windows::Win32::UI::WindowsAndMessaging::SetWindowTextW(
                            hwnd,
                            windows::core::PCWSTR(wide_title.as_ptr()),
                        );
                        if let Some(ref webview) = (*ptr).webview {
                            let _ = webview.switch_to_tab(active_id, bounds);
                        }
                    }
                    let _ = InvalidateRect(hwnd, None, false);
                }
                IDM_TAB_CLOSE_OTHERS => {
                    let mut tabs = (*ptr).tab_manager.borrow_mut();
                    let closed = tabs.close_other_tabs(tab_id);
                    tabs.switch_tab(tab_id);
                    drop(tabs);

                    if let Some(ref webview) = (*ptr).webview {
                        for cid in closed {
                            webview.close_tab_view(cid);
                        }
                        let (x, y, w, h) = (*ptr).current_webview_rect();
                        let bounds = RECT { left: x, top: y, right: x + w, bottom: y + h };
                        let _ = webview.switch_to_tab(tab_id, bounds);
                    }
                    let _ = InvalidateRect(hwnd, None, false);
                }
                IDM_TAB_CLOSE_RIGHT => {
                    let mut tabs = (*ptr).tab_manager.borrow_mut();
                    let closed = tabs.close_tabs_to_right(tab_id);
                    drop(tabs);

                    if let Some(ref webview) = (*ptr).webview {
                        for cid in closed {
                            webview.close_tab_view(cid);
                        }
                    }
                    let _ = InvalidateRect(hwnd, None, false);
                }
                _ => {}
            }
        }
    }

    unsafe fn show_permissions_menu(hwnd: HWND, ptr: *mut WindowContext) {
        if ptr.is_null() { return; }
        let omni = (*ptr).layout.omnibox_rect();
        let mut pt = POINT { x: omni.0 + 8, y: omni.1 + omni.3 };
        let _ = ClientToScreen(hwnd, &mut pt);

        let active_url = {
            let tabs = (*ptr).tab_manager.borrow();
            tabs.get_active_tab().map(|t| t.url.clone()).unwrap_or_default()
        };

        if active_url.starts_with("nexora://") || active_url.is_empty() {
            if let Ok(hmenu) = CreatePopupMenu() {
                let _ = AppendMenuW(hmenu, MF_STRING, 0, w!("Nexora System Page - Full Trusted Permissions"));
                let _ = TrackPopupMenuEx(hmenu, (TPM_RIGHTALIGN.0 | TPM_TOPALIGN.0 | TPM_RETURNCMD.0) as u32, pt.x, pt.y, hwnd, None);
                let _ = DestroyMenu(hmenu);
            }
            return;
        }

        let origin = crate::privacy::PermissionsManager::normalize_origin(&active_url);

        let paths = match crate::app::application::ApplicationPaths::resolve() {
            Ok(p) => p,
            Err(_) => return,
        };
        let db_path = paths.data_dir.join("nexora.db");
        let conn = match crate::storage::SqliteConnection::open(&db_path) {
            Ok(c) => std::sync::Arc::new(c),
            Err(_) => return,
        };
        let perm_mgr = crate::privacy::PermissionsManager::new(conn);

        let cam_state = perm_mgr.get_permission(&origin, crate::privacy::PermissionKind::Camera).unwrap_or(crate::privacy::PermissionState::Prompt);
        let mic_state = perm_mgr.get_permission(&origin, crate::privacy::PermissionKind::Microphone).unwrap_or(crate::privacy::PermissionState::Prompt);
        let geo_state = perm_mgr.get_permission(&origin, crate::privacy::PermissionKind::Geolocation).unwrap_or(crate::privacy::PermissionState::Prompt);
        let notif_state = perm_mgr.get_permission(&origin, crate::privacy::PermissionKind::Notifications).unwrap_or(crate::privacy::PermissionState::Block);
        let auto_state = perm_mgr.get_permission(&origin, crate::privacy::PermissionKind::Autoplay).unwrap_or(crate::privacy::PermissionState::Allow);

        if let Ok(hmenu) = CreatePopupMenu() {
            let title_info = format!("Permissions for {}", origin);
            let wide_info: Vec<u16> = title_info.encode_utf16().chain(std::iter::once(0)).collect();
            let _ = AppendMenuW(hmenu, MF_STRING, 0, PCWSTR(wide_info.as_ptr()));
            let _ = AppendMenuW(hmenu, MF_STRING, 0, w!("Connection: Secure (TLS / HTTPS)"));
            let _ = AppendMenuW(hmenu, MF_SEPARATOR, 0, PCWSTR::null());

            let build_sub = |allow_id, ask_id, block_id, cur_state| {
                if let Ok(sub) = CreatePopupMenu() {
                    let a_flag = if cur_state == crate::privacy::PermissionState::Allow { MF_CHECKED.0 } else { MF_UNCHECKED.0 };
                    let p_flag = if cur_state == crate::privacy::PermissionState::Prompt { MF_CHECKED.0 } else { MF_UNCHECKED.0 };
                    let b_flag = if cur_state == crate::privacy::PermissionState::Block { MF_CHECKED.0 } else { MF_UNCHECKED.0 };
                    let _ = AppendMenuW(sub, MENU_ITEM_FLAGS(MF_STRING.0 | a_flag), allow_id, w!("Allow"));
                    let _ = AppendMenuW(sub, MENU_ITEM_FLAGS(MF_STRING.0 | p_flag), ask_id, w!("Ask (Default)"));
                    let _ = AppendMenuW(sub, MENU_ITEM_FLAGS(MF_STRING.0 | b_flag), block_id, w!("Block"));
                    sub
                } else {
                    windows::Win32::UI::WindowsAndMessaging::HMENU::default()
                }
            };

            let cam_sub = build_sub(IDM_PERM_CAMERA_ALLOW, IDM_PERM_CAMERA_ASK, IDM_PERM_CAMERA_BLOCK, cam_state);
            let mic_sub = build_sub(IDM_PERM_MIC_ALLOW, IDM_PERM_MIC_ASK, IDM_PERM_MIC_BLOCK, mic_state);
            let geo_sub = build_sub(IDM_PERM_GEO_ALLOW, IDM_PERM_GEO_ASK, IDM_PERM_GEO_BLOCK, geo_state);
            let notif_sub = build_sub(IDM_PERM_NOTIF_ALLOW, IDM_PERM_NOTIF_ASK, IDM_PERM_NOTIF_BLOCK, notif_state);
            let auto_sub = build_sub(IDM_PERM_AUTOPLAY_ALLOW, IDM_PERM_AUTOPLAY_ASK, IDM_PERM_AUTOPLAY_BLOCK, auto_state);

            let _ = AppendMenuW(hmenu, MENU_ITEM_FLAGS(MF_POPUP.0 | MF_STRING.0), cam_sub.0 as usize, w!("Camera"));
            let _ = AppendMenuW(hmenu, MENU_ITEM_FLAGS(MF_POPUP.0 | MF_STRING.0), mic_sub.0 as usize, w!("Microphone"));
            let _ = AppendMenuW(hmenu, MENU_ITEM_FLAGS(MF_POPUP.0 | MF_STRING.0), geo_sub.0 as usize, w!("Location"));
            let _ = AppendMenuW(hmenu, MENU_ITEM_FLAGS(MF_POPUP.0 | MF_STRING.0), notif_sub.0 as usize, w!("Notifications"));
            let _ = AppendMenuW(hmenu, MENU_ITEM_FLAGS(MF_POPUP.0 | MF_STRING.0), auto_sub.0 as usize, w!("Autoplay"));

            let _ = AppendMenuW(hmenu, MF_SEPARATOR, 0, PCWSTR::null());
            let _ = AppendMenuW(hmenu, MF_STRING, IDM_PERM_RESET_SITE, w!("Reset Permissions for This Site"));

            let cmd = TrackPopupMenuEx(
                hmenu,
                (TPM_RIGHTALIGN.0 | TPM_TOPALIGN.0 | TPM_RETURNCMD.0 | TPM_NONOTIFY.0) as u32,
                pt.x,
                pt.y,
                hwnd,
                None,
            );
            let _ = DestroyMenu(hmenu);

            match cmd.0 as usize {
                IDM_PERM_CAMERA_ALLOW => { let _ = perm_mgr.set_permission(&origin, crate::privacy::PermissionKind::Camera, crate::privacy::PermissionState::Allow); }
                IDM_PERM_CAMERA_ASK => { let _ = perm_mgr.set_permission(&origin, crate::privacy::PermissionKind::Camera, crate::privacy::PermissionState::Prompt); }
                IDM_PERM_CAMERA_BLOCK => { let _ = perm_mgr.set_permission(&origin, crate::privacy::PermissionKind::Camera, crate::privacy::PermissionState::Block); }
                IDM_PERM_MIC_ALLOW => { let _ = perm_mgr.set_permission(&origin, crate::privacy::PermissionKind::Microphone, crate::privacy::PermissionState::Allow); }
                IDM_PERM_MIC_ASK => { let _ = perm_mgr.set_permission(&origin, crate::privacy::PermissionKind::Microphone, crate::privacy::PermissionState::Prompt); }
                IDM_PERM_MIC_BLOCK => { let _ = perm_mgr.set_permission(&origin, crate::privacy::PermissionKind::Microphone, crate::privacy::PermissionState::Block); }
                IDM_PERM_GEO_ALLOW => { let _ = perm_mgr.set_permission(&origin, crate::privacy::PermissionKind::Geolocation, crate::privacy::PermissionState::Allow); }
                IDM_PERM_GEO_ASK => { let _ = perm_mgr.set_permission(&origin, crate::privacy::PermissionKind::Geolocation, crate::privacy::PermissionState::Prompt); }
                IDM_PERM_GEO_BLOCK => { let _ = perm_mgr.set_permission(&origin, crate::privacy::PermissionKind::Geolocation, crate::privacy::PermissionState::Block); }
                IDM_PERM_NOTIF_ALLOW => { let _ = perm_mgr.set_permission(&origin, crate::privacy::PermissionKind::Notifications, crate::privacy::PermissionState::Allow); }
                IDM_PERM_NOTIF_ASK => { let _ = perm_mgr.set_permission(&origin, crate::privacy::PermissionKind::Notifications, crate::privacy::PermissionState::Prompt); }
                IDM_PERM_NOTIF_BLOCK => { let _ = perm_mgr.set_permission(&origin, crate::privacy::PermissionKind::Notifications, crate::privacy::PermissionState::Block); }
                IDM_PERM_AUTOPLAY_ALLOW => { let _ = perm_mgr.set_permission(&origin, crate::privacy::PermissionKind::Autoplay, crate::privacy::PermissionState::Allow); }
                IDM_PERM_AUTOPLAY_ASK => { let _ = perm_mgr.set_permission(&origin, crate::privacy::PermissionKind::Autoplay, crate::privacy::PermissionState::Prompt); }
                IDM_PERM_AUTOPLAY_BLOCK => { let _ = perm_mgr.set_permission(&origin, crate::privacy::PermissionKind::Autoplay, crate::privacy::PermissionState::Block); }
                IDM_PERM_RESET_SITE => {
                    let _ = perm_mgr.clear_all_for_origin(&origin);
                }
                _ => {}
            }
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
            crate::app::config::StreamingEngineMode::Standard,
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
