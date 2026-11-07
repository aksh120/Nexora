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
}

impl Default for NexoraConfig {
    fn default() -> Self {
        Self {
            homepage: "https://www.google.com".to_string(),
            default_search_engine: "Google".to_string(),
            theme: ThemeMode::System,
            hardware_acceleration: true,
            smart_suspension: true,
            suspension_timeout_minutes: 10,
            disk_cache_limit_mb: 256,
            media_cache_limit_mb: 128,
            ad_block_enabled: true,
            tracker_protection_enabled: true,
        }
    }
}

impl NexoraConfig {
    pub fn load_or_default(config_dir: &Path) -> Self {
        let config_file = config_dir.join("config.json");
        if config_file.exists() {
            if let Ok(content) = fs::read_to_string(&config_file) {
                if let Ok(config) = serde_json::from_str(&content) {
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
        assert_eq!(config.homepage, "https://www.google.com");
        assert_eq!(config.default_search_engine, "Google");
        assert!(config.hardware_acceleration);
        assert!(config.smart_suspension);
        assert_eq!(config.suspension_timeout_minutes, 10);
        assert_eq!(config.disk_cache_limit_mb, 256);
        assert_eq!(config.media_cache_limit_mb, 128);
        assert!(config.ad_block_enabled);
        assert!(config.tracker_protection_enabled);
    }

    #[test]
    fn test_config_serialization() {
        let config = NexoraConfig::default();
        let json = serde_json::to_string(&config).expect("serialize config");
        let deserialized: NexoraConfig = serde_json::from_str(&json).expect("deserialize config");
        assert_eq!(config.homepage, deserialized.homepage);
        assert_eq!(config.theme, deserialized.theme);
    }
}

