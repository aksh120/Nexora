//! Download Management Module
pub mod manager;
pub mod downloads_ui;

#[allow(unused_imports)]
pub use manager::{DownloadItem, DownloadManager, DownloadState};
#[allow(unused_imports)]
pub use downloads_ui::DownloadsUi;

