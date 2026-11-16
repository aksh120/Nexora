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

/// Browser & Streaming Engine Architecture Option
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StreamingEngineMode {
    /// Option 0: Standard Multi-Controller Engine (Default):
    /// WebView2 Evergreen + Direct3D 11 NVDEC GPU decode offload + Widevine DRM.
    /// Full web functionality (comments, live chat, recommendations, Google account login).
    /// Base footprint: ~120 – 220 MB RAM (< 1 MB when suspended).
    #[serde(rename = "standard")]
    Standard,
    /// Option 1: Ultralight (Lightweight WebKit):
    /// Pure C++ lightweight WebKit fork with Direct3D 11 / OpenGL render surface.
    /// Base footprint: ~15 – 35 MB RAM (+35 – 50 MB binary footprint).
    /// Web Compatibility: ~65% (Basic HTML5/CSS, zero DRM, breaks heavy Google/Discord SPAs).
    #[serde(rename = "ultralight")]
    Ultralight,
    /// Option 2: Servo (Pure Rust Web Engine):
    /// Highly parallel browser engine written in pure Rust with WebRender.
    /// Base footprint: ~25 – 45 MB RAM (+40 – 60 MB binary footprint).
    /// Web Compatibility: ~45% (Experimental CSS/DOM, pure memory-safe Rust core).
    #[serde(rename = "servo")]
    Servo,
}

impl Default for StreamingEngineMode {
    fn default() -> Self {
        Self::Standard
    }
}

impl StreamingEngineMode {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Standard => "Standard Multi-Controller Engine (Default)",
            Self::Ultralight => "Option 1: Ultralight (Lightweight WebKit)",
            Self::Servo => "Option 2: Servo (Pure Rust Web Engine)",
        }
    }

    pub fn ram_estimate(&self) -> &'static str {
        match self {
            Self::Standard => "~120 – 220 MB RAM",
            Self::Ultralight => "~15 – 35 MB RAM",
            Self::Servo => "~25 – 45 MB RAM",
        }
    }

    pub fn compatibility_summary(&self) -> &'static str {
        match self {
            Self::Standard => "100% (Full Web, V8, Widevine DRM, Direct3D 11 NVDEC)",
            Self::Ultralight => "~65% (Basic HTML5/CSS, No DRM, lightweight reading)",
            Self::Servo => "~45% (Experimental CSS/DOM, WebRender, pure Rust core)",
        }
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
        config.streaming_engine_mode = StreamingEngineMode::Ultralight;
        let json = serde_json::to_string(&config).expect("serialize config");
        let deserialized: NexoraConfig = serde_json::from_str(&json).expect("deserialize config");
        assert_eq!(config.homepage, deserialized.homepage);
        assert_eq!(config.theme, deserialized.theme);
        assert_eq!(deserialized.streaming_engine_mode, StreamingEngineMode::Ultralight);

        let mut config_servo = NexoraConfig::default();
        config_servo.streaming_engine_mode = StreamingEngineMode::Servo;
        let json_servo = serde_json::to_string(&config_servo).expect("serialize config servo");
        let deserialized_servo: NexoraConfig = serde_json::from_str(&json_servo).expect("deserialize config servo");
        assert_eq!(deserialized_servo.streaming_engine_mode, StreamingEngineMode::Servo);
    }
}

