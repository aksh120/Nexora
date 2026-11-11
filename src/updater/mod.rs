//! Update Detection & Native In-Place Installer Module
//!
//! Checks GitHub Releases for new updates, displays an interactive
//! update banner, and applies seamless in-place updates.

pub mod detector;
pub mod installer;
pub mod banner;

pub use detector::{ReleaseInfo, UpdateDetector};
pub use installer::UpdateInstaller;
pub use banner::UpdateBanner;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

pub const DEFAULT_REPO_OWNER: &str = "aksh120";
pub const DEFAULT_REPO_NAME: &str = "Nexora";

#[derive(Clone)]
pub struct UpdateService {
    repo_owner: String,
    repo_name: String,
    cached_release: Arc<Mutex<Option<ReleaseInfo>>>,
    is_checking: Arc<AtomicBool>,
    is_updating: Arc<AtomicBool>,
    dismissed: Arc<AtomicBool>,
}

impl Default for UpdateService {
    fn default() -> Self {
        Self::new(DEFAULT_REPO_OWNER, DEFAULT_REPO_NAME)
    }
}

impl UpdateService {
    pub fn new(repo_owner: &str, repo_name: &str) -> Self {
        Self {
            repo_owner: repo_owner.to_string(),
            repo_name: repo_name.to_string(),
            cached_release: Arc::new(Mutex::new(None)),
            is_checking: Arc::new(AtomicBool::new(false)),
            is_updating: Arc::new(AtomicBool::new(false)),
            dismissed: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn cached_update(&self) -> Option<ReleaseInfo> {
        self.cached_release.lock().ok().and_then(|g| g.clone())
    }

    pub fn is_dismissed(&self) -> bool {
        self.dismissed.load(Ordering::SeqCst)
    }

    pub fn dismiss(&self) {
        self.dismissed.store(true, Ordering::SeqCst);
    }

    pub fn is_updating(&self) -> bool {
        self.is_updating.load(Ordering::SeqCst)
    }

    /// Spawns an asynchronous background check for GitHub releases.
    pub fn check_in_background<F>(&self, on_update_found: F)
    where
        F: Fn(ReleaseInfo) + Send + 'static,
    {
        if self.is_checking.swap(true, Ordering::SeqCst) {
            return; // Check already in flight
        }

        let owner = self.repo_owner.clone();
        let name = self.repo_name.clone();
        let is_checking = self.is_checking.clone();
        let cached = self.cached_release.clone();
        let current_version = env!("CARGO_PKG_VERSION").to_string();

        std::thread::Builder::new()
            .name("nexora-update-checker".into())
            .spawn(move || {
                log::info!("[UpdateService] Querying GitHub releases for {}/{}...", owner, name);
                match UpdateDetector::check_for_updates(&owner, &name, &current_version) {
                    Ok(Some(release)) => {
                        log::info!("[UpdateService] Found update: {}", release.version);
                        if let Ok(mut g) = cached.lock() {
                            *g = Some(release.clone());
                        }
                        on_update_found(release);
                    }
                    Ok(None) => {
                        log::info!("[UpdateService] Current version {} is up to date.", current_version);
                        if let Ok(mut g) = cached.lock() {
                            *g = None;
                        }
                    }
                    Err(e) => {
                        log::warn!("[UpdateService] Update check failed: {}", e);
                    }
                }
                is_checking.store(false, Ordering::SeqCst);
            })
            .expect("Failed to spawn update checker thread");
    }

    /// Downloads the new version and performs an in-place atomic upgrade and restart.
    pub fn apply_update_async<FProg, FErr>(&self, download_url: String, on_progress: FProg, on_error: FErr)
    where
        FProg: Fn(f64, String) + Send + Sync + 'static,
        FErr: Fn(String) + Send + 'static,
    {
        if self.is_updating.swap(true, Ordering::SeqCst) {
            log::warn!("[UpdateService] Update already in progress");
            return;
        }

        let is_updating = self.is_updating.clone();

        std::thread::Builder::new()
            .name("nexora-installer".into())
            .spawn(move || {
                let current_exe = match std::env::current_exe() {
                    Ok(p) => p,
                    Err(e) => {
                        is_updating.store(false, Ordering::SeqCst);
                        on_error(format!("Failed to determine exe path: {}", e));
                        return;
                    }
                };

                let exe_dir = match current_exe.parent() {
                    Some(p) => p,
                    None => {
                        is_updating.store(false, Ordering::SeqCst);
                        on_error("Failed to determine exe directory".into());
                        return;
                    }
                };

                let staged_path = exe_dir.join("Nexora_update.tmp");
                log::info!("[UpdateService] Downloading update payload to: {:?}", staged_path);

                let progress_fn = move |percent: f64, received: i64, total: i64| {
                    let rec_mb = received as f64 / (1024.0 * 1024.0);
                    let status = if total > 0 {
                        let tot_mb = total as f64 / (1024.0 * 1024.0);
                        format!("Downloading update: {:.1} / {:.1} MB ({:.0}%)", rec_mb, tot_mb, percent)
                    } else {
                        format!("Downloading update: {:.1} MB...", rec_mb)
                    };
                    on_progress(percent, status);
                };

                if let Err(e) = UpdateInstaller::download_update_payload(&download_url, &staged_path, Some(progress_fn)) {
                    is_updating.store(false, Ordering::SeqCst);
                    let _ = std::fs::remove_file(&staged_path);
                    on_error(format!("Download failed: {}", e));
                    return;
                }

                log::info!("[UpdateService] Download verified. Applying in-place atomic swap...");
                if let Err(e) = UpdateInstaller::apply_update_and_restart(&staged_path) {
                    is_updating.store(false, Ordering::SeqCst);
                    let _ = std::fs::remove_file(&staged_path);
                    on_error(format!("Installation failed: {}", e));
                }
            })
            .expect("Failed to spawn update installer thread");
    }
}
