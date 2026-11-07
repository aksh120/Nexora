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

use std::rc::Rc;
use crate::app::Application;
use crate::browser::HostWindow;
use crate::engine::WebViewEngine;

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

    // 2. Create the native Win32 Host Window
    let host_window = match HostWindow::new("Nexora", 1280, 800) {
        Ok(window) => window,
        Err(err) => {
            log::error!("Failed to create Win32 host window: {}", err);
            return;
        }
    };

    // 3. Initialize WebView2 Engine attached to the Host Window
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

    // 4. Configure initial bounds and resize handler
    let (left, top, width, height) = host_window.get_client_bounds();
    let _ = webview.set_bounds(left, top, width, height);

    let webview_clone = webview.clone();
    host_window.set_resize_callback(move |x, y, w, h| {
        let _ = webview_clone.set_bounds(x, y, w, h);
    });

    // 5. Display the Host Window
    host_window.show();
    log::info!("Nexora main window shown. Entering message loop.");

    // 6. Enter the native Win32 message loop
    host_window.run_message_loop();

    log::info!("Nexora Browser exited cleanly.");
}
