// SQLite Persistence and Storage Layer
#![allow(unused_imports)]

pub mod bookmarks;
pub mod cache;
pub mod db;
pub mod history;
pub mod session;
pub mod backup;
pub mod recovery;

pub use bookmarks::{BookmarkEntry, BookmarkManager};
pub use cache::CacheManager;
pub use db::{SqliteConnection, SqliteRow, SqliteValue};
pub use history::{HistoryEntry, HistoryManager};
pub use session::{SavedSession, SavedTab, SessionManager};
pub use backup::DatabaseBackupManager;
pub use recovery::{CrashRecoveryManager, SessionCheckpoint, CheckpointTab};
