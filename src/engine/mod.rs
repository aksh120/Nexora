// Embedded WebView2 Engine Integration
pub mod webview;
pub mod crash_handler;

pub use webview::WebViewEngine;
pub use crash_handler::{CrashRecoveryAction, CrashRecoveryController};
