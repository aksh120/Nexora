//! Ultralight Lightweight WebKit Engine Architecture
//!
//! Provides specifications, memory profiles, capabilities, and runtime bridge
//! for the Ultralight C++ lightweight WebKit rendering subsystem.

use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UltralightCapabilities {
    pub engine_family: &'static str,
    pub core_version: &'static str,
    pub ram_footprint_mb: &'static str,
    pub binary_overhead_mb: &'static str,
    pub compatibility_percent: u32,
    pub d3d11_hardware_surface: bool,
    pub supports_widevine_drm: bool,
    pub supports_complex_spas: bool,
    pub primary_use_case: &'static str,
}

impl Default for UltralightCapabilities {
    fn default() -> Self {
        Self {
            engine_family: "Ultralight (Lightweight WebKit)",
            core_version: "1.4.0",
            ram_footprint_mb: "~15 – 35 MB",
            binary_overhead_mb: "+35 – 50 MB",
            compatibility_percent: 65,
            d3d11_hardware_surface: true,
            supports_widevine_drm: false,
            supports_complex_spas: false,
            primary_use_case: "Ultra-lean document browsing, internal dashboards, and game UI rendering",
        }
    }
}

pub struct UltralightEngine;

impl UltralightEngine {
    /// Returns static capabilities and performance specifications of the Ultralight backend.
    pub fn capabilities() -> UltralightCapabilities {
        UltralightCapabilities::default()
    }

    /// Verifies whether the native Ultralight dynamic libraries (`Ultralight.dll`, `WebCore.dll`)
    /// are installed in the application execution directory.
    pub fn is_runtime_installed(app_dir: &Path) -> bool {
        let core_dll = app_dir.join("Ultralight.dll");
        let webcore_dll = app_dir.join("WebCore.dll");
        core_dll.exists() && webcore_dll.exists()
    }

    /// Generates informational diagnostic status for settings and diagnostics UI.
    pub fn status_summary(app_dir: &Path) -> &'static str {
        if Self::is_runtime_installed(app_dir) {
            "Installed (Native Ultralight DLLs Active)"
        } else {
            "Configured (Awaiting Standalone DLL Package)"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ultralight_capabilities() {
        let caps = UltralightEngine::capabilities();
        assert_eq!(caps.engine_family, "Ultralight (Lightweight WebKit)");
        assert_eq!(caps.compatibility_percent, 65);
        assert!(!caps.supports_widevine_drm);
        assert!(caps.d3d11_hardware_surface);
    }

    #[test]
    fn test_ultralight_runtime_detection() {
        let temp = std::env::temp_dir();
        assert!(!UltralightEngine::is_runtime_installed(&temp));
        assert_eq!(
            UltralightEngine::status_summary(&temp),
            "Configured (Awaiting Standalone DLL Package)"
        );
    }
}
