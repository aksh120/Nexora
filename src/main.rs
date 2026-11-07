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

use crate::app::Application;

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

    log::info!("Phase 1: Foundation successfully verified.");
}
