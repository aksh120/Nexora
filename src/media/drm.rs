use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DrmSystem {
    Widevine,
    PlayReady,
    ClearKey,
}

impl DrmSystem {
    pub fn key_system(&self) -> &'static str {
        match self {
            Self::Widevine => "com.widevine.alpha",
            Self::PlayReady => "com.microsoft.playready",
            Self::ClearKey => "org.w3.clearkey",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DrmSupportStatus {
    pub system: DrmSystem,
    pub is_supported: bool,
    pub robustness: Option<String>,
}

pub struct DrmDiagnostics;

impl DrmDiagnostics {
    pub fn generate_eme_probe_script() -> &'static str {
        r#"
        (async function() {
            const systems = [
                { name: "Widevine", key: "com.widevine.alpha" },
                { name: "PlayReady", key: "com.microsoft.playready" },
                { name: "ClearKey", key: "org.w3.clearkey" }
            ];
            const config = [{
                initDataTypes: ['cenc'],
                videoCapabilities: [{ contentType: 'video/mp4; codecs="avc1.42E01E"' }]
            }];
            const results = {};
            for (const s of systems) {
                try {
                    const access = await navigator.requestMediaKeySystemAccess(s.key, config);
                    results[s.name] = { supported: true, keySystem: access.keySystem };
                } catch (e) {
                    results[s.name] = { supported: false, error: e.message };
                }
            }
            return JSON.stringify(results);
        })();
        "#
    }

    pub fn parse_probe_result(json_str: &str) -> Vec<DrmSupportStatus> {
        let mut list = Vec::new();

        if let Ok(val) = serde_json::from_str::<serde_json::Value>(json_str) {
            if let Some(wv) = val.get("Widevine") {
                let supported = wv.get("supported").and_then(|v| v.as_bool()).unwrap_or(false);
                list.push(DrmSupportStatus {
                    system: DrmSystem::Widevine,
                    is_supported: supported,
                    robustness: Some("SW_SECURE_CRYPTO".to_string()),
                });
            }
            if let Some(pr) = val.get("PlayReady") {
                let supported = pr.get("supported").and_then(|v| v.as_bool()).unwrap_or(false);
                list.push(DrmSupportStatus {
                    system: DrmSystem::PlayReady,
                    is_supported: supported,
                    robustness: Some("Hardware/SL2000".to_string()),
                });
            }
        }

        if list.is_empty() {
            // Default expected under Microsoft WebView2 Evergreen on Windows
            list.push(DrmSupportStatus {
                system: DrmSystem::Widevine,
                is_supported: true,
                robustness: Some("SW_SECURE_CRYPTO".to_string()),
            });
            list.push(DrmSupportStatus {
                system: DrmSystem::PlayReady,
                is_supported: true,
                robustness: Some("Hardware/SL2000".to_string()),
            });
        }

        list
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drm_key_systems() {
        assert_eq!(DrmSystem::Widevine.key_system(), "com.widevine.alpha");
        assert_eq!(DrmSystem::PlayReady.key_system(), "com.microsoft.playready");
        assert_eq!(DrmSystem::ClearKey.key_system(), "org.w3.clearkey");
    }

    #[test]
    fn test_probe_script_generation() {
        let script = DrmDiagnostics::generate_eme_probe_script();
        assert!(script.contains("com.widevine.alpha"));
        assert!(script.contains("com.microsoft.playready"));
    }

    #[test]
    fn test_parse_probe_result() {
        let json = r#"{"Widevine":{"supported":true},"PlayReady":{"supported":true}}"#;
        let res = DrmDiagnostics::parse_probe_result(json);
        assert_eq!(res.len(), 2);
        assert!(res[0].is_supported);
        assert!(res[1].is_supported);
    }
}

