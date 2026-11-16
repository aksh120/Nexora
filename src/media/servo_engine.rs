//! Servo Pure Rust Web Engine Architecture
//!
//! Provides specifications, memory profiles, capabilities, and runtime bridge
//! for the Servo parallel browser engine written in pure Rust with WebRender.

use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServoCapabilities {
    pub engine_family: &'static str,
    pub core_version: &'static str,
    pub ram_footprint_mb: &'static str,
    pub binary_overhead_mb: &'static str,
    pub compatibility_percent: u32,
    pub webrender_active: bool,
    pub supports_widevine_drm: bool,
    pub memory_safe_rust_core: bool,
    pub primary_use_case: &'static str,
}

impl Default for ServoCapabilities {
    fn default() -> Self {
        Self {
            engine_family: "Servo (Pure Rust Web Engine)",
            core_version: "2026.1",
            ram_footprint_mb: "~25 – 45 MB",
            binary_overhead_mb: "+40 – 60 MB",
            compatibility_percent: 45,
            webrender_active: true,
            supports_widevine_drm: false,
            memory_safe_rust_core: true,
            primary_use_case: "Memory-safe parallel layout, experimental web rendering, and research",
        }
    }
}

pub struct ServoEngine;

impl ServoEngine {
    /// Returns static capabilities and performance specifications of the Servo backend.
    pub fn capabilities() -> ServoCapabilities {
        ServoCapabilities::default()
    }

    /// Verifies whether the native Servo standalone binary or library is present in the application directory.
    pub fn is_runtime_installed(app_dir: &Path) -> bool {
        let servo_exe = app_dir.join("servo.exe");
        let servo_dll = app_dir.join("libservo.dll");
        servo_exe.exists() || servo_dll.exists()
    }

    /// Generates informational diagnostic status for settings and diagnostics UI.
    pub fn status_summary(app_dir: &Path) -> &'static str {
        if Self::is_runtime_installed(app_dir) {
            "Installed (Servo WebRender Active)"
        } else {
            "Configured (Awaiting Standalone Servo Package)"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_servo_capabilities() {
        let caps = ServoEngine::capabilities();
        assert_eq!(caps.engine_family, "Servo (Pure Rust Web Engine)");
        assert_eq!(caps.compatibility_percent, 45);
        assert!(!caps.supports_widevine_drm);
        assert!(caps.memory_safe_rust_core);
        assert!(caps.webrender_active);
    }

    #[test]
    fn test_servo_runtime_detection() {
        let temp = std::env::temp_dir();
        assert!(!ServoEngine::is_runtime_installed(&temp));
        assert_eq!(
            ServoEngine::status_summary(&temp),
            "Configured (Awaiting Standalone Servo Package)"
        );
    }
}
