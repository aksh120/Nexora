//! GitHub Release Detector & Semantic Version Comparator
//!
//! Queries the GitHub Releases API to detect new binary distributions
//! and compares semantic versions without external runtime dependencies.

use crate::core::error::{NexoraError, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseInfo {
    pub tag_name: String,
    pub version: String,
    pub name: String,
    pub body: String,
    pub html_url: String,
    pub download_url: Option<String>,
    pub asset_name: Option<String>,
    pub asset_size: Option<u64>,
}

pub struct UpdateDetector;

impl UpdateDetector {
    /// Checks the specified GitHub repository for releases newer than `current_version`.
    pub fn check_for_updates(
        repo_owner: &str,
        repo_name: &str,
        current_version: &str,
    ) -> Result<Option<ReleaseInfo>> {
        let primary_url = format!(
            "https://api.github.com/repos/{}/{}/releases/latest",
            repo_owner, repo_name
        );
        let fallback_url = format!(
            "https://api.github.com/repos/{}/{}/releases",
            repo_owner, repo_name
        );

        log::info!("[Updater] Checking for updates at: {}", primary_url);

        let json_body = match Self::fetch_http(&primary_url) {
            Ok(body) if !body.contains(r#""message":"Not Found""#) => body,
            _ => {
                log::info!("[Updater] Latest endpoint not found, falling back to: {}", fallback_url);
                Self::fetch_http(&fallback_url)?
            }
        };

        if let Some(release) = Self::parse_release_json(&json_body)? {
            if Self::is_newer_version(current_version, &release.version) {
                log::info!(
                    "[Updater] New version detected: {} (Current: {})",
                    release.version,
                    current_version
                );
                return Ok(Some(release));
            } else {
                log::info!(
                    "[Updater] Up to date (Current: {}, Latest: {})",
                    current_version,
                    release.version
                );
            }
        }

        Ok(None)
    }

    /// Fetches response text using Windows built-in curl.exe with WinINet fallback.
    pub fn fetch_http(url: &str) -> Result<String> {
        // 1. Try Windows built-in curl.exe (present in System32 on Windows 10/11)
        if let Ok(output) = Self::fetch_with_curl(url) {
            return Ok(output);
        }

        // 2. WinINet native fallback
        Self::fetch_with_wininet(url)
    }

    fn fetch_with_curl(url: &str) -> Result<String> {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;

        let output = std::process::Command::new("curl.exe")
            .args([
                "-s",
                "-L",
                "--max-time",
                "15",
                "-H",
                "User-Agent: Nexora-Browser",
                "-H",
                "Accept: application/vnd.github.v3+json",
                url,
            ])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|e| NexoraError::Other(format!("Failed to execute curl.exe: {}", e)))?;

        if !output.status.success() {
            return Err(NexoraError::Other(format!(
                "curl.exe exited with status code: {:?}",
                output.status.code()
            )));
        }

        String::from_utf8(output.stdout)
            .map_err(|e| NexoraError::Other(format!("curl.exe returned invalid UTF-8: {}", e)))
    }

    fn fetch_with_wininet(url: &str) -> Result<String> {
        use windows::core::PCWSTR;
        use windows::Win32::Networking::WinInet::{
            InternetCloseHandle, InternetOpenUrlW, InternetOpenW, InternetReadFile,
            INTERNET_FLAG_NO_CACHE_WRITE, INTERNET_FLAG_RELOAD, INTERNET_FLAG_SECURE,
            INTERNET_OPEN_TYPE_PRECONFIG,
        };

        unsafe {
            let wide_agent: Vec<u16> = "Nexora-Browser".encode_utf16().chain(std::iter::once(0)).collect();
            let h_internet = InternetOpenW(
                PCWSTR(wide_agent.as_ptr()),
                INTERNET_OPEN_TYPE_PRECONFIG.0,
                PCWSTR::null(),
                PCWSTR::null(),
                0,
            );
            if h_internet.is_null() {
                return Err(NexoraError::Other("InternetOpenW failed".into()));
            }

            let wide_url: Vec<u16> = url.encode_utf16().chain(std::iter::once(0)).collect();
            let flags = INTERNET_FLAG_RELOAD | INTERNET_FLAG_NO_CACHE_WRITE | INTERNET_FLAG_SECURE;

            let h_file = InternetOpenUrlW(
                h_internet,
                PCWSTR(wide_url.as_ptr()),
                None,
                flags,
                0,
            );

            if h_file.is_null() {
                let _ = InternetCloseHandle(h_internet);
                return Err(NexoraError::Other(format!("InternetOpenUrlW failed for {}", url)));
            }

            let mut buffer = Vec::new();
            let mut chunk = [0u8; 8192];
            let mut bytes_read: u32 = 0;

            loop {
                let ok = InternetReadFile(
                    h_file,
                    chunk.as_mut_ptr() as *mut _,
                    chunk.len() as u32,
                    &mut bytes_read,
                );
                if ok.is_err() || bytes_read == 0 {
                    break;
                }
                buffer.extend_from_slice(&chunk[..bytes_read as usize]);
            }

            let _ = InternetCloseHandle(h_file);
            let _ = InternetCloseHandle(h_internet);

            String::from_utf8(buffer)
                .map_err(|e| NexoraError::Other(format!("WinInet returned non-UTF8 response: {}", e)))
        }
    }

    /// Parses a GitHub Release JSON object or array of release objects into `ReleaseInfo`.
    pub fn parse_release_json(json_str: &str) -> Result<Option<ReleaseInfo>> {
        let val: serde_json::Value = serde_json::from_str(json_str)
            .map_err(|e| NexoraError::Other(format!("JSON parse error: {}", e)))?;

        let release_obj = if val.is_array() {
            val.as_array().and_then(|arr| arr.first()).cloned()
        } else if val.is_object() {
            Some(val)
        } else {
            None
        };

        let obj = match release_obj {
            Some(o) => o,
            None => return Ok(None),
        };

        let tag_name = obj
            .get("tag_name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        if tag_name.is_empty() {
            return Ok(None);
        }

        let clean_version = tag_name.trim_start_matches('v').trim_start_matches('V').to_string();
        let name = obj
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or(&tag_name)
            .to_string();
        let body = obj
            .get("body")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let html_url = obj
            .get("html_url")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let mut download_url = None;
        let mut asset_name = None;
        let mut asset_size = None;

        if let Some(assets) = obj.get("assets").and_then(|a| a.as_array()) {
            for asset in assets {
                let a_name = asset.get("name").and_then(|n| n.as_str()).unwrap_or("");
                if a_name.ends_with(".exe") {
                    asset_name = Some(a_name.to_string());
                    download_url = asset
                        .get("browser_download_url")
                        .and_then(|u| u.as_str())
                        .map(String::from);
                    asset_size = asset.get("size").and_then(|s| s.as_u64());
                    if a_name.eq_ignore_ascii_case("Nexora.exe") {
                        break;
                    }
                }
            }
        }

        Ok(Some(ReleaseInfo {
            tag_name,
            version: clean_version,
            name,
            body,
            html_url,
            download_url,
            asset_name,
            asset_size,
        }))
    }

    /// Compares two semantic version strings (e.g. "0.1.0" and "0.2.0" or "v0.1.0" and "v0.1.1").
    /// Returns `true` if `candidate` is strictly greater than `current`.
    pub fn is_newer_version(current: &str, candidate: &str) -> bool {
        let cur = Self::parse_semver(current);
        let cand = Self::parse_semver(candidate);

        match (cur, cand) {
            (Some((c_maj, c_min, c_pat)), Some((n_maj, n_min, n_pat))) => {
                if n_maj > c_maj {
                    true
                } else if n_maj == c_maj && n_min > c_min {
                    true
                } else if n_maj == c_maj && n_min == c_min && n_pat > c_pat {
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Parses version components into (major, minor, patch).
    pub fn parse_semver(v: &str) -> Option<(u32, u32, u32)> {
        let clean = v.trim().trim_start_matches('v').trim_start_matches('V');
        let parts: Vec<&str> = clean.split('.').collect();
        if parts.is_empty() {
            return None;
        }

        let maj: u32 = parts[0].parse().ok()?;
        let min: u32 = if parts.len() > 1 {
            parts[1].split('-').next().unwrap_or("0").parse().unwrap_or(0)
        } else {
            0
        };
        let pat: u32 = if parts.len() > 2 {
            parts[2].split('-').next().unwrap_or("0").parse().unwrap_or(0)
        } else {
            0
        };

        Some((maj, min, pat))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semver_parsing() {
        assert_eq!(UpdateDetector::parse_semver("0.1.0"), Some((0, 1, 0)));
        assert_eq!(UpdateDetector::parse_semver("v1.2.3"), Some((1, 2, 3)));
        assert_eq!(UpdateDetector::parse_semver("V2.0.0-beta"), Some((2, 0, 0)));
        assert_eq!(UpdateDetector::parse_semver("0.1"), Some((0, 1, 0)));
    }

    #[test]
    fn test_is_newer_version() {
        assert!(UpdateDetector::is_newer_version("0.1.0", "0.2.0"));
        assert!(UpdateDetector::is_newer_version("0.1.0", "0.1.1"));
        assert!(UpdateDetector::is_newer_version("v0.1.0", "v1.0.0"));
        assert!(!UpdateDetector::is_newer_version("0.2.0", "0.1.0"));
        assert!(!UpdateDetector::is_newer_version("0.1.0", "0.1.0"));
        assert!(!UpdateDetector::is_newer_version("1.0.0", "0.9.9"));
    }

    #[test]
    fn test_parse_release_json() {
        let json = r#"{
            "tag_name": "v0.2.0",
            "name": "Nexora v0.2.0 - High Performance Release",
            "body": "Fixes and performance enhancements.",
            "html_url": "https://github.com/aksh120/Nexora/releases/tag/v0.2.0",
            "assets": [
                {
                    "name": "Nexora.exe",
                    "browser_download_url": "https://github.com/aksh120/Nexora/releases/download/v0.2.0/Nexora.exe",
                    "size": 1646080
                }
            ]
        }"#;

        let info = UpdateDetector::parse_release_json(json).unwrap().unwrap();
        assert_eq!(info.tag_name, "v0.2.0");
        assert_eq!(info.version, "0.2.0");
        assert_eq!(info.name, "Nexora v0.2.0 - High Performance Release");
        assert_eq!(info.asset_name.as_deref(), Some("Nexora.exe"));
        assert_eq!(info.asset_size, Some(1646080));
        assert!(info.download_url.unwrap().contains("Nexora.exe"));
    }
}
