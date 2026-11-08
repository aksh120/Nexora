use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::core::error::Result;
use crate::storage::db::{SqliteConnection, SqliteValue};

pub struct ZoomManager {
    db: Arc<SqliteConnection>,
}

impl ZoomManager {
    pub fn new(db: Arc<SqliteConnection>) -> Self {
        Self { db }
    }

    /// Extracts clean origin (scheme + host + optional port) from any URL
    pub fn extract_origin(url: &str) -> String {
        let clean = url.trim();
        if clean.starts_with("nexora://") || clean.starts_with("about:") {
            return "nexora://".to_string();
        }
        if let Some(pos) = clean.find("://") {
            let scheme_end = pos + 3;
            let rest = &clean[scheme_end..];
            let host_end = rest.find('/').unwrap_or(rest.len());
            format!("{}{}", &clean[..scheme_end], &rest[..host_end])
        } else {
            clean.to_string()
        }
    }

    /// Retrieve persisted zoom factor for an origin (returns None if not stored)
    pub fn get_zoom(&self, origin: &str) -> Result<Option<f64>> {
        let sql = "SELECT zoom_factor FROM site_zoom WHERE origin = ?;";
        let rows = self.db.query(sql, &[SqliteValue::Text(origin.to_string())], |row| {
            Ok(row.get_f64(0))
        })?;
        Ok(rows.first().copied())
    }

    /// Persist zoom factor for an origin
    pub fn set_zoom(&self, origin: &str, zoom: f64) -> Result<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let sql = "INSERT INTO site_zoom (origin, zoom_factor, updated_at) \
                   VALUES (?, ?, ?) \
                   ON CONFLICT(origin) DO UPDATE SET zoom_factor = excluded.zoom_factor, updated_at = excluded.updated_at;";
        self.db.execute_with_params(
            sql,
            &[
                SqliteValue::Text(origin.to_string()),
                SqliteValue::Real(zoom),
                SqliteValue::Integer(now),
            ],
        )?;
        Ok(())
    }

    /// Remove origin zoom override (reset to 100%)
    pub fn remove_zoom(&self, origin: &str) -> Result<()> {
        let sql = "DELETE FROM site_zoom WHERE origin = ?;";
        self.db.execute_with_params(sql, &[SqliteValue::Text(origin.to_string())])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_origin() {
        assert_eq!(ZoomManager::extract_origin("https://www.youtube.com/watch?v=123"), "https://www.youtube.com");
        assert_eq!(ZoomManager::extract_origin("http://localhost:8080/path"), "http://localhost:8080");
        assert_eq!(ZoomManager::extract_origin("nexora://home"), "nexora://");
    }

    #[test]
    fn test_zoom_crud() {
        let db = Arc::new(SqliteConnection::open_in_memory().unwrap());
        db.migrate().unwrap();
        let mgr = ZoomManager::new(db);

        assert_eq!(mgr.get_zoom("https://example.com").unwrap(), None);
        mgr.set_zoom("https://example.com", 1.25).unwrap();
        assert_eq!(mgr.get_zoom("https://example.com").unwrap(), Some(1.25));

        mgr.set_zoom("https://example.com", 1.5).unwrap();
        assert_eq!(mgr.get_zoom("https://example.com").unwrap(), Some(1.5));

        mgr.remove_zoom("https://example.com").unwrap();
        assert_eq!(mgr.get_zoom("https://example.com").unwrap(), None);
    }
}

