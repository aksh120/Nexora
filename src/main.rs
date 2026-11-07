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
use std::sync::Arc;
use crate::app::Application;
use crate::browser::HostWindow;
use crate::engine::WebViewEngine;
use crate::storage::{BookmarkManager, CacheManager, HistoryManager, SessionManager, SqliteConnection};
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

    // 2. Initialize Persistent Storage (SQLite via native winsqlite3.dll)
    let db_path = app.paths.data_dir.join("nexora.db");
    let db = match SqliteConnection::open(&db_path) {
        Ok(conn) => {
            if let Err(err) = conn.migrate() {
                log::error!("Database migration failed: {}", err);
            }
            Arc::new(conn)
        }
        Err(err) => {
            log::error!("Failed to open database at {:?}: {}. Falling back to in-memory.", db_path, err);
            let in_mem = SqliteConnection::open_in_memory().expect("In-memory SQLite failed");
            let _ = in_mem.migrate();
            Arc::new(in_mem)
        }
    };

    let history_mgr = Arc::new(HistoryManager::new(db.clone()));
    let bookmark_mgr = Arc::new(BookmarkManager::new(db.clone()));
    let session_mgr = Arc::new(SessionManager::new(db.clone()));
    let _permissions_mgr = Arc::new(crate::privacy::PermissionsManager::new(db.clone()));
    let adblock_engine = Arc::new(crate::privacy::AdBlockEngine::new(
        app.config.ad_block_enabled,
        app.config.tracker_protection_enabled,
    ));
    log::info!(
        "Privacy Guard Active: AdBlock={}, TrackerProtection={}, Loaded Rules={}",
        adblock_engine.enabled,
        adblock_engine.block_trackers,
        adblock_engine.total_rules()
    );

    // Audit engine switches for strict zero-telemetry compliance
    let gpu_caps = crate::media::GpuCapabilities::detect(app.config.hardware_acceleration);
    let browser_switches = gpu_caps.build_browser_switches().join(" ");
    let audit_result = crate::privacy::ZeroTelemetryAuditor::audit_flags(&browser_switches);
    log::info!(
        "Zero-Telemetry Audit Status: Compliant={}, Active Kill-Switches={}",
        audit_result.compliant,
        audit_result.verified_flags.len()
    );

    // Seed default streaming bookmarks if bookmark store is empty
    if let Ok(existing) = bookmark_mgr.get_all() {
        if existing.is_empty() {
            log::info!("Seeding default streaming bookmarks...");
            let _ = bookmark_mgr.add_bookmark("YouTube", "https://www.youtube.com", None);
            let _ = bookmark_mgr.add_bookmark("Netflix", "https://www.netflix.com", None);
            let _ = bookmark_mgr.add_bookmark("Prime Video", "https://www.primevideo.com", None);
            let _ = bookmark_mgr.add_bookmark("JioHotstar", "https://www.hotstar.com", None);
            let _ = bookmark_mgr.add_bookmark("Twitch", "https://www.twitch.tv", None);
            let _ = bookmark_mgr.add_bookmark("Spotify", "https://open.spotify.com", None);
        }
    }

    // 3. Enforce Engine Disk Cache Boundary
    if let Err(err) = CacheManager::enforce_boundary(&app.paths.engine_data_dir, app.config.disk_cache_limit_mb) {
        log::warn!("Cache boundary enforcement warning: {}", err);
    }

    // 4. Resolve Theme
    let theme_mode = match app.config.theme {
        crate::app::config::ThemeMode::Light => ThemeMode::Light,
        crate::app::config::ThemeMode::Dark => ThemeMode::Dark,
        crate::app::config::ThemeMode::System => ThemeMode::System,
    };
    let theme = ThemePalette::from_mode(theme_mode);
    log::info!("Active Theme: {:?} (dark: {})", app.config.theme, theme.is_dark);

    // 5. Initialize Multi-Tab State Machine via Lazy Session Restoration Pipeline
    let tab_manager = Rc::new(RefCell::new(TabManager::new()));
    let initial_url = if let Ok(saved_session) = session_mgr.load_session() {
        if !saved_session.tabs.is_empty() {
            log::info!("Executing Lazy Session Restoration Pipeline for {} tabs...", saved_session.tabs.len());
            let plan = crate::performance::LazyRestorePipeline::restore_session(
                &saved_session,
                &app.config.homepage,
                &mut tab_manager.borrow_mut(),
            );
            plan.active_url
        } else {
            let initial_tab_id = tab_manager.borrow_mut().create_tab(&app.config.homepage, Some("Nexora Home"));
            log::info!("Created initial browser tab #{}", initial_tab_id);
            app.config.homepage.clone()
        }
    } else {
        let initial_tab_id = tab_manager.borrow_mut().create_tab(&app.config.homepage, Some("Nexora Home"));
        log::info!("Created initial browser tab #{}", initial_tab_id);
        app.config.homepage.clone()
    };

    // 6. Create the native Win32 Host Window
    let host_window = match HostWindow::new(
        "Nexora",
        1280,
        800,
        theme,
        &app.config.homepage,
        &app.config.default_search_engine,
        tab_manager.clone(),
        app.config.smart_suspension,
        app.config.suspension_timeout_minutes,
    ) {
        Ok(window) => Rc::new(window),
        Err(err) => {
            log::error!("Failed to create Win32 host window: {}", err);
            return;
        }
    };

    // 7. Initialize Native Chrome (Tab bar, Navigation buttons, Omnibox)
    if let Err(err) = host_window.init_chrome() {
        log::error!("Failed to initialize browser chrome: {}", err);
        return;
    }
    host_window.update_omnibox_url(&initial_url);

    // 8. Initialize WebView2 Engine attached to the Host Window
    let webview = match WebViewEngine::new(
        host_window.hwnd(),
        &app.paths.engine_data_dir,
        &initial_url,
    ) {
        Ok(engine) => Rc::new(engine),
        Err(err) => {
            log::error!("Failed to initialize WebView2 engine: {}", err);
            return;
        }
    };

    // Record initial navigation in history
    let _ = history_mgr.add_visit(&initial_url, "Nexora Home");

    // 9. Associate WebView2 with HostWindow
    host_window.set_webview(webview);

    // 10. Display the Host Window
    host_window.show();
    log::info!("Nexora main window shown. Entering message loop.");

    // 11. Enter the native Win32 message loop
    host_window.run_message_loop();

    // 12. Clean Shutdown & Session Persistence
    log::info!("Persisting active session on application exit...");
    let tabs = tab_manager.borrow();
    let _ = session_mgr.save_session(tabs.tabs(), tabs.active_tab_id());
    drop(tabs);

    log::info!("Nexora Browser exited cleanly.");
}
