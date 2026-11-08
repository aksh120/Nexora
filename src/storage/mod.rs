// SQLite Persistence and Storage Layer
#![allow(unused_imports)]

pub mod bookmarks;
pub mod bookmarks_ui;
pub mod cache;
pub mod db;
pub mod history;
pub mod history_ui;
pub mod session;
pub mod backup;
pub mod recovery;
pub mod zoom;

pub use bookmarks::{BookmarkEntry, BookmarkManager};
pub use bookmarks_ui::BookmarksUi;
pub use cache::CacheManager;
pub use db::{SqliteConnection, SqliteRow, SqliteValue};
pub use history::{HistoryEntry, HistoryManager};
pub use history_ui::HistoryUi;
pub use session::{SavedSession, SavedTab, SessionManager};
pub use backup::DatabaseBackupManager;
pub use recovery::{CrashRecoveryManager, SessionCheckpoint, CheckpointTab};
pub use zoom::ZoomManager;
