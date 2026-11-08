use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::path::Path;
use std::sync::mpsc::channel;
use windows::core::{Interface, PCWSTR, PWSTR};
use windows::Win32::Foundation::{BOOL, HWND, RECT};
use windows::Win32::System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED};
use windows::Win32::System::WinRT::EventRegistrationToken;
use webview2_com::Microsoft::Web::WebView2::Win32::*;
use webview2_com::*;

use crate::core::error::{NexoraError, Result};

pub struct TabView {
    pub tab_id: usize,
    pub controller: ICoreWebView2Controller,
    pub webview: ICoreWebView2,
    pub is_internal_page: bool,
    pub current_url: String,
    pub current_title: String,
}

pub struct WebViewEngine {
    pub hwnd: HWND,
    environment: ICoreWebView2Environment,
    active_tab_id: Cell<Option<usize>>,
    bounds: Cell<RECT>,
    tab_views: RefCell<HashMap<usize, TabView>>,
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

        let default_bounds = RECT {
            left: 0,
            top: 78,
            right: 1280,
            bottom: 800,
        };

        let engine = Self {
            hwnd,
            environment,
            active_tab_id: Cell::new(Some(1)),
            bounds: Cell::new(default_bounds),
            tab_views: RefCell::new(HashMap::new()),
        };

        // Create the initial tab view for tab #1
        engine.create_tab_view(1, initial_url, default_bounds, true)?;

        Ok(engine)
    }

    /// Creates and configures an independent WebView2 controller for a tab view.
    pub fn create_tab_view(&self, tab_id: usize, initial_url: &str, bounds: RECT, make_active: bool) -> Result<()> {
        if self.tab_views.borrow().contains_key(&tab_id) {
            log::info!("[TabView] Tab #{} already exists, skipping creation.", tab_id);
            return Ok(());
        }

        log::info!("[TabView] Creating persistent controller for Tab #{} (make_active: {})...", tab_id, make_active);

        let (tx_ctl, rx_ctl) = channel();
        let ctl_handler = CreateCoreWebView2ControllerCompletedHandler::create(Box::new(move |res, ctl| {
            let _ = tx_ctl.send((res, ctl));
            Ok(())
        }));

        unsafe {
            self.environment.CreateCoreWebView2Controller(self.hwnd, &ctl_handler)
                .map_err(|e| NexoraError::Engine(format!("Failed to invoke CreateCoreWebView2Controller for tab #{}: {}", tab_id, e)))?;
        }

        let (res_ctl, ctl_opt) = wait_with_pump(rx_ctl)
            .map_err(|e| NexoraError::Engine(format!("Controller pump failed for tab #{}: {}", tab_id, e)))?;
        res_ctl.map_err(|e| NexoraError::Engine(format!("Controller creation returned error for tab #{}: {}", tab_id, e)))?;
        let controller = ctl_opt.ok_or_else(|| NexoraError::Engine(format!("Null WebView2 Controller returned for tab #{}", tab_id)))?;

        let webview = unsafe {
            controller.CoreWebView2()
                .map_err(|e| NexoraError::Engine(format!("Failed to get CoreWebView2 from controller for tab #{}: {}", tab_id, e)))?
        };

        unsafe {
            let _ = controller.SetBounds(bounds);
            let _ = controller.SetIsVisible(make_active);
            if make_active {
                let _ = controller.MoveFocus(COREWEBVIEW2_MOVE_FOCUS_REASON_PROGRAMMATIC);
            }
            if let Ok(settings) = webview.Settings() {
                let _ = settings.SetIsScriptEnabled(true);
                let _ = settings.SetIsWebMessageEnabled(true);
                let _ = settings.SetAreDefaultScriptDialogsEnabled(true);
                let _ = settings.SetIsStatusBarEnabled(false);
                let _ = settings.SetAreDevToolsEnabled(cfg!(debug_assertions));
            }
        }

        // Hook per-tab event handlers
        self.setup_tab_event_handlers(tab_id, &controller, &webview, self.hwnd)?;

        let is_internal = initial_url.starts_with("nexora://") || initial_url.is_empty() || initial_url == "about:blank";
        let initial_title = if is_internal {
            match initial_url {
                "nexora://settings" => "Settings",
                "nexora://diagnostics" => "Diagnostics",
                "nexora://bookmarks" => "Bookmarks",
                "nexora://history" => "History",
                _ => "Nexora Home",
            }.to_string()
        } else {
            "New Tab".to_string()
        };

        let tab_view = TabView {
            tab_id,
            controller,
            webview,
            is_internal_page: is_internal,
            current_url: initial_url.to_string(),
            current_title: initial_title,
        };

        self.tab_views.borrow_mut().insert(tab_id, tab_view);

        // Perform initial navigation
        self.navigate_tab(tab_id, initial_url)?;

        if make_active {
            // Guarantee all other tab controllers are hidden
            for (&id, view) in self.tab_views.borrow().iter() {
                if id != tab_id {
                    unsafe {
                        let _ = view.controller.SetIsVisible(false);
                    }
                }
            }
            self.active_tab_id.set(Some(tab_id));
        }

        log::info!("[TabView] Tab #{} initialized successfully.", tab_id);
        Ok(())
    }

    /// Performs instantaneous tab switching (< 15ms) via `SetIsVisible` toggling.
    /// Strictly guarantees that ONLY `tab_id` is visible, and all other controllers are hidden.
    pub fn switch_to_tab(&self, tab_id: usize, bounds: RECT) -> Result<()> {
        self.active_tab_id.set(Some(tab_id));

        // 1. Ensure destination tab view exists (lazy restore if suspended)
        let exists = self.tab_views.borrow().contains_key(&tab_id);
        if !exists {
            let (target_url, _) = unsafe {
                let ptr = windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW(
                    self.hwnd,
                    windows::Win32::UI::WindowsAndMessaging::GWLP_USERDATA,
                ) as *mut crate::browser::host_window::WindowContext;
                if !ptr.is_null() {
                    let tabs = (*ptr).tab_manager.borrow();
                    tabs.get_tab(tab_id)
                        .map(|t| (t.url.clone(), t.title.clone()))
                        .unwrap_or(("nexora://home".into(), "Nexora Home".into()))
                } else {
                    ("nexora://home".into(), "Nexora Home".into())
                }
            };
            self.create_tab_view(tab_id, &target_url, bounds, true)?;
        }

        // 2. Hide ALL other tab view controllers and display ONLY tab_id
        for (&id, view) in self.tab_views.borrow().iter() {
            unsafe {
                if id == tab_id {
                    let _ = view.controller.SetBounds(bounds);
                    let _ = view.controller.SetIsVisible(true);
                    let _ = view.controller.MoveFocus(COREWEBVIEW2_MOVE_FOCUS_REASON_PROGRAMMATIC);
                } else {
                    let _ = view.controller.SetIsVisible(false);
                }
            }
        }

        Ok(())
    }

    /// Closes a tab view and releases its WebView2 controller resources.
    pub fn close_tab_view(&self, tab_id: usize) {
        if let Some(view) = self.tab_views.borrow_mut().remove(&tab_id) {
            unsafe {
                let _ = view.controller.SetIsVisible(false);
                let _ = view.controller.Close();
            }
            log::info!("[TabView] Tab #{} controller closed and dropped.", tab_id);
        }
        if self.active_tab_id.get() == Some(tab_id) {
            self.active_tab_id.set(None);
        }
    }

    /// Suspends a tab view under memory pressure, closing controller while tab state remains in TabManager.
    pub fn suspend_tab_view(&self, tab_id: usize) {
        if let Some(view) = self.tab_views.borrow_mut().remove(&tab_id) {
            unsafe {
                let _ = view.controller.SetIsVisible(false);
                let _ = view.controller.Close();
            }
            log::info!("[TabView] Tab #{} suspended: controller closed to reclaim physical memory.", tab_id);
        }
    }

    pub fn has_tab_view(&self, tab_id: usize) -> bool {
        self.tab_views.borrow().contains_key(&tab_id)
    }

    pub fn active_tab_id(&self) -> Option<usize> {
        self.active_tab_id.get()
    }

    pub fn set_bounds(&self, x: i32, y: i32, width: i32, height: i32) -> Result<()> {
        let rect = RECT {
            left: x,
            top: y,
            right: x + width,
            bottom: y + height,
        };
        self.bounds.set(rect);
        if let Some(active_id) = self.active_tab_id.get() {
            if let Some(view) = self.tab_views.borrow().get(&active_id) {
                unsafe {
                    self.controller_set_bounds(&view.controller, rect)?;
                }
            }
        }
        Ok(())
    }

    unsafe fn controller_set_bounds(&self, controller: &ICoreWebView2Controller, rect: RECT) -> Result<()> {
        controller.SetBounds(rect)
            .map_err(|e| NexoraError::Engine(format!("Failed to set webview bounds: {}", e)))?;
        Ok(())
    }

    pub fn navigate(&self, target_url: &str) -> Result<()> {
        let active_id = unsafe {
            let ptr = windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW(
                self.hwnd,
                windows::Win32::UI::WindowsAndMessaging::GWLP_USERDATA,
            ) as *mut crate::browser::host_window::WindowContext;
            if !ptr.is_null() {
                (*ptr).tab_manager.borrow().active_tab_id().unwrap_or(1)
            } else {
                self.active_tab_id.get().unwrap_or(1)
            }
        };
        self.navigate_tab(active_id, target_url)
    }

    pub fn navigate_tab(&self, tab_id: usize, target_url: &str) -> Result<()> {
        let clean_url = target_url.trim_end_matches('/');
        let is_internal = clean_url == "nexora://home"
            || clean_url == "nexora://newtab"
            || clean_url == "nexora:"
            || clean_url == "nexora://settings"
            || clean_url == "nexora://diagnostics"
            || clean_url == "nexora://bookmarks"
            || clean_url == "nexora://history"
            || clean_url.is_empty();

        if is_internal {
            let html = self.render_internal_page(clean_url);
            let views = self.tab_views.borrow();
            if let Some(view) = views.get(&tab_id) {
                log::info!("[TabView] Rendering internal page '{}' for Tab #{} ({} bytes)", clean_url, tab_id, html.len());
                let wide = pwstr_from_str(&html);
                unsafe {
                    view.webview.NavigateToString(PCWSTR(wide.0))
                        .map_err(|e| NexoraError::Navigation(format!("NavigateToString failed: {}", e)))?;
                }
            }
            drop(views);
            if let Some(view) = self.tab_views.borrow_mut().get_mut(&tab_id) {
                view.is_internal_page = true;
                view.current_url = if clean_url.is_empty() || clean_url == "nexora:" || clean_url == "nexora://newtab" {
                    "nexora://home".to_string()
                } else {
                    clean_url.to_string()
                };
            }
            return Ok(());
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

        let views = self.tab_views.borrow();
        if let Some(view) = views.get(&tab_id) {
            log::info!("[TabView] Navigating tab #{} to: {}", tab_id, normalized);
            let wide = pwstr_from_str(&normalized);
            unsafe {
                view.webview.Navigate(PCWSTR(wide.0))
                    .map_err(|e| NexoraError::Navigation(format!("Navigation failed for {}: {}", normalized, e)))?;
            }
        }
        drop(views);
        if let Some(view) = self.tab_views.borrow_mut().get_mut(&tab_id) {
            view.is_internal_page = false;
            view.current_url = normalized;
        }
        Ok(())
    }

    pub fn navigate_to_string(&self, html_content: &str) -> Result<()> {
        if let Some(active_id) = self.active_tab_id.get() {
            let views = self.tab_views.borrow();
            if let Some(view) = views.get(&active_id) {
                let wide = pwstr_from_str(html_content);
                unsafe {
                    view.webview.NavigateToString(PCWSTR(wide.0))
                        .map_err(|e| NexoraError::Navigation(format!("NavigateToString failed: {}", e)))?;
                }
            }
        }
        Ok(())
    }

    pub fn go_back(&self) -> Result<()> {
        if let Some(active_id) = self.active_tab_id.get() {
            let views = self.tab_views.borrow();
            if let Some(view) = views.get(&active_id) {
                unsafe {
                    let mut can = BOOL(0);
                    let _ = view.webview.CanGoBack(&mut can);
                    if can.as_bool() {
                        view.webview.GoBack()
                            .map_err(|e| NexoraError::Engine(format!("GoBack failed: {}", e)))?;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn go_forward(&self) -> Result<()> {
        if let Some(active_id) = self.active_tab_id.get() {
            let views = self.tab_views.borrow();
            if let Some(view) = views.get(&active_id) {
                unsafe {
                    let mut can = BOOL(0);
                    let _ = view.webview.CanGoForward(&mut can);
                    if can.as_bool() {
                        view.webview.GoForward()
                            .map_err(|e| NexoraError::Engine(format!("GoForward failed: {}", e)))?;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn reload_tab(&self, tab_id: usize) -> Result<()> {
        let views = self.tab_views.borrow();
        if let Some(view) = views.get(&tab_id) {
            if view.is_internal_page {
                let url = view.current_url.clone();
                drop(views);
                return self.navigate_tab(tab_id, &url);
            }
            unsafe {
                view.webview.Reload()
                    .map_err(|e| NexoraError::Engine(format!("Reload failed: {}", e)))?;
            }
        }
        Ok(())
    }

    pub fn reload(&self) -> Result<()> {
        let active_id = unsafe {
            let ptr = windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW(
                self.hwnd,
                windows::Win32::UI::WindowsAndMessaging::GWLP_USERDATA,
            ) as *mut crate::browser::host_window::WindowContext;
            if !ptr.is_null() {
                (*ptr).tab_manager.borrow().active_tab_id().unwrap_or(1)
            } else {
                self.active_tab_id.get().unwrap_or(1)
            }
        };
        self.reload_tab(active_id)
    }

    pub fn stop(&self) -> Result<()> {
        if let Some(active_id) = self.active_tab_id.get() {
            let views = self.tab_views.borrow();
            if let Some(view) = views.get(&active_id) {
                unsafe {
                    view.webview.Stop()
                        .map_err(|e| NexoraError::Engine(format!("Stop failed: {}", e)))?;
                }
            }
        }
        Ok(())
    }

    pub fn get_source(&self) -> Result<String> {
        if let Some(active_id) = self.active_tab_id.get() {
            let views = self.tab_views.borrow();
            if let Some(view) = views.get(&active_id) {
                if view.is_internal_page {
                    return Ok(view.current_url.clone());
                }
                unsafe {
                    let mut uri = PWSTR::null();
                    view.webview.Source(&mut uri)
                        .map_err(|e| NexoraError::Engine(format!("Failed to get source URL: {}", e)))?;
                    return Ok(string_from_pcwstr(&PCWSTR(uri.0)));
                }
            }
        }
        Ok(String::new())
    }

    pub fn get_title(&self) -> Result<String> {
        if let Some(active_id) = self.active_tab_id.get() {
            let views = self.tab_views.borrow();
            if let Some(view) = views.get(&active_id) {
                unsafe {
                    let mut title = PWSTR::null();
                    view.webview.DocumentTitle(&mut title)
                        .map_err(|e| NexoraError::Engine(format!("Failed to get document title: {}", e)))?;
                    return Ok(string_from_pcwstr(&PCWSTR(title.0)));
                }
            }
        }
        Ok(String::new())
    }

    pub fn execute_script(&self, script: &str) -> Result<()> {
        if let Some(active_id) = self.active_tab_id.get() {
            let views = self.tab_views.borrow();
            if let Some(view) = views.get(&active_id) {
                let wide = pwstr_from_str(script);
                unsafe {
                    view.webview.ExecuteScript(PCWSTR(wide.0), None)
                        .map_err(|e| NexoraError::Engine(format!("ExecuteScript failed: {}", e)))?;
                }
            }
        }
        Ok(())
    }

    pub fn trigger_pip(&self) -> Result<()> {
        let script = crate::media::PipWindow::generate_pip_script();
        self.execute_script(script)
    }

    pub fn set_tab_muted(&self, tab_id: usize, muted: bool) {
        let views = self.tab_views.borrow();
        if let Some(view) = views.get(&tab_id) {
            unsafe {
                if let Ok(wv8) = view.webview.cast::<ICoreWebView2_8>() {
                    let _ = wv8.SetIsMuted(BOOL(if muted { 1 } else { 0 }));
                }
            }
            let script = format!(
                "try {{ document.querySelectorAll('audio, video').forEach(el => {{ el.muted = {}; }}); }} catch(e) {{}}",
                muted
            );
            let wide = pwstr_from_str(&script);
            unsafe {
                let _ = view.webview.ExecuteScript(PCWSTR(wide.0), None);
            }
        }
    }

    pub fn get_tab_muted(&self, tab_id: usize) -> bool {
        let views = self.tab_views.borrow();
        if let Some(view) = views.get(&tab_id) {
            unsafe {
                if let Ok(wv8) = view.webview.cast::<ICoreWebView2_8>() {
                    let mut is_muted = BOOL(0);
                    if wv8.IsMuted(&mut is_muted).is_ok() {
                        return is_muted.as_bool();
                    }
                }
            }
        }
        false
    }

    pub fn set_tab_zoom(&self, tab_id: usize, zoom: f64) -> Result<()> {
        let views = self.tab_views.borrow();
        if let Some(view) = views.get(&tab_id) {
            unsafe {
                view.controller.SetZoomFactor(zoom)
                    .map_err(|e| NexoraError::Engine(format!("SetZoomFactor failed: {}", e)))?;
            }
        }
        Ok(())
    }

    pub fn get_tab_zoom(&self, tab_id: usize) -> f64 {
        let views = self.tab_views.borrow();
        if let Some(view) = views.get(&tab_id) {
            unsafe {
                let mut zoom = 1.0;
                if view.controller.ZoomFactor(&mut zoom).is_ok() {
                    return zoom;
                }
            }
        }
        1.0
    }

    fn render_internal_page(&self, clean_url: &str) -> String {
        let clean = clean_url.trim_end_matches('/');
        if clean == "nexora://home" || clean == "nexora://newtab" || clean == "nexora:" || clean.is_empty() {
            let app_data_dir = crate::app::application::ApplicationPaths::resolve()
                .map(|p| p.app_data_dir)
                .unwrap_or_else(|_| std::env::temp_dir());
            let config = crate::app::config::NexoraConfig::load_or_default(&app_data_dir);
            let data_dir = crate::app::application::ApplicationPaths::resolve()
                .map(|p| p.data_dir)
                .unwrap_or_else(|_| std::env::temp_dir());
            let db_path = data_dir.join("nexora.db");
            let bookmarks = if let Ok(conn) = crate::storage::SqliteConnection::open(&db_path) {
                let bm_mgr = crate::storage::BookmarkManager::new(std::sync::Arc::new(conn));
                bm_mgr.get_all().unwrap_or_default()
            } else {
                Vec::new()
            };
            crate::home::HomeUi::render_html(&config, &bookmarks)
        } else if clean == "nexora://settings" {
            let app_data_dir = crate::app::application::ApplicationPaths::resolve()
                .map(|p| p.app_data_dir)
                .unwrap_or_else(|_| std::env::temp_dir());
            let config = crate::app::config::NexoraConfig::load_or_default(&app_data_dir);
            crate::settings::SettingsUi::render_html(&config)
        } else if clean == "nexora://diagnostics" {
            let (total_tabs, active_title, active_url, suspended_tabs, adblock_stats) = unsafe {
                let ptr = windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW(
                    self.hwnd,
                    windows::Win32::UI::WindowsAndMessaging::GWLP_USERDATA,
                ) as *mut crate::browser::host_window::WindowContext;
                if !ptr.is_null() {
                    let tabs = (*ptr).tab_manager.borrow();
                    let total = tabs.tab_count();
                    let suspended = tabs.tabs().iter().filter(|t| t.is_suspended()).count();
                    let (title, url) = tabs.get_active_tab()
                        .map(|t| (t.title.clone(), t.url.clone()))
                        .unwrap_or(("Nexora Diagnostics".to_string(), "nexora://diagnostics".to_string()));
                    let stats = (*ptr).adblock_engine.get_stats();
                    (total, title, url, suspended, stats)
                } else {
                    (1, "Nexora Diagnostics".to_string(), "nexora://diagnostics".to_string(), 0, Default::default())
                }
            };
            let resident_views = self.tab_views.borrow().len();
            let snapshot = crate::diagnostics::DiagnosticsSnapshot::capture(
                total_tabs,
                &active_title,
                &active_url,
                suspended_tabs,
                resident_views,
                adblock_stats,
                false,
            );
            crate::diagnostics::DiagnosticsUi::render_html(&snapshot)
        } else if clean == "nexora://bookmarks" {
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
            crate::storage::BookmarksUi::render_html(&bookmarks)
        } else if clean == "nexora://history" {
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
            crate::storage::HistoryUi::render_html(&history)
        } else {
            format!("<html><body style='background:#121214;color:#f4f4f5;'>Page not found: {}</body></html>", clean)
        }
    }

    fn setup_tab_event_handlers(&self, tab_id: usize, controller: &ICoreWebView2Controller, webview: &ICoreWebView2, hwnd: HWND) -> Result<()> {
        let hwnd_parent = hwnd;

        // 1. NavigationStarting
        let starting_handler = NavigationStartingEventHandler::create(Box::new(move |_sender, args| {
            if let Some(args) = args {
                unsafe {
                    let mut uri = PWSTR::null();
                    if args.Uri(&mut uri).is_ok() {
                        let url_str = string_from_pcwstr(&PCWSTR(uri.0));
                        log::info!("[WebView2 Tab #{}] Navigation Starting: {}", tab_id, url_str);

                        // Strict Guard: Never overwrite omnibox or tab state with about:blank or data: URIs
                        if url_str.is_empty() || url_str == "about:blank" || url_str.starts_with("data:") {
                            return Ok(());
                        }

                        let ptr = windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW(
                            hwnd_parent,
                            windows::Win32::UI::WindowsAndMessaging::GWLP_USERDATA,
                        ) as *mut crate::browser::host_window::WindowContext;
                        if !ptr.is_null() {
                            let is_active = (*ptr).tab_manager.borrow().active_tab_id() == Some(tab_id);
                            let mut tabs = (*ptr).tab_manager.borrow_mut();
                            if let Some(tab) = tabs.get_tab_mut(tab_id) {
                                if !tab.url.starts_with("nexora://") {
                                    tab.url = url_str.clone();
                                    tab.push_history(&url_str);
                                    if is_active {
                                        if let Some(ref chrome) = (*ptr).chrome {
                                            chrome.set_omnibox_text(&url_str);
                                        }
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
            let _ = webview.add_NavigationStarting(&starting_handler, &mut starting_token);
        }

        // 2. SourceChanged
        let source_handler = SourceChangedEventHandler::create(Box::new(move |sender, _args| {
            if let Some(sender) = sender {
                unsafe {
                    let mut uri = PWSTR::null();
                    if sender.Source(&mut uri).is_ok() {
                        let url_str = string_from_pcwstr(&PCWSTR(uri.0));
                        log::info!("[WebView2 Tab #{}] Source Changed: {}", tab_id, url_str);

                        // Strict Guard: Never overwrite omnibox or tab state with about:blank or data: URIs
                        if url_str.is_empty() || url_str == "about:blank" || url_str.starts_with("data:") {
                            return Ok(());
                        }

                        let ptr = windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW(
                            hwnd_parent,
                            windows::Win32::UI::WindowsAndMessaging::GWLP_USERDATA,
                        ) as *mut crate::browser::host_window::WindowContext;
                        if !ptr.is_null() {
                            let is_active = (*ptr).tab_manager.borrow().active_tab_id() == Some(tab_id);
                            let mut tabs = (*ptr).tab_manager.borrow_mut();
                            if let Some(tab) = tabs.get_tab_mut(tab_id) {
                                if !tab.url.starts_with("nexora://") {
                                    tab.url = url_str.clone();
                                    tab.push_history(&url_str);
                                    if is_active {
                                        if let Some(ref chrome) = (*ptr).chrome {
                                            chrome.set_omnibox_text(&url_str);
                                        }
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
            let _ = webview.add_SourceChanged(&source_handler, &mut source_token);
        }

        // 3. DocumentTitleChanged
        let title_handler = DocumentTitleChangedEventHandler::create(Box::new(move |sender, _args| {
            if let Some(sender) = sender {
                unsafe {
                    let mut title = PWSTR::null();
                    if sender.DocumentTitle(&mut title).is_ok() {
                        let title_str = string_from_pcwstr(&PCWSTR(title.0));
                        log::info!("[WebView2 Tab #{}] Document Title Changed: {}", tab_id, title_str);

                        // Guard: Do not overwrite titles with blank, about:blank, or data:
                        if title_str.trim().is_empty() || title_str == "about:blank" || title_str.starts_with("data:") {
                            return Ok(());
                        }

                        let ptr = windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW(
                            hwnd_parent,
                            windows::Win32::UI::WindowsAndMessaging::GWLP_USERDATA,
                        ) as *mut crate::browser::host_window::WindowContext;
                        if !ptr.is_null() {
                            let is_active = (*ptr).tab_manager.borrow().active_tab_id() == Some(tab_id);
                            let mut tabs = (*ptr).tab_manager.borrow_mut();
                            if let Some(tab) = tabs.get_tab_mut(tab_id) {
                                tab.title = title_str.clone();
                            }
                            drop(tabs);

                            if is_active {
                                let full_title = format!("{} - Nexora", title_str);
                                let wide_title: Vec<u16> = full_title.encode_utf16().chain(std::iter::once(0)).collect();
                                let _ = windows::Win32::UI::WindowsAndMessaging::SetWindowTextW(
                                    hwnd_parent,
                                    windows::core::PCWSTR(wide_title.as_ptr()),
                                );
                            }
                            let _ = windows::Win32::Graphics::Gdi::InvalidateRect(hwnd_parent, None, false);
                        }
                    }
                }
            }
            Ok(())
        }));
        let mut title_token = EventRegistrationToken::default();
        unsafe {
            let _ = webview.add_DocumentTitleChanged(&title_handler, &mut title_token);
        }

        // 4. NavigationCompleted
        let completed_handler = NavigationCompletedEventHandler::create(Box::new(move |sender, _args| {
            if let Some(s) = sender {
                unsafe {
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
                                let is_active = (*ptr).tab_manager.borrow().active_tab_id() == Some(tab_id);
                                let mut tabs = (*ptr).tab_manager.borrow_mut();
                                if let Some(tab) = tabs.get_tab_mut(tab_id) {
                                    if !tab.url.starts_with("nexora://") {
                                        tab.url = url_str.clone();
                                        tab.push_history(&url_str);
                                        if !title_str.trim().is_empty() && title_str != "about:blank" && !title_str.starts_with("data:") {
                                            tab.title = title_str.clone();
                                        }
                                        if is_active {
                                            if let Some(ref chrome) = (*ptr).chrome {
                                                chrome.set_omnibox_text(&url_str);
                                            }
                                        }
                                    }
                                }

                                if is_active {
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

                            let _ = crate::performance::memory::trim_child_processes();
                        }
                    }
                }
            }
            Ok(())
        }));
        let mut completed_token = EventRegistrationToken::default();
        unsafe {
            let _ = webview.add_NavigationCompleted(&completed_handler, &mut completed_token);
        }

        // 5. NewWindowRequested (Intercept target="_blank" and popups -> open as native Nexora tab)
        let new_window_handler = NewWindowRequestedEventHandler::create(Box::new(move |_sender, args| {
            if let Some(args) = args {
                unsafe {
                    let _ = args.SetHandled(true);
                    let mut uri = PWSTR::null();
                    if args.Uri(&mut uri).is_ok() {
                        let target_url = string_from_pcwstr(&PCWSTR(uri.0));
                        log::info!("[WebView2 Tab #{}] NewWindowRequested for '{}' -> Opening as tab in Nexora", tab_id, target_url);

                        let ptr = windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW(
                            hwnd_parent,
                            windows::Win32::UI::WindowsAndMessaging::GWLP_USERDATA,
                        ) as *mut crate::browser::host_window::WindowContext;
                        if !ptr.is_null() {
                            let new_tab_id = {
                                let mut tabs = (*ptr).tab_manager.borrow_mut();
                                let id = tabs.create_tab(&target_url, None);
                                tabs.switch_tab(id);
                                id
                            };
                            let tab_count = (*ptr).tab_manager.borrow().tab_count();

                            let (x, y, w, h) = (*ptr).layout.webview_rect();
                            let bounds = RECT { left: x, top: y, right: x + w, bottom: y + h };

                            if let Some(ref webview_eng) = (*ptr).webview {
                                let _ = webview_eng.create_tab_view(new_tab_id, &target_url, bounds, true);
                                let _ = webview_eng.switch_to_tab(new_tab_id, bounds);
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
            let _ = webview.add_NewWindowRequested(&new_window_handler, &mut new_window_token);
        }

        // 6. ProcessFailed (Crash Recovery Hook)
        let crash_controller = std::sync::Arc::new(std::sync::Mutex::new(crate::engine::CrashRecoveryController::default()));
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

                    let should_reload = if let Ok(mut ctrl) = crash_controller.lock() {
                        matches!(ctrl.handle_failure(&uri_str, &format!("{:?}", kind)), crate::engine::CrashRecoveryAction::SilentReload(_))
                    } else {
                        false
                    };

                    if should_reload {
                        if let Some(ref s) = sender {
                            let _ = s.Reload();
                        }
                    }
                }
            }
            Ok(())
        }));
        let mut failed_token = EventRegistrationToken::default();
        unsafe {
            let _ = webview.add_ProcessFailed(&process_failed_handler, &mut failed_token);
        }

        // 7. ContainsFullScreenElementChanged
        let ctl_for_fs = controller.clone();
        let fs_handler = ContainsFullScreenElementChangedEventHandler::create(Box::new(move |sender, _args| {
            if let Some(sender) = sender {
                unsafe {
                    let mut is_fs = BOOL(0);
                    if sender.ContainsFullScreenElement(&mut is_fs).is_ok() {
                        let entering_fs = is_fs.as_bool();
                        log::info!("[Media Engine Tab #{}] HTML5 Fullscreen changed: {}", tab_id, entering_fs);
                        let ptr = windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW(
                            hwnd_parent,
                            windows::Win32::UI::WindowsAndMessaging::GWLP_USERDATA,
                        ) as *mut crate::browser::host_window::WindowContext;
                        if !ptr.is_null() {
                            if entering_fs {
                                let (_, _, w, h) = (*ptr).fullscreen.enter_fullscreen(hwnd_parent);
                                (*ptr).layout = crate::ui::layout::BrowserLayout::new(w, h);
                                if let Some(ref chrome) = (*ptr).chrome {
                                    chrome.set_omnibox_visible(false);
                                }
                                let rect = RECT { left: 0, top: 0, right: w, bottom: h };
                                let _ = ctl_for_fs.SetBounds(rect);
                                if let Some(ref webview) = (*ptr).webview {
                                    let _ = webview.set_bounds(0, 0, w, h);
                                }
                            } else {
                                let (_, _, w, h) = (*ptr).fullscreen.exit_fullscreen(hwnd_parent);
                                (*ptr).layout = crate::ui::layout::BrowserLayout::new(w, h);
                                if let Some(ref chrome) = (*ptr).chrome {
                                    chrome.set_omnibox_visible(true);
                                    let tab_count = (*ptr).tab_manager.borrow().tab_count();
                                    chrome.layout_controls(&(*ptr).layout, tab_count);
                                }
                                let (wx, wy, ww, wh) = (*ptr).layout.webview_rect();
                                let rect = RECT { left: wx, top: wy, right: wx + ww, bottom: wy + wh };
                                let _ = ctl_for_fs.SetBounds(rect);
                                if let Some(ref webview) = (*ptr).webview {
                                    let _ = webview.set_bounds(wx, wy, ww, wh);
                                }
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
            let _ = webview.add_ContainsFullScreenElementChanged(&fs_handler, &mut fs_token);
        }

        // 8. IsDocumentPlayingAudioChanged
        if let Ok(wv8) = webview.cast::<ICoreWebView2_8>() {
            let audio_handler = IsDocumentPlayingAudioChangedEventHandler::create(Box::new(move |sender, _args| {
                if let Some(sender) = sender {
                    unsafe {
                        if let Ok(wv8_sender) = sender.cast::<ICoreWebView2_8>() {
                            let mut is_audio = BOOL(0);
                            if wv8_sender.IsDocumentPlayingAudio(&mut is_audio).is_ok() {
                                let playing_bool = is_audio.as_bool();
                                log::info!("[WebView2 Tab #{}] Audio playing state changed: {}", tab_id, playing_bool);

                                let ptr = windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW(
                                    hwnd_parent,
                                    windows::Win32::UI::WindowsAndMessaging::GWLP_USERDATA,
                                ) as *mut crate::browser::host_window::WindowContext;
                                if !ptr.is_null() {
                                    let mut tabs = (*ptr).tab_manager.borrow_mut();
                                    tabs.set_audio_playing(tab_id, playing_bool);
                                    drop(tabs);

                                    if playing_bool {
                                        let _ = crate::performance::memory::trim_child_processes();
                                    }
                                    let _ = windows::Win32::Graphics::Gdi::InvalidateRect(hwnd_parent, None, false);
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

        // 9. PermissionRequested
        let perm_handler = PermissionRequestedEventHandler::create(Box::new(move |_sender, args| {
            if let Some(args) = args {
                unsafe {
                    let mut uri = PWSTR::null();
                    let mut kind = COREWEBVIEW2_PERMISSION_KIND::default();
                    if args.Uri(&mut uri).is_ok() && args.PermissionKind(&mut kind).is_ok() {
                        let origin = string_from_pcwstr(&PCWSTR(uri.0));
                        let maybe_pk = match kind {
                            COREWEBVIEW2_PERMISSION_KIND_CAMERA => Some(crate::privacy::PermissionKind::Camera),
                            COREWEBVIEW2_PERMISSION_KIND_MICROPHONE => Some(crate::privacy::PermissionKind::Microphone),
                            COREWEBVIEW2_PERMISSION_KIND_GEOLOCATION => Some(crate::privacy::PermissionKind::Geolocation),
                            COREWEBVIEW2_PERMISSION_KIND_NOTIFICATIONS => Some(crate::privacy::PermissionKind::Notifications),
                            COREWEBVIEW2_PERMISSION_KIND_CLIPBOARD_READ => Some(crate::privacy::PermissionKind::ClipboardRead),
                            COREWEBVIEW2_PERMISSION_KIND_AUTOPLAY => Some(crate::privacy::PermissionKind::Autoplay),
                            _ => None,
                        };

                        if let Some(perm_type) = maybe_pk {
                            let default_state = match perm_type {
                                crate::privacy::PermissionKind::Autoplay => crate::privacy::PermissionState::Allow,
                                crate::privacy::PermissionKind::Notifications => crate::privacy::PermissionState::Block,
                                _ => crate::privacy::PermissionState::Prompt,
                            };

                            let state = if let Ok(paths) = crate::app::application::ApplicationPaths::resolve() {
                                let db_path = paths.data_dir.join("nexora.db");
                                if let Ok(conn) = crate::storage::SqliteConnection::open(&db_path) {
                                    let mgr = crate::privacy::PermissionsManager::new(std::sync::Arc::new(conn));
                                    mgr.get_permission(&origin, perm_type).unwrap_or(default_state)
                                } else {
                                    default_state
                                }
                            } else {
                                default_state
                            };

                            match state {
                                crate::privacy::permissions::PermissionState::Allow => {
                                    let _ = args.SetState(COREWEBVIEW2_PERMISSION_STATE_ALLOW);
                                }
                                crate::privacy::permissions::PermissionState::Block => {
                                    let _ = args.SetState(COREWEBVIEW2_PERMISSION_STATE_DENY);
                                }
                                crate::privacy::permissions::PermissionState::Prompt => {
                                    let _ = args.SetState(COREWEBVIEW2_PERMISSION_STATE_DEFAULT);
                                }
                            }
                        }
                    }
                }
            }
            Ok(())
        }));
        let mut perm_token = EventRegistrationToken::default();
        unsafe {
            let _ = webview.add_PermissionRequested(&perm_handler, &mut perm_token);
        }

        // 10. WebResourceRequested (High-Speed Ad & Tracker Blocker)
        let env_for_adblock = self.environment.clone();
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
                                log::info!("[AdBlock Intercept Tab #{}] Blocked {:?}: {}", tab_id, category, url_str);
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
            let _ = webview.AddWebResourceRequestedFilter(
                PCWSTR(filter_all.0),
                COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL,
            );
            let _ = webview.add_WebResourceRequested(&adblock_handler, &mut adblock_token);
        }

        // 11. WebMessageReceived (IPC for internal pages)
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
                        log::info!("[WebView2 IPC Tab #{}] Received WebMessage: {}", tab_id, msg_str);
                        Self::handle_ipc_message(hwnd_parent, tab_id, sender.as_ref(), &msg_str);
                    }
                }
            }
            Ok(())
        }));
        let mut msg_token = EventRegistrationToken::default();
        unsafe {
            let _ = webview.add_WebMessageReceived(&web_message_handler, &mut msg_token);
        }

        Ok(())
    }

    fn handle_ipc_message(hwnd: HWND, tab_id: usize, sender: Option<&ICoreWebView2>, msg_str: &str) {
        #[derive(serde::Deserialize)]
        struct IpcPayload {
            action: String,
            url: Option<String>,
            title: Option<String>,
            id: Option<i64>,
            key: Option<String>,
            value: Option<serde_json::Value>,
        }

        let parsed: std::result::Result<IpcPayload, _> = serde_json::from_str(msg_str);
        match parsed {
            Ok(payload) => match payload.action.as_str() {
                "navigate" => {
                    if let Some(url) = payload.url {
                        log::info!("[IPC Action Tab #{}] Navigating to '{}'", tab_id, url);
                        let ptr = unsafe {
                            windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW(
                                hwnd,
                                windows::Win32::UI::WindowsAndMessaging::GWLP_USERDATA,
                            ) as *mut crate::browser::host_window::WindowContext
                        };
                        if !ptr.is_null() {
                            unsafe {
                                let is_active = (*ptr).tab_manager.borrow().active_tab_id() == Some(tab_id);
                                if is_active {
                                    if let Some(ref chrome) = (*ptr).chrome {
                                        chrome.set_omnibox_text(&url);
                                    }
                                }
                                let mut tabs = (*ptr).tab_manager.borrow_mut();
                                if let Some(tab) = tabs.get_tab_mut(tab_id) {
                                    tab.url = url.clone();
                                    tab.push_history(&url);
                                }
                                drop(tabs);
                                let _ = windows::Win32::Graphics::Gdi::InvalidateRect(hwnd, None, false);

                                if let Some(ref webview) = (*ptr).webview {
                                    let _ = webview.navigate_tab(tab_id, &url);
                                    return;
                                }
                            }
                        }

                        if let Some(s) = sender {
                            unsafe {
                                let wide = pwstr_from_str(&url);
                                let _ = s.Navigate(PCWSTR(wide.0));
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
                            let _ = conn.execute("PRAGMA wal_checkpoint(TRUNCATE);");
                        }
                    }
                }
                "clear_cache" => {
                    log::info!("[IPC Action] Clearing browser cache...");
                    if let Ok(paths) = crate::app::application::ApplicationPaths::resolve() {
                        let _ = std::fs::remove_dir_all(&paths.engine_data_dir);
                        let _ = std::fs::create_dir_all(&paths.engine_data_dir);
                    }
                }
                "reset_defaults" => {
                    log::info!("[IPC Action] Resetting configuration to defaults...");
                    if let Ok(paths) = crate::app::application::ApplicationPaths::resolve() {
                        let default_cfg = crate::app::config::NexoraConfig::default();
                        let _ = default_cfg.save(&paths.app_data_dir);
                    }
                }
                other => {
                    log::warn!("[WebView2 IPC] Unhandled action: {}", other);
                }
            },
            Err(e) => {
                log::error!("[WebView2 IPC] Failed to parse message JSON: {} (raw: {})", e, msg_str);
            }
        }
    }

    fn apply_and_save_setting(hwnd: HWND, key: &str, val: &serde_json::Value) {
        log::info!("[Settings IPC] Applying setting '{}' = {:?}", key, val);
        let paths = match crate::app::application::ApplicationPaths::resolve() {
            Ok(p) => p,
            Err(e) => {
                log::error!("[Settings IPC] Could not resolve paths: {}", e);
                return;
            }
        };

        let mut config = crate::app::config::NexoraConfig::load_or_default(&paths.app_data_dir);

        match key {
            "homepage" => {
                if let Some(s) = val.as_str() {
                    config.homepage = s.to_string();
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
                        "light" => crate::app::config::ThemeMode::Light,
                        "system" => crate::app::config::ThemeMode::System,
                        _ => crate::app::config::ThemeMode::Dark,
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
                    }
                    "default_search_engine" => {
                        (*ptr).default_search_engine = config.default_search_engine.clone();
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
        WebViewEngine::handle_ipc_message(dummy_hwnd, 1, None, msg);

        let paths = crate::app::application::ApplicationPaths::resolve().expect("paths");
        let config = crate::app::config::NexoraConfig::load_or_default(&paths.app_data_dir);
        assert_eq!(config.homepage, "https://duckduckgo.com");

        // Test theme setting
        let msg_theme = r#"{"action":"save_setting","key":"theme","value":"dark"}"#;
        WebViewEngine::handle_ipc_message(dummy_hwnd, 1, None, msg_theme);
        let config2 = crate::app::config::NexoraConfig::load_or_default(&paths.app_data_dir);
        assert_eq!(config2.theme, crate::app::config::ThemeMode::Dark);

        // Test timeout setting
        let msg_timeout = r#"{"action":"save_setting","key":"suspension_timeout_minutes","value":25}"#;
        WebViewEngine::handle_ipc_message(dummy_hwnd, 1, None, msg_timeout);
        let config3 = crate::app::config::NexoraConfig::load_or_default(&paths.app_data_dir);
        assert_eq!(config3.suspension_timeout_minutes, 25);

        // Reset to default
        let reset_msg = r#"{"action":"reset_defaults"}"#;
        WebViewEngine::handle_ipc_message(dummy_hwnd, 1, None, reset_msg);
        let config_def = crate::app::config::NexoraConfig::load_or_default(&paths.app_data_dir);
        assert_eq!(config_def.homepage, "nexora://home");
        assert_eq!(config_def.suspension_timeout_minutes, 10);
    }
}
