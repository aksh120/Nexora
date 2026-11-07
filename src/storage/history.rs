use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::error::Result;
use crate::storage::db::{SqliteConnection, SqliteValue};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryEntry {
    pub id: i64,
    pub url: String,
    pub title: String,
    pub visit_time: i64,
    pub visit_count: i32,
    pub typed_count: i32,
}

pub struct HistoryManager {
    db: Arc<SqliteConnection>,
}

impl HistoryManager {
    pub fn new(db: Arc<SqliteConnection>) -> Self {
        Self { db }
    }

    pub fn add_visit(&self, url: &str, title: &str) -> Result<()> {
        let trimmed_url = url.trim();
        if trimmed_url.is_empty() || trimmed_url == "about:blank" {
            return Ok(());
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        // Check if URL already exists
        let existing: Vec<i64> = self.db.query(
            "SELECT id FROM history WHERE url = ? LIMIT 1;",
            &[SqliteValue::Text(trimmed_url.to_string())],
            |row| Ok(row.get_i64(0)),
        )?;

        if let Some(&id) = existing.first() {
            let update_title = if title.is_empty() {
                "UPDATE history SET visit_time = ?, visit_count = visit_count + 1 WHERE id = ?;"
            } else {
                "UPDATE history SET title = ?, visit_time = ?, visit_count = visit_count + 1 WHERE id = ?;"
            };

            if title.is_empty() {
                self.db.execute_with_params(
                    update_title,
                    &[SqliteValue::Integer(now), SqliteValue::Integer(id)],
                )?;
            } else {
                self.db.execute_with_params(
                    update_title,
                    &[
                        SqliteValue::Text(title.to_string()),
                        SqliteValue::Integer(now),
                        SqliteValue::Integer(id),
                    ],
                )?;
            }
        } else {
            let display_title = if title.is_empty() { trimmed_url } else { title };
            self.db.execute_with_params(
                "INSERT INTO history (url, title, visit_time, visit_count, typed_count) VALUES (?, ?, ?, 1, 0);",
                &[
                    SqliteValue::Text(trimmed_url.to_string()),
                    SqliteValue::Text(display_title.to_string()),
                    SqliteValue::Integer(now),
                ],
            )?;
        }

        Ok(())
    }

    pub fn record_typed(&self, url: &str) -> Result<()> {
        let trimmed = url.trim();
        self.db.execute_with_params(
            "UPDATE history SET typed_count = typed_count + 1 WHERE url = ?;",
            &[SqliteValue::Text(trimmed.to_string())],
        )?;
        Ok(())
    }

    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<HistoryEntry>> {
        let pattern = format!("%{}%", query.trim());
        let sql = "SELECT id, url, title, visit_time, visit_count, typed_count \
                   FROM history \
                   WHERE url LIKE ? OR title LIKE ? \
                   ORDER BY (typed_count * 2 + visit_count) DESC, visit_time DESC \
                   LIMIT ?;";

        self.db.query(
            sql,
            &[
                SqliteValue::Text(pattern.clone()),
                SqliteValue::Text(pattern),
                SqliteValue::Integer(limit as i64),
            ],
            |row| {
                Ok(HistoryEntry {
                    id: row.get_i64(0),
                    url: row.get_text(1),
                    title: row.get_text(2),
                    visit_time: row.get_i64(3),
                    visit_count: row.get_i32(4),
                    typed_count: row.get_i32(5),
                })
            },
        )
    }

    pub fn get_recent(&self, limit: usize) -> Result<Vec<HistoryEntry>> {
        let sql = "SELECT id, url, title, visit_time, visit_count, typed_count \
                   FROM history \
                   ORDER BY visit_time DESC \
                   LIMIT ?;";

        self.db.query(
            sql,
            &[SqliteValue::Integer(limit as i64)],
            |row| {
                Ok(HistoryEntry {
                    id: row.get_i64(0),
                    url: row.get_text(1),
                    title: row.get_text(2),
                    visit_time: row.get_i64(3),
                    visit_count: row.get_i32(4),
                    typed_count: row.get_i32(5),
                })
            },
        )
    }

    pub fn delete_entry(&self, id: i64) -> Result<()> {
        self.db.execute_with_params(
            "DELETE FROM history WHERE id = ?;",
            &[SqliteValue::Integer(id)],
        )?;
        Ok(())
    }

    pub fn clear_all(&self) -> Result<()> {
        self.db.execute("DELETE FROM history;")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_crud_and_search() {
        let db = Arc::new(SqliteConnection::open_in_memory().unwrap());
        db.migrate().unwrap();

        let history = HistoryManager::new(db);

        // 1. Add visits
        history.add_visit("https://www.youtube.com", "YouTube - Home").unwrap();
        history.add_visit("https://github.com/rust-lang", "GitHub: Rust").unwrap();
        history.add_visit("https://www.youtube.com", "YouTube - Home Updated").unwrap();

        // 2. Verify recent visits
        let recent = history.get_recent(10).unwrap();
        assert_eq!(recent.len(), 2);
        // YouTube should have visit_count = 2
        let yt = recent.iter().find(|e| e.url.contains("youtube.com")).unwrap();
        assert_eq!(yt.visit_count, 2);
        assert_eq!(yt.title, "YouTube - Home Updated");

        // 3. Search history
        let results = history.search("github", 5).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].url, "https://github.com/rust-lang");

        // 4. Record typed count
        history.record_typed("https://github.com/rust-lang").unwrap();
        let updated = history.search("github", 1).unwrap();
        assert_eq!(updated[0].typed_count, 1);

        // 5. Delete entry
        history.delete_entry(yt.id).unwrap();
        let remaining = history.get_recent(10).unwrap();
        assert_eq!(remaining.len(), 1);

        // 6. Clear all
        history.clear_all().unwrap();
        assert_eq!(history.get_recent(10).unwrap().len(), 0);
    }
}

