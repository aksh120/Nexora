#![windows_subsystem = "windows"]
#![allow(dead_code)]

mod app;
mod browser;
mod core;
mod diagnostics;
mod engine;
mod media;
mod navigation;
mod performance;
mod privacy;
mod settings;
mod storage;
mod tabs;
mod ui;

use std::cell::RefCell;
use std::rc::Rc;
use crate::app::Application;
use crate::browser::HostWindow;
use crate::engine::WebViewEngine;
use crate::tabs::TabManager;
use crate::ui::theme::{ThemeMode, ThemePalette};

fn main() {
    // 1. Initialize core application lifecycle, logging, and directories
    let app = match Application::init() {
        Ok(app) => app,
        Err(err) => {
            eprintln!("Fatal error initializing Nexora Browser: {}", err);
            return;
        }
    };

    log::info!("Nexora Browser v{} successfully booted.", env!("CARGO_PKG_VERSION"));
    log::info!("System Config Summary:");
    log::info!("  Homepage:              {}", app.config.homepage);
    log::info!("  Default Search Engine: {}", app.config.default_search_engine);
    log::info!("  Hardware Acceleration: {}", app.config.hardware_acceleration);
    log::info!("  Smart Tab Suspension:  {}", app.config.smart_suspension);
    log::info!("  Suspension Timeout:    {} minutes", app.config.suspension_timeout_minutes);
    log::info!("  Disk Cache Limit:      {} MB", app.config.disk_cache_limit_mb);
    log::info!("  Media Cache Limit:     {} MB", app.config.media_cache_limit_mb);
    log::info!("  Ad Blocker:            {}", app.config.ad_block_enabled);
    log::info!("  Tracker Protection:    {}", app.config.tracker_protection_enabled);

    // 2. Resolve Theme
    let theme_mode = match app.config.theme {
        crate::app::config::ThemeMode::Light => ThemeMode::Light,
        crate::app::config::ThemeMode::Dark => ThemeMode::Dark,
        crate::app::config::ThemeMode::System => ThemeMode::System,
    };
    let theme = ThemePalette::from_mode(theme_mode);
    log::info!("Active Theme: {:?} (dark: {})", app.config.theme, theme.is_dark);

    // 3. Initialize Multi-Tab State Machine
    let tab_manager = Rc::new(RefCell::new(TabManager::new()));
    let initial_tab_id = tab_manager.borrow_mut().create_tab(&app.config.homepage, Some("Nexora Home"));
    log::info!("Created initial browser tab #{}", initial_tab_id);

    // 4. Create the native Win32 Host Window
    let host_window = match HostWindow::new(
        "Nexora",
        1280,
        800,
        theme,
        &app.config.homepage,
        &app.config.default_search_engine,
        tab_manager.clone(),
    ) {
        Ok(window) => Rc::new(window),
        Err(err) => {
            log::error!("Failed to create Win32 host window: {}", err);
            return;
        }
    };

    // 5. Initialize Native Chrome (Tab bar, Navigation buttons, Omnibox)
    if let Err(err) = host_window.init_chrome() {
        log::error!("Failed to initialize browser chrome: {}", err);
        return;
    }
    host_window.update_omnibox_url(&app.config.homepage);

    // 6. Initialize WebView2 Engine attached to the Host Window
    let webview = match WebViewEngine::new(
        host_window.hwnd(),
        &app.paths.engine_data_dir,
        &app.config.homepage,
    ) {
        Ok(engine) => Rc::new(engine),
        Err(err) => {
            log::error!("Failed to initialize WebView2 engine: {}", err);
            return;
        }
    };

    // 7. Associate WebView2 with HostWindow
    host_window.set_webview(webview);

    // 8. Display the Host Window
    host_window.show();
    log::info!("Nexora main window shown. Entering message loop.");

    // 9. Enter the native Win32 message loop
    host_window.run_message_loop();

    log::info!("Nexora Browser exited cleanly.");
}
