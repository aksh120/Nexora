use std::path::Path;
use std::sync::mpsc::channel;
use windows::core::{Interface, PCWSTR, PWSTR};
use windows::Win32::Foundation::{BOOL, HWND, RECT};
use windows::Win32::System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED};
use windows::Win32::System::WinRT::EventRegistrationToken;
use webview2_com::Microsoft::Web::WebView2::Win32::*;
use webview2_com::*;

use crate::core::error::{NexoraError, Result};

pub struct WebViewEngine {
    _environment: ICoreWebView2Environment,
    controller: ICoreWebView2Controller,
    webview: ICoreWebView2,
}

impl WebViewEngine {
    pub fn new(hwnd: HWND, user_data_dir: &Path, initial_url: &str) -> Result<Self> {
        unsafe {
            // Initialize COM for UI apartment thread
            let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        }

        std::fs::create_dir_all(user_data_dir)?;
        let udata_str = user_data_dir.to_string_lossy().to_string();
        let udata_wide = pwstr_from_str(&udata_str);

        // Hardware video acceleration and low-overhead flags via GPU capabilities
        let gpu_caps = crate::media::GpuCapabilities::detect(true);
        let mut browser_args_vec = gpu_caps.build_browser_switches();
        browser_args_vec.push("--disk-cache-size=16777216"); // 16 MB disk cache cap
        browser_args_vec.push("--media-cache-size=8388608"); // 8 MB media cache cap
        let browser_args = browser_args_vec.join(" ");

        log::info!("Configuring Microsoft WebView2 Environment:");
        log::info!("  UserDataFolder: {}", udata_str);
        log::info!("  Arguments:      {}", browser_args);

        let options = CoreWebView2EnvironmentOptions::default();
        unsafe {
            options.set_additional_browser_arguments(browser_args);
        }
        let options_iface: ICoreWebView2EnvironmentOptions = options.into();

        // 1. Create WebView2 Environment
        let (tx_env, rx_env) = channel();
        let env_handler = CreateCoreWebView2EnvironmentCompletedHandler::create(Box::new(move |res, env| {
            let _ = tx_env.send((res, env));
            Ok(())
        }));

        unsafe {
            CreateCoreWebView2EnvironmentWithOptions(
                None,
                PCWSTR(udata_wide.0),
                &options_iface,
                &env_handler,
            )
            .map_err(|e| NexoraError::Engine(format!("Failed to invoke CreateCoreWebView2EnvironmentWithOptions: {}", e)))?;
        }

        let (res_env, env_opt) = wait_with_pump(rx_env)
            .map_err(|e| NexoraError::Engine(format!("Environment pump failed: {}", e)))?;
        res_env.map_err(|e| NexoraError::Engine(format!("Environment creation returned error: {}", e)))?;
        let environment = env_opt.ok_or_else(|| NexoraError::Engine("Null WebView2 Environment returned".into()))?;

        log::info!("WebView2 Environment created successfully.");

        // 2. Create WebView2 Controller attached to host HWND
        let (tx_ctl, rx_ctl) = channel();
        let ctl_handler = CreateCoreWebView2ControllerCompletedHandler::create(Box::new(move |res, ctl| {
            let _ = tx_ctl.send((res, ctl));
            Ok(())
        }));

        unsafe {
            environment.CreateCoreWebView2Controller(hwnd, &ctl_handler)
                .map_err(|e| NexoraError::Engine(format!("Failed to invoke CreateCoreWebView2Controller: {}", e)))?;
        }

        let (res_ctl, ctl_opt) = wait_with_pump(rx_ctl)
            .map_err(|e| NexoraError::Engine(format!("Controller pump failed: {}", e)))?;
        res_ctl.map_err(|e| NexoraError::Engine(format!("Controller creation returned error: {}", e)))?;
        let controller = ctl_opt.ok_or_else(|| NexoraError::Engine("Null WebView2 Controller returned".into()))?;

        log::info!("WebView2 Controller instantiated and attached to HWND {:?}.", hwnd.0);

        let webview = unsafe {
            controller.CoreWebView2()
                .map_err(|e| NexoraError::Engine(format!("Failed to get CoreWebView2 from controller: {}", e)))?
        };

        // 3. Configure initial visibility and settings
        unsafe {
            let _ = controller.SetIsVisible(true);
            if let Ok(settings) = webview.Settings() {
                let _ = settings.SetIsScriptEnabled(true);
                let _ = settings.SetIsWebMessageEnabled(true);
                let _ = settings.SetAreDefaultScriptDialogsEnabled(true);
                let _ = settings.SetIsStatusBarEnabled(false);
                let _ = settings.SetAreDevToolsEnabled(cfg!(debug_assertions));
            }
        }

        let engine = Self {
            _environment: environment,
            controller,
            webview,
        };

        // 4. Hook Core Events
        engine.setup_event_handlers(hwnd)?;

        // 5. Navigate to initial URL
        engine.navigate(initial_url)?;

        Ok(engine)
    }

    pub fn set_bounds(&self, x: i32, y: i32, width: i32, height: i32) -> Result<()> {
        let rect = RECT {
            left: x,
            top: y,
            right: x + width,
            bottom: y + height,
        };
        unsafe {
            self.controller.SetBounds(rect)
                .map_err(|e| NexoraError::Engine(format!("Failed to set webview bounds: {}", e)))?;
        }
        Ok(())
    }

    pub fn navigate(&self, target_url: &str) -> Result<()> {
        if target_url == "nexora://settings" {
            let app_data_dir = crate::app::application::ApplicationPaths::resolve()
                .map(|p| p.app_data_dir)
                .unwrap_or_else(|_| std::env::temp_dir());
            let config = crate::app::config::NexoraConfig::load_or_default(&app_data_dir);
            let html = crate::settings::SettingsUi::render_html(&config);
            return self.navigate_to_string(&html);
        } else if target_url == "nexora://diagnostics" {
            let snapshot = crate::diagnostics::DiagnosticsSnapshot::capture(
                1,
                "Nexora Diagnostics",
                "nexora://diagnostics",
                0,
                Default::default(),
                false,
            );
            let html = crate::diagnostics::DiagnosticsUi::render_html(&snapshot);
            return self.navigate_to_string(&html);
        } else if target_url == "nexora://bookmarks" {
            let app_data_dir = crate::app::application::ApplicationPaths::resolve()
                .map(|p| p.data_dir)
                .unwrap_or_else(|_| std::env::temp_dir());
            let db_path = app_data_dir.join("nexora.db");
            let bookmarks = if let Ok(conn) = crate::storage::SqliteConnection::open(&db_path) {
                let bm_mgr = crate::storage::BookmarkManager::new(std::sync::Arc::new(conn));
                bm_mgr.get_all().unwrap_or_default()
            } else {
                Vec::new()
            };
            let html = crate::storage::BookmarksUi::render_html(&bookmarks);
            return self.navigate_to_string(&html);
        } else if target_url == "nexora://history" {
            let app_data_dir = crate::app::application::ApplicationPaths::resolve()
                .map(|p| p.data_dir)
                .unwrap_or_else(|_| std::env::temp_dir());
            let db_path = app_data_dir.join("nexora.db");
            let history = if let Ok(conn) = crate::storage::SqliteConnection::open(&db_path) {
                let hist_mgr = crate::storage::HistoryManager::new(std::sync::Arc::new(conn));
                hist_mgr.get_recent(200).unwrap_or_default()
            } else {
                Vec::new()
            };
            let html = crate::storage::HistoryUi::render_html(&history);
            return self.navigate_to_string(&html);
        }

        let normalized = if target_url.starts_with("http://")
            || target_url.starts_with("https://")
            || target_url.starts_with("edge://")
            || target_url.starts_with("about:")
            || target_url.starts_with("data:")
        {
            target_url.to_string()
        } else {
            format!("https://{}", target_url)
        };

        log::info!("Navigating to: {}", normalized);
        let wide = pwstr_from_str(&normalized);
        unsafe {
            self.webview.Navigate(PCWSTR(wide.0))
                .map_err(|e| NexoraError::Navigation(format!("Navigation failed for {}: {}", normalized, e)))?;
        }
        Ok(())
    }

    pub fn navigate_to_string(&self, html_content: &str) -> Result<()> {
        log::info!("Rendering internal page via NavigateToString ({} bytes)", html_content.len());
        let wide = pwstr_from_str(html_content);
        unsafe {
            self.webview.NavigateToString(PCWSTR(wide.0))
                .map_err(|e| NexoraError::Navigation(format!("NavigateToString failed: {}", e)))?;
        }
        Ok(())
    }

    pub fn go_back(&self) -> Result<()> {
        unsafe {
            let mut can_go_back = BOOL(0);
            let _ = self.webview.CanGoBack(&mut can_go_back);
            if can_go_back.as_bool() {
                self.webview.GoBack()
                    .map_err(|e| NexoraError::Engine(format!("GoBack failed: {}", e)))?;
            }
        }
        Ok(())
    }

    pub fn go_forward(&self) -> Result<()> {
        unsafe {
            let mut can_go_forward = BOOL(0);
            let _ = self.webview.CanGoForward(&mut can_go_forward);
            if can_go_forward.as_bool() {
                self.webview.GoForward()
                    .map_err(|e| NexoraError::Engine(format!("GoForward failed: {}", e)))?;
            }
        }
        Ok(())
    }

    pub fn reload(&self) -> Result<()> {
        unsafe {
            self.webview.Reload()
                .map_err(|e| NexoraError::Engine(format!("Reload failed: {}", e)))?;
        }
        Ok(())
    }

    pub fn stop(&self) -> Result<()> {
        unsafe {
            self.webview.Stop()
                .map_err(|e| NexoraError::Engine(format!("Stop failed: {}", e)))?;
        }
        Ok(())
    }

    pub fn get_source(&self) -> Result<String> {
        unsafe {
            let mut uri = PWSTR::null();
            self.webview.Source(&mut uri)
                .map_err(|e| NexoraError::Engine(format!("Failed to get source URL: {}", e)))?;
            Ok(string_from_pcwstr(&PCWSTR(uri.0)))
        }
    }

    pub fn get_title(&self) -> Result<String> {
        unsafe {
            let mut title = PWSTR::null();
            self.webview.DocumentTitle(&mut title)
                .map_err(|e| NexoraError::Engine(format!("Failed to get document title: {}", e)))?;
            Ok(string_from_pcwstr(&PCWSTR(title.0)))
        }
    }

    pub fn execute_script(&self, script: &str) -> Result<()> {
        let wide = pwstr_from_str(script);
        unsafe {
            self.webview.ExecuteScript(PCWSTR(wide.0), None)
                .map_err(|e| NexoraError::Engine(format!("ExecuteScript failed: {}", e)))?;
        }
        Ok(())
    }

    pub fn trigger_pip(&self) -> Result<()> {
        let script = crate::media::PipWindow::generate_pip_script();
        self.execute_script(script)
    }

    fn setup_event_handlers(&self, hwnd: HWND) -> Result<()> {
        let hwnd_parent = hwnd;

        // 1. NavigationStarting (Immediately show newly navigated URL in omnibox)
        let starting_handler = NavigationStartingEventHandler::create(Box::new(move |_sender, args| {
            if let Some(args) = args {
                unsafe {
                    let mut uri = PWSTR::null();
                    if args.Uri(&mut uri).is_ok() {
                        let url_str = string_from_pcwstr(&PCWSTR(uri.0));
                        log::info!("[WebView2] Navigation Starting: {}", url_str);

                        // Strict Guard: Never overwrite omnibox or tab state with about:blank or data: URIs
                        if url_str.is_empty() || url_str == "about:blank" || url_str.starts_with("data:") {
                            return Ok(());
                        }

                        let ptr = windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW(
                            hwnd_parent,
                            windows::Win32::UI::WindowsAndMessaging::GWLP_USERDATA,
                        ) as *mut crate::browser::host_window::WindowContext;
                        if !ptr.is_null() {
                            let mut tabs = (*ptr).tab_manager.borrow_mut();
                            if let Some(active) = tabs.get_active_tab_mut() {
                                // Guard: Never allow an external navigation to overwrite an internal page
                                if !active.url.starts_with("nexora://") {
                                    active.url = url_str.clone();
                                    active.push_history(&url_str);
                                    if let Some(ref chrome) = (*ptr).chrome {
                                        chrome.set_omnibox_text(&url_str);
                                    }
                                }
                            }
                            drop(tabs);
                            let _ = windows::Win32::Graphics::Gdi::InvalidateRect(hwnd_parent, None, false);
                        }
                    }
                }
            }
            Ok(())
        }));
        let mut starting_token = EventRegistrationToken::default();
        unsafe {
            let _ = self.webview.add_NavigationStarting(&starting_handler, &mut starting_token);
        }

        // 2. SourceChanged (Guaranteed sync on redirects, SPAs, and deep link navigations)
        let source_handler = SourceChangedEventHandler::create(Box::new(move |sender, _args| {
            if let Some(sender) = sender {
                unsafe {
                    let mut uri = PWSTR::null();
                    if sender.Source(&mut uri).is_ok() {
                        let url_str = string_from_pcwstr(&PCWSTR(uri.0));
                        log::info!("[WebView2] Source Changed: {}", url_str);

                        // Strict Guard: Never overwrite omnibox or tab state with about:blank or data: URIs
                        if url_str.is_empty() || url_str == "about:blank" || url_str.starts_with("data:") {
                            return Ok(());
                        }

                        let ptr = windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW(
                            hwnd_parent,
                            windows::Win32::UI::WindowsAndMessaging::GWLP_USERDATA,
                        ) as *mut crate::browser::host_window::WindowContext;
                        if !ptr.is_null() {
                            let mut tabs = (*ptr).tab_manager.borrow_mut();
                            if let Some(active) = tabs.get_active_tab_mut() {
                                // Guard: Never allow an external source change to overwrite an internal page
                                if !active.url.starts_with("nexora://") {
                                    active.url = url_str.clone();
                                    active.push_history(&url_str);
                                    if let Some(ref chrome) = (*ptr).chrome {
                                        chrome.set_omnibox_text(&url_str);
                                    }
                                }
                            }
                            drop(tabs);
                            let _ = windows::Win32::Graphics::Gdi::InvalidateRect(hwnd_parent, None, false);
                        }
                    }
                }
            }
            Ok(())
        }));
        let mut source_token = EventRegistrationToken::default();
        unsafe {
            let _ = self.webview.add_SourceChanged(&source_handler, &mut source_token);
        }

        // 3. DocumentTitleChanged (Keep tab header and window title synced with webpage title)
        let title_handler = DocumentTitleChangedEventHandler::create(Box::new(move |sender, _args| {
            if let Some(sender) = sender {
                unsafe {
                    let mut title = PWSTR::null();
                    if sender.DocumentTitle(&mut title).is_ok() {
                        let title_str = string_from_pcwstr(&PCWSTR(title.0));
                        log::info!("[WebView2] Document Title Changed: {}", title_str);

                        // Guard: Do not overwrite titles with blank, about:blank, or data:
                        if title_str.trim().is_empty() || title_str == "about:blank" || title_str.starts_with("data:") {
                            return Ok(());
                        }

                        let ptr = windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW(
                            hwnd_parent,
                            windows::Win32::UI::WindowsAndMessaging::GWLP_USERDATA,
                        ) as *mut crate::browser::host_window::WindowContext;
                        if !ptr.is_null() {
                            let mut tabs = (*ptr).tab_manager.borrow_mut();
                            if let Some(active) = tabs.get_active_tab_mut() {
                                // Guard: Never allow an external page title to overwrite an internal page
                                if !active.url.starts_with("nexora://") {
                                    active.title = title_str.clone();
                                    let full_title = format!("{} - Nexora", title_str);
                                    let wide_title: Vec<u16> = full_title.encode_utf16().chain(std::iter::once(0)).collect();
                                    let _ = windows::Win32::UI::WindowsAndMessaging::SetWindowTextW(
                                        hwnd_parent,
                                        windows::core::PCWSTR(wide_title.as_ptr()),
                                    );
                                }
                            }
                            drop(tabs);
                            let _ = windows::Win32::Graphics::Gdi::InvalidateRect(hwnd_parent, None, false);
                        }
                    }
                }
            }
            Ok(())
        }));
        let mut title_token = EventRegistrationToken::default();
        unsafe {
            let _ = self.webview.add_DocumentTitleChanged(&title_handler, &mut title_token);
        }

        // 4. NavigationCompleted (Record visit in history and final state sync)
        let completed_handler = NavigationCompletedEventHandler::create(Box::new(move |sender, args| {
            if let Some(args) = args {
                unsafe {
                    let mut is_success = BOOL(0);
                    let _ = args.IsSuccess(&mut is_success);
                    let mut web_error = COREWEBVIEW2_WEB_ERROR_STATUS::default();
                    let _ = args.WebErrorStatus(&mut web_error);
                    log::info!("[WebView2] Navigation Completed: success={}, status={:?}", is_success.as_bool(), web_error);

                    if is_success.as_bool() {
                        if let Some(ref s) = sender {
                            let mut uri = PWSTR::null();
                            if s.Source(&mut uri).is_ok() {
                                let url_str = string_from_pcwstr(&PCWSTR(uri.0));
                                let mut title = PWSTR::null();
                                let title_str = if s.DocumentTitle(&mut title).is_ok() {
                                    string_from_pcwstr(&PCWSTR(title.0))
                                } else {
                                    String::new()
                                };

                                // Guard: Never record or sync about:blank or data: URIs
                                if !url_str.is_empty() && url_str != "about:blank" && !url_str.starts_with("data:") {
                                    let ptr = windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW(
                                        hwnd_parent,
                                        windows::Win32::UI::WindowsAndMessaging::GWLP_USERDATA,
                                    ) as *mut crate::browser::host_window::WindowContext;
                                    if !ptr.is_null() {
                                        let mut tabs = (*ptr).tab_manager.borrow_mut();
                                        if let Some(active) = tabs.get_active_tab_mut() {
                                            // Guard: Never allow an external navigation to overwrite an internal page
                                            if !active.url.starts_with("nexora://") {
                                                active.url = url_str.clone();
                                                active.push_history(&url_str);
                                                if !title_str.trim().is_empty() && title_str != "about:blank" && !title_str.starts_with("data:") {
                                                    active.title = title_str.clone();
                                                }
                                                if let Some(ref chrome) = (*ptr).chrome {
                                                    chrome.set_omnibox_text(&url_str);
                                                }
                                            }
                                        }

                                        let is_bm = if !url_str.starts_with("nexora://") {
                                            if let Ok(paths) = crate::app::application::ApplicationPaths::resolve() {
                                                let db_path = paths.data_dir.join("nexora.db");
                                                if let Ok(conn) = crate::storage::SqliteConnection::open(&db_path) {
                                                    let bm_mgr = crate::storage::BookmarkManager::new(std::sync::Arc::new(conn));
                                                    bm_mgr.is_bookmarked(&url_str).unwrap_or(false)
                                                } else { false }
                                            } else { false }
                                        } else { false };

                                        if let Some(ref mut chrome) = (*ptr).chrome {
                                            chrome.set_is_bookmarked(is_bm);
                                        }

                                        drop(tabs);
                                        let _ = windows::Win32::Graphics::Gdi::InvalidateRect(hwnd_parent, None, false);
                                    }

                                    if !url_str.starts_with("nexora://") {
                                        if let Ok(paths) = crate::app::application::ApplicationPaths::resolve() {
                                            let db_path = paths.data_dir.join("nexora.db");
                                            if let Ok(conn) = crate::storage::SqliteConnection::open(&db_path) {
                                                let hist_mgr = crate::storage::HistoryManager::new(std::sync::Arc::new(conn));
                                                let _ = hist_mgr.add_visit(&url_str, &title_str);
                                            }
                                        }
                                    }

                                    // Trim working sets to keep memory tightly bounded below 200MB
                                    let _ = crate::performance::memory::trim_child_processes();
                                }
                            }
                        }
                    }
                }
            }
            Ok(())
        }));
        let mut completed_token = EventRegistrationToken::default();
        unsafe {
            let _ = self.webview.add_NavigationCompleted(&completed_handler, &mut completed_token);
        }

        // 5. NewWindowRequested (Intercept target="_blank" and popups -> open as native Nexora tab)
        let new_window_handler = NewWindowRequestedEventHandler::create(Box::new(move |_sender, args| {
            if let Some(args) = args {
                unsafe {
                    let _ = args.SetHandled(true);
                    let mut uri = PWSTR::null();
                    if args.Uri(&mut uri).is_ok() {
                        let target_url = string_from_pcwstr(&PCWSTR(uri.0));
                        log::info!("[WebView2] NewWindowRequested for '{}' -> Opening as tab in Nexora", target_url);

                        let ptr = windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW(
                            hwnd_parent,
                            windows::Win32::UI::WindowsAndMessaging::GWLP_USERDATA,
                        ) as *mut crate::browser::host_window::WindowContext;
                        if !ptr.is_null() {
                            let mut tabs = (*ptr).tab_manager.borrow_mut();
                            let _tab_id = tabs.create_tab(&target_url, None);
                            let tab_count = tabs.tab_count();
                            drop(tabs);

                            if let Some(ref webview) = (*ptr).webview {
                                let _ = webview.navigate(&target_url);
                            }
                            if let Some(ref chrome) = (*ptr).chrome {
                                chrome.set_omnibox_text(&target_url);
                                chrome.layout_controls(&(*ptr).layout, tab_count);
                            }
                            let _ = windows::Win32::Graphics::Gdi::InvalidateRect(hwnd_parent, None, false);
                        }
                    }
                }
            }
            Ok(())
        }));
        let mut new_window_token = EventRegistrationToken::default();
        unsafe {
            let _ = self.webview.add_NewWindowRequested(&new_window_handler, &mut new_window_token);
        }

        // ProcessFailed (Crash Recovery Hook - TASK-803)
        let crash_controller = std::sync::Arc::new(std::sync::Mutex::new(crate::engine::CrashRecoveryController::default()));
        let crash_ctrl_clone = crash_controller.clone();

        let process_failed_handler = ProcessFailedEventHandler::create(Box::new(move |sender, args| {
            if let Some(args) = args {
                unsafe {
                    let mut kind = COREWEBVIEW2_PROCESS_FAILED_KIND::default();
                    let _ = args.ProcessFailedKind(&mut kind);

                    let mut uri_str = String::from("about:blank");
                    if let Some(ref s) = sender {
                        let mut uri = PWSTR::null();
                        if s.Source(&mut uri).is_ok() {
                            uri_str = string_from_pcwstr(&PCWSTR(uri.0));
                        }
                    }

                    let reason = format!("{:?}", kind);
                    let action = if let Ok(mut ctrl) = crash_ctrl_clone.lock() {
                        ctrl.handle_failure(&uri_str, &reason)
                    } else {
                        crate::engine::CrashRecoveryAction::SilentReload(uri_str.clone())
                    };

                    if let Some(s) = sender {
                        match action {
                            crate::engine::CrashRecoveryAction::SilentReload(url) => {
                                log::info!("[CrashRecovery] Auto-respawning crashed renderer for {}", url);
                                let _ = s.Reload();
                            }
                            crate::engine::CrashRecoveryAction::ShowCrashCard(html) => {
                                log::warn!("[CrashRecovery] Crash limit reached, presenting fallback card");
                                let wide_html = pwstr_from_str(&html);
                                let _ = s.NavigateToString(PCWSTR(wide_html.0));
                            }
                        }
                    }
                }
            }
            Ok(())
        }));
        let mut failed_token = EventRegistrationToken::default();
        unsafe {
            let _ = self.webview.add_ProcessFailed(&process_failed_handler, &mut failed_token);
        }

        // ContainsFullScreenElementChanged (HTML5 Fullscreen Hook - TASK-505)
        let hwnd_parent = hwnd;
        let ctl_for_fs = self.controller.clone();
        let fs_handler = ContainsFullScreenElementChangedEventHandler::create(Box::new(move |sender, _args| {
            if let Some(sender) = sender {
                unsafe {
                    let mut is_fs = BOOL(0);
                    if sender.ContainsFullScreenElement(&mut is_fs).is_ok() {
                        let entering_fs = is_fs.as_bool();
                        log::info!("[Media Engine] HTML5 Fullscreen changed: {}", entering_fs);
                        let ptr = windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW(
                            hwnd_parent,
                            windows::Win32::UI::WindowsAndMessaging::GWLP_USERDATA,
                        ) as *mut crate::browser::host_window::WindowContext;
                        if !ptr.is_null() {
                            if entering_fs {
                                let (_, _, w, h) = (*ptr).fullscreen.enter_fullscreen(hwnd_parent);
                                let rect = RECT { left: 0, top: 0, right: w, bottom: h };
                                let _ = ctl_for_fs.SetBounds(rect);
                            } else {
                                let (_, _, w, h) = (*ptr).fullscreen.exit_fullscreen(hwnd_parent);
                                (*ptr).layout = crate::ui::layout::BrowserLayout::new(w, h);
                                let (wx, wy, ww, wh) = (*ptr).layout.webview_rect();
                                let rect = RECT { left: wx, top: wy, right: wx + ww, bottom: wy + wh };
                                let _ = ctl_for_fs.SetBounds(rect);
                            }
                            let _ = windows::Win32::Graphics::Gdi::InvalidateRect(hwnd_parent, None, false);
                        }
                    }
                }
            }
            Ok(())
        }));
        let mut fs_token = EventRegistrationToken::default();
        unsafe {
            let _ = self.webview.add_ContainsFullScreenElementChanged(&fs_handler, &mut fs_token);
        }

        // AcceleratorKeyPressed (Forward F11, Escape, F5, Ctrl+shortcuts to host window)
        let accel_handler = AcceleratorKeyPressedEventHandler::create(Box::new(move |_sender, args| {
            if let Some(args) = args {
                unsafe {
                    let mut kind = COREWEBVIEW2_KEY_EVENT_KIND::default();
                    let _ = args.KeyEventKind(&mut kind);
                    if kind == COREWEBVIEW2_KEY_EVENT_KIND_KEY_DOWN || kind == COREWEBVIEW2_KEY_EVENT_KIND_SYSTEM_KEY_DOWN {
                        let mut vk: u32 = 0;
                        if args.VirtualKey(&mut vk).is_ok() {
                            // F11 (0x7A), Escape (0x1B), F5 (0x74)
                            if vk == 0x7A || vk == 0x1B || vk == 0x74 {
                                let _ = windows::Win32::UI::WindowsAndMessaging::PostMessageW(
                                    hwnd_parent,
                                    windows::Win32::UI::WindowsAndMessaging::WM_KEYDOWN,
                                    windows::Win32::Foundation::WPARAM(vk as usize),
                                    windows::Win32::Foundation::LPARAM(0),
                                );
                                let _ = args.SetHandled(BOOL(1));
                            }
                        }
                    }
                }
            }
            Ok(())
        }));
        let mut accel_token = EventRegistrationToken::default();
        unsafe {
            let _ = self.controller.add_AcceleratorKeyPressed(&accel_handler, &mut accel_token);
        }

        // IsDocumentPlayingAudioChanged (Active Media Detection - TASK-504)
        if let Ok(wv8) = self.webview.cast::<ICoreWebView2_8>() {
            let audio_handler = IsDocumentPlayingAudioChangedEventHandler::create(Box::new(|sender, _args| {
                if let Some(sender) = sender {
                    unsafe {
                        if let Ok(wv8_sender) = sender.cast::<ICoreWebView2_8>() {
                            let mut is_audio = BOOL(0);
                            if wv8_sender.IsDocumentPlayingAudio(&mut is_audio).is_ok() {
                                log::info!("[Media Engine] Document audio playback changed: {}", is_audio.as_bool());
                                if is_audio.as_bool() {
                                    // Video/Audio playback detected -> trim working set immediately
                                    let _ = crate::performance::memory::trim_child_processes();
                                }
                            }
                        }
                    }
                }
                Ok(())
            }));
            let mut audio_token = EventRegistrationToken::default();
            unsafe {
                let _ = wv8.add_IsDocumentPlayingAudioChanged(&audio_handler, &mut audio_token);
            }
        }

        // PermissionRequested (Per-Site Permissions - TASK-702)
        let perm_handler = PermissionRequestedEventHandler::create(Box::new(|_sender, args| {
            if let Some(args) = args {
                unsafe {
                    let mut uri = PWSTR::null();
                    let _ = args.Uri(&mut uri);
                    let origin_str = string_from_pcwstr(&PCWSTR(uri.0));

                    let mut kind = COREWEBVIEW2_PERMISSION_KIND::default();
                    let _ = args.PermissionKind(&mut kind);

                    log::info!("[Permissions] Permission requested: {:?} for origin '{}'", kind, origin_str);

                    let norm_origin = crate::privacy::PermissionsManager::normalize_origin(&origin_str);
                    let maybe_pk = match kind {
                        COREWEBVIEW2_PERMISSION_KIND_CAMERA => Some(crate::privacy::PermissionKind::Camera),
                        COREWEBVIEW2_PERMISSION_KIND_MICROPHONE => Some(crate::privacy::PermissionKind::Microphone),
                        COREWEBVIEW2_PERMISSION_KIND_GEOLOCATION => Some(crate::privacy::PermissionKind::Geolocation),
                        COREWEBVIEW2_PERMISSION_KIND_NOTIFICATIONS => Some(crate::privacy::PermissionKind::Notifications),
                        COREWEBVIEW2_PERMISSION_KIND_CLIPBOARD_READ => Some(crate::privacy::PermissionKind::ClipboardRead),
                        COREWEBVIEW2_PERMISSION_KIND_AUTOPLAY => Some(crate::privacy::PermissionKind::Autoplay),
                        _ => None,
                    };

                    if let Some(pk) = maybe_pk {
                        let state = if let Ok(paths) = crate::app::application::ApplicationPaths::resolve() {
                            let db_path = paths.data_dir.join("nexora.db");
                            if let Ok(conn) = crate::storage::SqliteConnection::open(&db_path) {
                                let pm = crate::privacy::PermissionsManager::new(std::sync::Arc::new(conn));
                                pm.get_permission(&norm_origin, pk).unwrap_or(crate::privacy::PermissionState::Prompt)
                            } else {
                                crate::privacy::PermissionState::Prompt
                            }
                        } else {
                            crate::privacy::PermissionState::Prompt
                        };

                        let wv_state = match state {
                            crate::privacy::PermissionState::Allow => COREWEBVIEW2_PERMISSION_STATE_ALLOW,
                            crate::privacy::PermissionState::Block => COREWEBVIEW2_PERMISSION_STATE_DENY,
                            crate::privacy::PermissionState::Prompt => COREWEBVIEW2_PERMISSION_STATE_DEFAULT,
                        };
                        let _ = args.SetState(wv_state);
                        log::info!("[Permissions] Enforced permission {:?} for {} -> {:?}", pk, norm_origin, wv_state);
                    }
                }
            }
            Ok(())
        }));
        let mut perm_token = EventRegistrationToken::default();
        unsafe {
            let _ = self.webview.add_PermissionRequested(&perm_handler, &mut perm_token);
        }

        // WebResourceRequested (High-Speed Ad & Tracker Blocker - TASK-701)
        let env_for_adblock = self._environment.clone();
        let adblock = std::sync::Arc::new(crate::privacy::AdBlockEngine::new(true, true));
        let adblock_handler = WebResourceRequestedEventHandler::create(Box::new(move |_sender, args| {
            if let Some(args) = args {
                unsafe {
                    if let Ok(req) = args.Request() {
                        let mut uri = PWSTR::null();
                        if req.Uri(&mut uri).is_ok() {
                            let url_str = string_from_pcwstr(&PCWSTR(uri.0));
                            let (should_block, category) = adblock.should_block(&url_str);
                            if should_block {
                                log::info!("[AdBlock Intercept] Blocked {:?}: {}", category, url_str);
                                let reason = pwstr_from_str("Blocked by Nexora AdBlock");
                                let headers = pwstr_from_str("Content-Type: text/plain\r\n");
                                if let Ok(resp) = env_for_adblock.CreateWebResourceResponse(
                                    None,
                                    403,
                                    PCWSTR(reason.0),
                                    PCWSTR(headers.0),
                                ) {
                                    let _ = args.SetResponse(&resp);
                                }
                            }
                        }
                    }
                }
            }
            Ok(())
        }));
        let mut adblock_token = EventRegistrationToken::default();
        unsafe {
            let filter_all = pwstr_from_str("*");
            let _ = self.webview.AddWebResourceRequestedFilter(
                PCWSTR(filter_all.0),
                COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL,
            );
            let _ = self.webview.add_WebResourceRequested(&adblock_handler, &mut adblock_token);
        }

        // WebMessageReceived (Bidirectional Settings, Bookmarks, History & Engine IPC)
        let web_message_handler = WebMessageReceivedEventHandler::create(Box::new(move |sender, args| {
            if let Some(args) = args {
                unsafe {
                    let mut message = PWSTR::null();
                    let got_str = args.TryGetWebMessageAsString(&mut message).is_ok();
                    let msg_str = if got_str && !message.0.is_null() {
                        string_from_pcwstr(&PCWSTR(message.0))
                    } else {
                        let mut json_str = PWSTR::null();
                        if args.WebMessageAsJson(&mut json_str).is_ok() && !json_str.0.is_null() {
                            string_from_pcwstr(&PCWSTR(json_str.0))
                        } else {
                            String::new()
                        }
                    };

                    if !msg_str.is_empty() {
                        log::info!("[WebView2 IPC] Received WebMessage: {}", msg_str);
                        Self::handle_ipc_message(hwnd_parent, sender.as_ref(), &msg_str);
                    }
                }
            }
            Ok(())
        }));
        let mut msg_token = EventRegistrationToken::default();
        unsafe {
            let _ = self.webview.add_WebMessageReceived(&web_message_handler, &mut msg_token);
        }

        Ok(())
    }

    fn handle_ipc_message(hwnd: HWND, sender: Option<&ICoreWebView2>, msg_str: &str) {
        #[derive(serde::Deserialize)]
        struct IpcPayload {
            action: String,
            key: Option<String>,
            value: Option<serde_json::Value>,
            id: Option<i64>,
            title: Option<String>,
            url: Option<String>,
        }

        let parsed: std::result::Result<IpcPayload, _> = serde_json::from_str(msg_str);
        match parsed {
            Ok(payload) => match payload.action.as_str() {
                "navigate" => {
                    if let Some(url) = payload.url {
                        log::info!("[IPC Action] Navigating to '{}'", url);
                        
                        let ptr = unsafe {
                            windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW(
                                hwnd,
                                windows::Win32::UI::WindowsAndMessaging::GWLP_USERDATA,
                            ) as *mut crate::browser::host_window::WindowContext
                        };
                        if !ptr.is_null() {
                            unsafe {
                                if let Some(ref chrome) = (*ptr).chrome {
                                    chrome.set_omnibox_text(&url);
                                }
                                let mut tabs = (*ptr).tab_manager.borrow_mut();
                                if let Some(active) = tabs.get_active_tab_mut() {
                                    active.url = url.clone();
                                    active.push_history(&url);
                                }
                                drop(tabs);
                                let _ = windows::Win32::Graphics::Gdi::InvalidateRect(hwnd, None, false);
                            }
                        }

                        if let Some(s) = sender {
                            unsafe {
                                if url == "nexora://bookmarks" {
                                    if let Ok(paths) = crate::app::application::ApplicationPaths::resolve() {
                                        let db_path = paths.data_dir.join("nexora.db");
                                        let bookmarks = if let Ok(conn) = crate::storage::SqliteConnection::open(&db_path) {
                                            let bm_mgr = crate::storage::BookmarkManager::new(std::sync::Arc::new(conn));
                                            bm_mgr.get_all().unwrap_or_default()
                                        } else {
                                            Vec::new()
                                        };
                                        let html = crate::storage::BookmarksUi::render_html(&bookmarks);
                                        let wide = pwstr_from_str(&html);
                                        let _ = s.NavigateToString(PCWSTR(wide.0));
                                    }
                                } else if url == "nexora://history" {
                                    if let Ok(paths) = crate::app::application::ApplicationPaths::resolve() {
                                        let db_path = paths.data_dir.join("nexora.db");
                                        let history = if let Ok(conn) = crate::storage::SqliteConnection::open(&db_path) {
                                            let hist_mgr = crate::storage::HistoryManager::new(std::sync::Arc::new(conn));
                                            hist_mgr.get_recent(200).unwrap_or_default()
                                        } else {
                                            Vec::new()
                                        };
                                        let html = crate::storage::HistoryUi::render_html(&history);
                                        let wide = pwstr_from_str(&html);
                                        let _ = s.NavigateToString(PCWSTR(wide.0));
                                    }
                                } else if url == "nexora://settings" {
                                    if let Ok(paths) = crate::app::application::ApplicationPaths::resolve() {
                                        let config = crate::app::config::NexoraConfig::load_or_default(&paths.app_data_dir);
                                        let html = crate::settings::SettingsUi::render_html(&config);
                                        let wide = pwstr_from_str(&html);
                                        let _ = s.NavigateToString(PCWSTR(wide.0));
                                    }
                                } else {
                                    let wide = pwstr_from_str(&url);
                                    let _ = s.Navigate(PCWSTR(wide.0));
                                }
                            }
                        }
                    }
                }
                "delete_bookmark" => {
                    if let Some(id) = payload.id {
                        log::info!("[IPC Action] Deleting bookmark #{}", id);
                        if let Ok(paths) = crate::app::application::ApplicationPaths::resolve() {
                            let db_path = paths.data_dir.join("nexora.db");
                            if let Ok(conn) = crate::storage::SqliteConnection::open(&db_path) {
                                let bm_mgr = crate::storage::BookmarkManager::new(std::sync::Arc::new(conn));
                                let _ = bm_mgr.delete_bookmark(id);
                            }
                        }
                    }
                }
                "add_bookmark" => {
                    if let (Some(title), Some(url)) = (payload.title, payload.url) {
                        log::info!("[IPC Action] Adding bookmark '{}' ({})", title, url);
                        if let Ok(paths) = crate::app::application::ApplicationPaths::resolve() {
                            let db_path = paths.data_dir.join("nexora.db");
                            if let Ok(conn) = crate::storage::SqliteConnection::open(&db_path) {
                                let bm_mgr = crate::storage::BookmarkManager::new(std::sync::Arc::new(conn));
                                let _ = bm_mgr.add_bookmark(&title, &url, None);
                            }
                        }
                    }
                }
                "delete_history_entry" => {
                    if let Some(id) = payload.id {
                        log::info!("[IPC Action] Deleting history entry #{}", id);
                        if let Ok(paths) = crate::app::application::ApplicationPaths::resolve() {
                            let db_path = paths.data_dir.join("nexora.db");
                            if let Ok(conn) = crate::storage::SqliteConnection::open(&db_path) {
                                let hist_mgr = crate::storage::HistoryManager::new(std::sync::Arc::new(conn));
                                let _ = hist_mgr.delete_entry(id);
                            }
                        }
                    }
                }
                "clear_history" => {
                    log::info!("[IPC Action] Clearing all browsing history...");
                    if let Ok(paths) = crate::app::application::ApplicationPaths::resolve() {
                        let db_path = paths.data_dir.join("nexora.db");
                        if let Ok(conn) = crate::storage::SqliteConnection::open(&db_path) {
                            let hist_mgr = crate::storage::HistoryManager::new(std::sync::Arc::new(conn));
                            let _ = hist_mgr.clear_all();
                        }
                    }
                }
                "save_setting" => {
                    if let (Some(key), Some(val)) = (payload.key, payload.value) {
                        Self::apply_and_save_setting(hwnd, &key, &val);
                    }
                }
                "compact_wal" => {
                    log::info!("[IPC Action] Compacting SQLite database WAL...");
                    if let Ok(paths) = crate::app::application::ApplicationPaths::resolve() {
                        let db_path = paths.data_dir.join("nexora.db");
                        if let Ok(conn) = crate::storage::SqliteConnection::open(&db_path) {
                            if let Err(e) = conn.execute("PRAGMA wal_checkpoint(TRUNCATE);") {
                                log::error!("[IPC Action] WAL checkpoint failed: {}", e);
                            } else {
                                log::info!("[IPC Action] SQLite WAL truncated and compacted successfully.");
                            }
                        }
                    }
                }
                "clear_cache" => {
                    log::info!("[IPC Action] Clearing browser cache...");
                    if let Ok(paths) = crate::app::application::ApplicationPaths::resolve() {
                        let _ = std::fs::remove_dir_all(&paths.engine_data_dir);
                        let _ = std::fs::create_dir_all(&paths.engine_data_dir);
                        log::info!("[IPC Action] Engine cache directory cleared.");
                    }
                }
                "reset_defaults" => {
                    log::info!("[IPC Action] Resetting all settings to factory defaults...");
                    if let Ok(paths) = crate::app::application::ApplicationPaths::resolve() {
                        let default_config = crate::app::config::NexoraConfig::default();
                        let _ = default_config.save(&paths.app_data_dir);
                        log::info!("[IPC Action] Default configuration saved to {:?}", paths.app_data_dir);
                    }
                }
                other => {
                    log::warn!("[WebView2 IPC] Unrecognized action: {}", other);
                }
            },
            Err(e) => {
                log::warn!("[WebView2 IPC] Failed to parse message JSON: {}. Raw: {}", e, msg_str);
            }
        }
    }

    fn apply_and_save_setting(hwnd: HWND, key: &str, val: &serde_json::Value) {
        let paths = match crate::app::application::ApplicationPaths::resolve() {
            Ok(p) => p,
            Err(e) => {
                log::error!("[Settings IPC] Could not resolve app paths: {}", e);
                return;
            }
        };

        let mut config = crate::app::config::NexoraConfig::load_or_default(&paths.app_data_dir);

        match key {
            "homepage" => {
                if let Some(s) = val.as_str() {
                    config.homepage = s.trim().to_string();
                }
            }
            "default_search_engine" => {
                if let Some(s) = val.as_str() {
                    config.default_search_engine = s.to_string();
                }
            }
            "theme" => {
                if let Some(s) = val.as_str() {
                    config.theme = match s {
                        "dark" => crate::app::config::ThemeMode::Dark,
                        "light" => crate::app::config::ThemeMode::Light,
                        _ => crate::app::config::ThemeMode::System,
                    };
                }
            }
            "hardware_acceleration" => {
                if let Some(b) = val.as_bool() {
                    config.hardware_acceleration = b;
                }
            }
            "smart_suspension" => {
                if let Some(b) = val.as_bool() {
                    config.smart_suspension = b;
                }
            }
            "suspension_timeout_minutes" => {
                if let Some(n) = val.as_u64() {
                    config.suspension_timeout_minutes = (n as u32).clamp(1, 120);
                }
            }
            "ad_block_enabled" => {
                if let Some(b) = val.as_bool() {
                    config.ad_block_enabled = b;
                }
            }
            "tracker_protection_enabled" => {
                if let Some(b) = val.as_bool() {
                    config.tracker_protection_enabled = b;
                }
            }
            "disk_cache_limit_mb" => {
                if let Some(n) = val.as_u64() {
                    config.disk_cache_limit_mb = (n as u32).clamp(32, 4096);
                }
            }
            "media_cache_limit_mb" => {
                if let Some(n) = val.as_u64() {
                    config.media_cache_limit_mb = (n as u32).clamp(16, 2048);
                }
            }
            unknown => {
                log::warn!("[Settings IPC] Unrecognized configuration key: {}", unknown);
                return;
            }
        }

        if let Err(e) = config.save(&paths.app_data_dir) {
            log::error!("[Settings IPC] Failed to save config for key '{}': {}", key, e);
        } else {
            log::info!("[Settings IPC] Successfully persisted '{}' = {:?} to config.json", key, val);
        }

        // Dynamically update live WindowContext
        let ptr = unsafe {
            windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW(
                hwnd,
                windows::Win32::UI::WindowsAndMessaging::GWLP_USERDATA,
            ) as *mut crate::browser::host_window::WindowContext
        };
        if !ptr.is_null() {
            unsafe {
                match key {
                    "homepage" => {
                        (*ptr).homepage = config.homepage.clone();
                        log::info!("[Live Settings] Updated HostWindow homepage to '{}'", (*ptr).homepage);
                    }
                    "default_search_engine" => {
                        (*ptr).default_search_engine = config.default_search_engine.clone();
                        log::info!("[Live Settings] Updated HostWindow default_search_engine to '{}'", (*ptr).default_search_engine);
                    }
                    "smart_suspension" => {
                        (*ptr).suspension_engine.enabled = config.smart_suspension;
                    }
                    "suspension_timeout_minutes" => {
                        (*ptr).suspension_engine.base_timeout_minutes = config.suspension_timeout_minutes;
                    }
                    "ad_block_enabled" => {
                        let block_trackers = (&(*ptr).adblock_engine).block_trackers;
                        (*ptr).adblock_engine = std::sync::Arc::new(crate::privacy::AdBlockEngine::new(
                            config.ad_block_enabled,
                            block_trackers,
                        ));
                    }
                    "tracker_protection_enabled" => {
                        let enabled = (&(*ptr).adblock_engine).enabled;
                        (*ptr).adblock_engine = std::sync::Arc::new(crate::privacy::AdBlockEngine::new(
                            enabled,
                            config.tracker_protection_enabled,
                        ));
                    }
                    _ => {}
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::browser::HostWindow;

    #[test]
    fn test_webview_engine_initialization() {
        let tab_mgr = std::rc::Rc::new(std::cell::RefCell::new(crate::tabs::TabManager::new()));
        let theme = crate::ui::theme::ThemePalette::dark();
        let window = HostWindow::new(
            "Nexora Engine Test Window",
            800,
            600,
            theme,
            "https://www.google.com",
            "Google",
            tab_mgr,
            true,
            10,
        )
        .expect("HostWindow creation failed in test");

        let temp_dir = std::env::temp_dir().join(format!("nexora_test_udf_{}", std::process::id()));
        let engine = WebViewEngine::new(window.hwnd(), &temp_dir, "https://www.google.com");
        
        assert!(engine.is_ok(), "WebViewEngine creation should succeed: {:?}", engine.err());
        let engine = engine.unwrap();

        let (left, top, w, h) = window.get_client_bounds();
        let bounds_res = engine.set_bounds(left, top, w, h);
        assert!(bounds_res.is_ok(), "Setting bounds should succeed");

        // Clean up temp dir
        let _ = std::fs::remove_dir_all(&temp_dir);

        // Clean up window
        unsafe {
            let _ = windows::Win32::UI::WindowsAndMessaging::DestroyWindow(window.hwnd());
        }
    }

    #[test]
    fn test_settings_ipc_save_and_persist() {
        let dummy_hwnd = HWND::default();
        let msg = r#"{"action":"save_setting","key":"homepage","value":"https://duckduckgo.com"}"#;
        WebViewEngine::handle_ipc_message(dummy_hwnd, None, msg);

        let paths = crate::app::application::ApplicationPaths::resolve().expect("paths");
        let config = crate::app::config::NexoraConfig::load_or_default(&paths.app_data_dir);
        assert_eq!(config.homepage, "https://duckduckgo.com");

        // Test theme setting
        let msg_theme = r#"{"action":"save_setting","key":"theme","value":"dark"}"#;
        WebViewEngine::handle_ipc_message(dummy_hwnd, None, msg_theme);
        let config2 = crate::app::config::NexoraConfig::load_or_default(&paths.app_data_dir);
        assert_eq!(config2.theme, crate::app::config::ThemeMode::Dark);

        // Test timeout setting
        let msg_timeout = r#"{"action":"save_setting","key":"suspension_timeout_minutes","value":25}"#;
        WebViewEngine::handle_ipc_message(dummy_hwnd, None, msg_timeout);
        let config3 = crate::app::config::NexoraConfig::load_or_default(&paths.app_data_dir);
        assert_eq!(config3.suspension_timeout_minutes, 25);

        // Reset to default
        let reset_msg = r#"{"action":"reset_defaults"}"#;
        WebViewEngine::handle_ipc_message(dummy_hwnd, None, reset_msg);
        let config_def = crate::app::config::NexoraConfig::load_or_default(&paths.app_data_dir);
        assert_eq!(config_def.homepage, "https://www.google.com");
        assert_eq!(config_def.suspension_timeout_minutes, 10);
    }
}
