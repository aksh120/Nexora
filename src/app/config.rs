use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use crate::core::error::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeMode {
    System,
    Dark,
    Light,
}

impl Default for ThemeMode {
    fn default() -> Self {
        Self::System
    }
}

/// Video Streaming Engine Architecture Option
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StreamingEngineMode {
    /// Standard Multi-Controller Engine (Default):
    /// WebView2 Evergreen + Direct3D 11 NVDEC GPU decode offload + Widevine DRM.
    /// Full web functionality (comments, live chat, recommendations, account features).
    #[serde(rename = "standard")]
    Standard,
    /// Option 1: Native D3D11/WMF Video Player Engine (< 60 MB RAM):
    /// Stream extraction (InnerTube/DASH) feeds directly into Windows Media Foundation (WMF)
    /// and Direct3D 11 NVDEC swapchain. Zero Chromium DOM, zero V8 JavaScript engine.
    /// ~35–55 MB total host RAM, ~1–3% CPU on 1080p60.
    #[serde(rename = "native_d3d11")]
    NativeD3D11,
    /// Option 2: Hybrid Stealth Core (Dual Engine):
    /// Standard web browsing with seamless intercept to Native D3D11 player on YouTube video playback,
    /// placing the heavy webview tab into dormant suspension (~50–65 MB playback RAM).
    #[serde(rename = "hybrid_stealth")]
    HybridStealth,
}

impl Default for StreamingEngineMode {
    fn default() -> Self {
        Self::Standard
    }
}

/// ANGLE Hardware Graphics Backend
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AngleBackend {
    #[serde(rename = "d3d11")]
    D3D11,
    #[serde(rename = "d3d11on12")]
    D3D11on12,
    #[serde(rename = "d3d9")]
    D3D9,
    #[serde(rename = "vulkan")]
    Vulkan,
    #[serde(rename = "warp")]
    Warp,
    #[serde(rename = "default")]
    Default,
}

impl Default for AngleBackend {
    fn default() -> Self {
        Self::D3D11
    }
}

impl AngleBackend {
    pub fn as_chromium_switch(&self) -> Option<&'static str> {
        match self {
            Self::D3D11 => Some("--use-angle=d3d11"),
            Self::D3D11on12 => Some("--use-angle=d3d11on12"),
            Self::D3D9 => Some("--use-angle=d3d9"),
            Self::Vulkan => Some("--use-angle=vulkan"),
            Self::Warp => Some("--use-angle=warp"),
            Self::Default => None,
        }
    }
}

/// Automated Memory Governor Configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryGovernorConfig {
    pub enabled: bool,
    pub pressure_threshold_percent: u32,
    pub max_tab_idle_seconds: u64,
    pub auto_trim_working_set: bool,
}

impl Default for MemoryGovernorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            pressure_threshold_percent: 15,
            max_tab_idle_seconds: 300,
            auto_trim_working_set: true,
        }
    }
}

fn default_raster_threads() -> u32 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NexoraConfig {
    pub homepage: String,
    pub default_search_engine: String,
    pub theme: ThemeMode,
    pub hardware_acceleration: bool,
    pub smart_suspension: bool,
    pub suspension_timeout_minutes: u32,
    pub disk_cache_limit_mb: u32,
    pub media_cache_limit_mb: u32,
    pub ad_block_enabled: bool,
    pub tracker_protection_enabled: bool,
    #[serde(default)]
    pub streaming_engine_mode: StreamingEngineMode,
    #[serde(default)]
    pub vertical_tabs: bool,
    #[serde(default)]
    pub vertical_tabs_collapsed: bool,
    #[serde(default)]
    pub memory_governor: MemoryGovernorConfig,
    #[serde(default)]
    pub angle_backend: AngleBackend,
    #[serde(default = "default_raster_threads")]
    pub raster_threads: u32,
}

impl Default for NexoraConfig {
    fn default() -> Self {
        Self {
            homepage: "nexora://home".to_string(),
            default_search_engine: "Google".to_string(),
            theme: ThemeMode::System,
            hardware_acceleration: true,
            smart_suspension: true,
            suspension_timeout_minutes: 10,
            disk_cache_limit_mb: 256,
            media_cache_limit_mb: 128,
            ad_block_enabled: true,
            tracker_protection_enabled: true,
            streaming_engine_mode: StreamingEngineMode::Standard,
            vertical_tabs: false,
            vertical_tabs_collapsed: false,
            memory_governor: MemoryGovernorConfig::default(),
            angle_backend: AngleBackend::default(),
            raster_threads: 1,
        }
    }
}

impl NexoraConfig {
    pub fn load_or_default(config_dir: &Path) -> Self {
        let config_file = config_dir.join("config.json");
        if config_file.exists() {
            if let Ok(content) = fs::read_to_string(&config_file) {
                if let Ok(mut config) = serde_json::from_str::<NexoraConfig>(&content) {
                    // Automatically upgrade legacy google.com default to the dedicated Nexora homepage
                    if config.homepage == "https://www.google.com" || config.homepage.is_empty() {
                        config.homepage = "nexora://home".to_string();
                        let _ = config.save(config_dir);
                        log::info!("Migrated legacy homepage to nexora://home");
                    }
                    log::info!("Loaded configuration from {:?}", config_file);
                    return config;
                }
            }
        }
        let config = Self::default();
        let _ = config.save(config_dir);
        log::info!("Initialized default configuration at {:?}", config_file);
        config
    }

    pub fn save(&self, config_dir: &Path) -> Result<()> {
        fs::create_dir_all(config_dir)?;
        let config_file = config_dir.join("config.json");
        let content = serde_json::to_string_pretty(self)?;
        fs::write(config_file, content)?;
        log::debug!("Saved configuration to {:?}", config_dir);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = NexoraConfig::default();
        assert_eq!(config.homepage, "nexora://home");
        assert_eq!(config.default_search_engine, "Google");
        assert!(config.hardware_acceleration);
        assert!(config.smart_suspension);
        assert_eq!(config.suspension_timeout_minutes, 10);
        assert_eq!(config.disk_cache_limit_mb, 256);
        assert_eq!(config.media_cache_limit_mb, 128);
        assert!(config.ad_block_enabled);
        assert!(config.tracker_protection_enabled);
        assert_eq!(config.streaming_engine_mode, StreamingEngineMode::Standard);
    }

    #[test]
    fn test_config_serialization() {
        let mut config = NexoraConfig::default();
        config.streaming_engine_mode = StreamingEngineMode::NativeD3D11;
        let json = serde_json::to_string(&config).expect("serialize config");
        let deserialized: NexoraConfig = serde_json::from_str(&json).expect("deserialize config");
        assert_eq!(config.homepage, deserialized.homepage);
        assert_eq!(config.theme, deserialized.theme);
        assert_eq!(deserialized.streaming_engine_mode, StreamingEngineMode::NativeD3D11);
    }
}

