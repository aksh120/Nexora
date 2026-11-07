use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use crate::core::error::{NexoraError, Result};
use crate::tabs::Tab;

/// Checkpoint information for a single tab
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CheckpointTab {
    pub id: usize,
    pub url: String,
    pub title: String,
    pub is_suspended: bool,
}

/// Structured session checkpoint saved periodically to disk
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionCheckpoint {
    pub timestamp: u64,
    pub active_tab_id: Option<usize>,
    pub tabs: Vec<CheckpointTab>,
}

/// Crash Recovery Manager orchestrating clean shutdown markers
/// and periodic atomic session checkpointing.
pub struct CrashRecoveryManager {
    checkpoint_file: PathBuf,
    marker_file: PathBuf,
}

impl CrashRecoveryManager {
    pub fn new(data_dir: &Path) -> Self {
        Self {
            checkpoint_file: data_dir.join("session_checkpoint.json"),
            marker_file: data_dir.join("clean_exit.marker"),
        }
    }

    /// Checks whether the previous browser session terminated abnormally (e.g. power loss, crash, taskkill)
    pub fn detect_abnormal_shutdown(&self) -> bool {
        // If a session checkpoint exists AND the clean exit marker is missing, the previous session crashed
        self.checkpoint_file.exists() && !self.marker_file.exists()
    }

    /// Marks the start of an active browser session (clears previous clean exit marker)
    pub fn mark_session_active(&self) -> Result<()> {
        if self.marker_file.exists() {
            let _ = fs::remove_file(&self.marker_file);
        }
        log::debug!("Session marked active. Clean exit marker cleared.");
        Ok(())
    }

    /// Records an atomic crash-recovery checkpoint
    pub fn save_checkpoint(&self, tabs: &[Tab], active_id: Option<usize>) -> Result<()> {
        if tabs.is_empty() {
            return Ok(());
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let checkpoint = SessionCheckpoint {
            timestamp: now,
            active_tab_id: active_id,
            tabs: tabs
                .iter()
                .map(|t| CheckpointTab {
                    id: t.id,
                    url: t.url.clone(),
                    title: t.title.clone(),
                    is_suspended: t.is_suspended(),
                })
                .collect(),
        };

        let json = serde_json::to_string_pretty(&checkpoint)?;

        // Atomic write via temp file rename
        let temp_file = self.checkpoint_file.with_extension("tmp");
        fs::write(&temp_file, json)?;
        if self.checkpoint_file.exists() {
            let _ = fs::remove_file(&self.checkpoint_file);
        }
        fs::rename(&temp_file, &self.checkpoint_file)?;

        log::debug!(
            "Saved crash checkpoint with {} tabs (active: {:?})",
            checkpoint.tabs.len(),
            active_id
        );

        Ok(())
    }

    /// Loads the latest checkpoint if available
    pub fn load_checkpoint(&self) -> Result<Option<SessionCheckpoint>> {
        if !self.checkpoint_file.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&self.checkpoint_file)?;
        let checkpoint: SessionCheckpoint = serde_json::from_str(&content)
            .map_err(|e| NexoraError::Serialization(format!("Corrupt checkpoint JSON: {}", e)))?;

        log::info!(
            "Loaded crash checkpoint containing {} tabs from {:?}",
            checkpoint.tabs.len(),
            self.checkpoint_file
        );

        Ok(Some(checkpoint))
    }

    /// Marks that the browser exited cleanly (writes clean exit marker)
    pub fn mark_clean_exit(&self) -> Result<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        fs::write(&self.marker_file, format!("clean_exit={}\npid={}", now, std::process::id()))?;
        log::info!("Clean exit marker recorded at {:?}", self.marker_file);
        Ok(())
    }

    /// Clears the checkpoint file (e.g. after clean restore or user reset)
    pub fn clear_checkpoint(&self) -> Result<()> {
        if self.checkpoint_file.exists() {
            let _ = fs::remove_file(&self.checkpoint_file);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checkpoint_save_and_load() {
        let temp_dir = std::env::temp_dir().join(format!("nexora_recovery_test_{}", std::process::id()));
        let _ = fs::create_dir_all(&temp_dir);

        let mgr = CrashRecoveryManager::new(&temp_dir);

        let tab1 = Tab::new(1, "https://youtube.com", Some("YouTube"));
        let tab2 = Tab::new(2, "https://netflix.com", Some("Netflix"));
        let tabs = vec![tab1, tab2];

        mgr.save_checkpoint(&tabs, Some(1)).expect("save checkpoint");

        let loaded = mgr.load_checkpoint().expect("load checkpoint");
        assert!(loaded.is_some());
        let cp = loaded.unwrap();
        assert_eq!(cp.tabs.len(), 2);
        assert_eq!(cp.active_tab_id, Some(1));
        assert_eq!(cp.tabs[0].url, "https://youtube.com");
        assert_eq!(cp.tabs[1].url, "https://netflix.com");

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_abnormal_shutdown_detection() {
        let temp_dir = std::env::temp_dir().join(format!("nexora_crash_test_{}", std::process::id()));
        let _ = fs::create_dir_all(&temp_dir);

        let mgr = CrashRecoveryManager::new(&temp_dir);

        // Scenario 1: Initial boot with no prior files
        assert!(!mgr.detect_abnormal_shutdown());

        // Scenario 2: Active session crashes (checkpoint saved, no clean exit marker)
        mgr.mark_session_active().unwrap();
        let tabs = vec![Tab::new(1, "https://google.com", None)];
        mgr.save_checkpoint(&tabs, Some(1)).unwrap();
        assert!(mgr.detect_abnormal_shutdown(), "Should detect abnormal shutdown when marker is missing");

        // Scenario 3: Clean exit
        mgr.mark_clean_exit().unwrap();
        assert!(!mgr.detect_abnormal_shutdown(), "Should NOT detect crash after clean exit marker is written");

        let _ = fs::remove_dir_all(temp_dir);
    }
}

