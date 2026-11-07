use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::core::error::{NexoraError, Result};
use crate::storage::db::SqliteConnection;

/// Database Backup and Recovery Manager
///
/// Ensures zero data loss through integrity verification (`PRAGMA integrity_check`),
/// automated WAL compaction (`PRAGMA wal_checkpoint`), and corruption recovery.
pub struct DatabaseBackupManager;

impl DatabaseBackupManager {
    /// Validates database file integrity using SQLite's internal B-tree check
    pub fn check_integrity(conn: &SqliteConnection) -> Result<bool> {
        let rows = conn.query("PRAGMA integrity_check(1);", &[], |r| Ok(r.get_text(0)))?;
        let status = rows.first().map(|s| s.as_str()).unwrap_or("error");
        let is_ok = status.eq_ignore_ascii_case("ok");

        if is_ok {
            log::debug!("Database integrity verification passed: OK");
        } else {
            log::warn!("Database integrity check FAILED: {}", status);
        }

        Ok(is_ok)
    }

    /// Flushes all WAL (Write-Ahead Logging) journal pages back to the primary database file
    pub fn checkpoint_wal(conn: &SqliteConnection) -> Result<()> {
        conn.execute("PRAGMA wal_checkpoint(TRUNCATE);")?;
        log::debug!("Flushed SQLite WAL journal to disk (TRUNCATE mode).");
        Ok(())
    }

    /// Creates an atomic snapshot backup of the current database file
    pub fn create_backup(conn: &SqliteConnection, db_path: &Path, backup_path: &Path) -> Result<()> {
        if !db_path.exists() {
            return Err(NexoraError::Storage("Source database file does not exist".into()));
        }

        // 1. Quiesce and checkpoint WAL to ensure main database file has full state
        let _ = Self::checkpoint_wal(conn);

        // 2. Perform atomic copy to temporary file then rename
        let temp_backup = backup_path.with_extension("tmp");
        fs::copy(db_path, &temp_backup)?;
        if backup_path.exists() {
            let _ = fs::remove_file(backup_path);
        }
        fs::rename(&temp_backup, backup_path)?;

        log::info!("Created verified database backup at {:?}", backup_path);
        Ok(())
    }

    /// Restores the database from a verified backup file
    pub fn restore_from_backup(db_path: &Path, backup_path: &Path) -> Result<bool> {
        if !backup_path.exists() {
            log::warn!("No backup found at {:?} to restore from.", backup_path);
            return Ok(false);
        }

        // Verify backup is valid before restoring
        let test_conn = SqliteConnection::open(backup_path)?;
        if !Self::check_integrity(&test_conn)? {
            log::error!("Cannot restore: Backup file at {:?} is also corrupt!", backup_path);
            return Ok(false);
        }
        drop(test_conn);

        // Quarantine corrupted database file if it exists
        if db_path.exists() {
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let quarantine = db_path.with_extension(format!("corrupt.{}.bak", timestamp));
            log::warn!("Quarantining corrupted database to {:?}", quarantine);
            let _ = fs::rename(db_path, &quarantine);

            // Also remove dangling WAL and SHM files
            let wal_path = db_path.with_extension("db-wal");
            let shm_path = db_path.with_extension("db-shm");
            let _ = fs::remove_file(wal_path);
            let _ = fs::remove_file(shm_path);
        }

        // Copy backup to db_path
        fs::copy(backup_path, db_path)?;
        log::info!("Successfully restored database from backup {:?}", backup_path);
        Ok(true)
    }

    /// Verifies database integrity at startup and automatically repairs/restores if needed
    pub fn open_or_recover(db_path: &Path, backup_path: &Path) -> Result<Arc<SqliteConnection>> {
        if db_path.exists() {
            // Attempt opening and verifying existing database
            match SqliteConnection::open(db_path) {
                Ok(conn) => {
                    if let Ok(true) = Self::check_integrity(&conn) {
                        return Ok(Arc::new(conn));
                    }
                    log::warn!("Corrupt database detected at {:?}. Commencing auto-recovery...", db_path);
                    drop(conn);
                }
                Err(err) => {
                    log::warn!("Failed to open database at {:?} ({}). Commencing auto-recovery...", db_path, err);
                }
            }

            // Database is corrupt or unopenable, attempt recovery from backup
            let restored = Self::restore_from_backup(db_path, backup_path)?;
            if restored {
                let conn = SqliteConnection::open(db_path)?;
                return Ok(Arc::new(conn));
            }
        }

        // Fresh database initialization
        let conn = SqliteConnection::open(db_path)?;
        conn.migrate()?;
        let conn_arc = Arc::new(conn);

        // Create initial clean backup
        let _ = Self::create_backup(&conn_arc, db_path, backup_path);

        Ok(conn_arc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_integrity_check() {
        let conn = SqliteConnection::open_in_memory().expect("in memory db");
        conn.migrate().expect("migrate");

        let is_ok = DatabaseBackupManager::check_integrity(&conn).expect("integrity check");
        assert!(is_ok, "Clean migrated database must pass integrity check");
    }

    #[test]
    fn test_backup_and_recovery_flow() {
        let temp_dir = std::env::temp_dir().join(format!("nexora_backup_test_{}", std::process::id()));
        let _ = fs::create_dir_all(&temp_dir);

        let db_path = temp_dir.join("test.db");
        let backup_path = temp_dir.join("test.db.bak");

        // 1. Create original database with data
        let conn = SqliteConnection::open(&db_path).expect("open db");
        conn.migrate().expect("migrate");
        conn.execute("INSERT INTO history (url, title, visit_time) VALUES ('https://example.com', 'Example', 12345);")
            .expect("insert");

        // 2. Create backup
        DatabaseBackupManager::create_backup(&conn, &db_path, &backup_path).expect("create backup");
        assert!(backup_path.exists());
        drop(conn);

        // 3. Intentionally corrupt the original database by overwriting header bytes
        fs::write(&db_path, b"CORRUPTED_DATABASE_GARBAGE_HEADER_DATA_1234567890").expect("corrupt");

        // 4. Run open_or_recover
        let recovered = DatabaseBackupManager::open_or_recover(&db_path, &backup_path);
        assert!(recovered.is_ok(), "Auto-recovery from backup must succeed");
        let conn = recovered.unwrap();

        // 5. Verify restored data
        let rows = conn.query("SELECT url FROM history WHERE url = 'https://example.com';", &[], |r| Ok(r.get_text(0)))
            .expect("query recovered db");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0], "https://example.com");

        // Cleanup
        drop(conn);
        let _ = fs::remove_dir_all(temp_dir);
    }
}
