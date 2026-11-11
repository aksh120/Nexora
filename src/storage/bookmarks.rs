use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::error::Result;
use crate::storage::db::{SqliteConnection, SqliteValue};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BookmarkEntry {
    pub id: i64,
    pub parent_id: Option<i64>,
    pub title: String,
    pub url: String,
    pub is_folder: bool,
    pub position: i32,
    pub created_at: i64,
}

pub struct BookmarkManager {
    db: Arc<SqliteConnection>,
}

impl BookmarkManager {
    pub fn new(db: Arc<SqliteConnection>) -> Self {
        Self { db }
    }

    pub fn add_bookmark(&self, title: &str, url: &str, parent_id: Option<i64>) -> Result<i64> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let parent_val = match parent_id {
            Some(pid) => SqliteValue::Integer(pid),
            None => SqliteValue::Null,
        };

        // Determine next position in folder
        let next_pos = self.get_next_position(parent_id)?;

        self.db.execute_with_params(
            "INSERT INTO bookmarks (parent_id, title, url, is_folder, position, created_at) VALUES (?, ?, ?, 0, ?, ?);",
            &[
                parent_val,
                SqliteValue::Text(title.to_string()),
                SqliteValue::Text(url.to_string()),
                SqliteValue::Integer(next_pos as i64),
                SqliteValue::Integer(now),
            ],
        )?;

        let id = self.db.query("SELECT last_insert_rowid();", &[], |row| Ok(row.get_i64(0)))?
            .first()
            .copied()
            .unwrap_or(0);

        log::info!("Added bookmark #{} '{}' ({})", id, title, url);
        Ok(id)
    }

    pub fn add_bookmarks_batch(&self, bookmarks: &[(&str, &str, Option<i64>)]) -> Result<Vec<i64>> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        self.db.begin_transaction()?;
        let mut ids = Vec::with_capacity(bookmarks.len());

        for &(title, url, parent_id) in bookmarks {
            let parent_val = match parent_id {
                Some(pid) => SqliteValue::Integer(pid),
                None => SqliteValue::Null,
            };

            let next_pos = self.get_next_position(parent_id)?;

            self.db.execute_with_params(
                "INSERT INTO bookmarks (parent_id, title, url, is_folder, position, created_at) VALUES (?, ?, ?, 0, ?, ?);",
                &[
                    parent_val,
                    SqliteValue::Text(title.to_string()),
                    SqliteValue::Text(url.to_string()),
                    SqliteValue::Integer(next_pos as i64),
                    SqliteValue::Integer(now),
                ],
            )?;

            let id = self.db.query("SELECT last_insert_rowid();", &[], |row| Ok(row.get_i64(0)))?
                .first()
                .copied()
                .unwrap_or(0);
            ids.push(id);
        }

        self.db.commit()?;
        Ok(ids)
    }

    pub fn add_folder(&self, title: &str, parent_id: Option<i64>) -> Result<i64> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let parent_val = match parent_id {
            Some(pid) => SqliteValue::Integer(pid),
            None => SqliteValue::Null,
        };

        let next_pos = self.get_next_position(parent_id)?;

        self.db.execute_with_params(
            "INSERT INTO bookmarks (parent_id, title, url, is_folder, position, created_at) VALUES (?, ?, '', 1, ?, ?);",
            &[
                parent_val,
                SqliteValue::Text(title.to_string()),
                SqliteValue::Integer(next_pos as i64),
                SqliteValue::Integer(now),
            ],
        )?;

        let id = self.db.query("SELECT last_insert_rowid();", &[], |row| Ok(row.get_i64(0)))?
            .first()
            .copied()
            .unwrap_or(0);

        log::info!("Created bookmark folder #{} '{}'", id, title);
        Ok(id)
    }

    pub fn get_bookmarks(&self, parent_id: Option<i64>) -> Result<Vec<BookmarkEntry>> {
        let (sql, params) = match parent_id {
            Some(pid) => (
                "SELECT id, parent_id, title, url, is_folder, position, created_at \
                 FROM bookmarks WHERE parent_id = ? ORDER BY position ASC, id ASC;",
                vec![SqliteValue::Integer(pid)],
            ),
            None => (
                "SELECT id, parent_id, title, url, is_folder, position, created_at \
                 FROM bookmarks WHERE parent_id IS NULL ORDER BY position ASC, id ASC;",
                vec![],
            ),
        };

        self.db.query(sql, &params, |row| {
            Ok(BookmarkEntry {
                id: row.get_i64(0),
                parent_id: if row.get_text(1).is_empty() { None } else { Some(row.get_i64(1)) },
                title: row.get_text(2),
                url: row.get_text(3),
                is_folder: row.get_i32(4) != 0,
                position: row.get_i32(5),
                created_at: row.get_i64(6),
            })
        })
    }

    pub fn get_all(&self) -> Result<Vec<BookmarkEntry>> {
        let sql = "SELECT id, parent_id, title, url, is_folder, position, created_at \
                   FROM bookmarks ORDER BY position ASC, id ASC;";

        self.db.query(sql, &[], |row| {
            Ok(BookmarkEntry {
                id: row.get_i64(0),
                parent_id: if row.get_text(1).is_empty() { None } else { Some(row.get_i64(1)) },
                title: row.get_text(2),
                url: row.get_text(3),
                is_folder: row.get_i32(4) != 0,
                position: row.get_i32(5),
                created_at: row.get_i64(6),
            })
        })
    }

    pub fn search(&self, query: &str) -> Result<Vec<BookmarkEntry>> {
        let pattern = format!("%{}%", query.trim());
        let sql = "SELECT id, parent_id, title, url, is_folder, position, created_at \
                   FROM bookmarks \
                   WHERE (title LIKE ? OR url LIKE ?) AND is_folder = 0 \
                   ORDER BY title ASC;";

        self.db.query(
            sql,
            &[
                SqliteValue::Text(pattern.clone()),
                SqliteValue::Text(pattern),
            ],
            |row| {
                Ok(BookmarkEntry {
                    id: row.get_i64(0),
                    parent_id: if row.get_text(1).is_empty() { None } else { Some(row.get_i64(1)) },
                    title: row.get_text(2),
                    url: row.get_text(3),
                    is_folder: row.get_i32(4) != 0,
                    position: row.get_i32(5),
                    created_at: row.get_i64(6),
                })
            },
        )
    }

    pub fn update_bookmark(&self, id: i64, title: &str, url: &str) -> Result<()> {
        self.db.execute_with_params(
            "UPDATE bookmarks SET title = ?, url = ? WHERE id = ?;",
            &[
                SqliteValue::Text(title.to_string()),
                SqliteValue::Text(url.to_string()),
                SqliteValue::Integer(id),
            ],
        )?;
        Ok(())
    }

    pub fn delete_bookmark(&self, id: i64) -> Result<()> {
        // Also delete sub-items if folder
        self.db.execute_with_params(
            "DELETE FROM bookmarks WHERE id = ? OR parent_id = ?;",
            &[SqliteValue::Integer(id), SqliteValue::Integer(id)],
        )?;
        Ok(())
    }

    pub fn is_bookmarked(&self, url: &str) -> Result<bool> {
        let sql = "SELECT COUNT(*) FROM bookmarks WHERE url = ? AND is_folder = 0;";
        let rows = self.db.query(sql, &[SqliteValue::Text(url.to_string())], |row| {
            Ok(row.get_i64(0))
        })?;
        Ok(rows.first().copied().unwrap_or(0) > 0)
    }

    pub fn delete_bookmark_by_url(&self, url: &str) -> Result<()> {
        self.db.execute_with_params(
            "DELETE FROM bookmarks WHERE url = ? AND is_folder = 0;",
            &[SqliteValue::Text(url.to_string())],
        )?;
        Ok(())
    }

    pub fn purge_default_bookmarks(&self) -> Result<()> {
        let default_urls = [
            "https://www.youtube.com",
            "https://www.netflix.com",
            "https://www.primevideo.com",
            "https://www.hotstar.com",
            "https://www.twitch.tv",
            "https://open.spotify.com",
        ];
        for url in default_urls {
            let _ = self.delete_bookmark_by_url(url);
        }
        Ok(())
    }

    fn get_next_position(&self, parent_id: Option<i64>) -> Result<i32> {
        let (sql, params) = match parent_id {
            Some(pid) => (
                "SELECT COALESCE(MAX(position), -1) + 1 FROM bookmarks WHERE parent_id = ?;",
                vec![SqliteValue::Integer(pid)],
            ),
            None => (
                "SELECT COALESCE(MAX(position), -1) + 1 FROM bookmarks WHERE parent_id IS NULL;",
                vec![],
            ),
        };

        let pos: i32 = self.db.query(sql, &params, |row| Ok(row.get_i32(0)))?
            .first()
            .copied()
            .unwrap_or(0);

        Ok(pos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bookmark_management() {
        let db = Arc::new(SqliteConnection::open_in_memory().unwrap());
        db.migrate().unwrap();

        let bm = BookmarkManager::new(db);

        // 1. Create a folder
        let folder_id = bm.add_folder("Media Streaming", None).unwrap();
        assert!(folder_id > 0);

        // 2. Add bookmarks inside folder
        let b1 = bm.add_bookmark("YouTube", "https://youtube.com", Some(folder_id)).unwrap();
        let b2 = bm.add_bookmark("Netflix", "https://netflix.com", Some(folder_id)).unwrap();
        assert!(b1 > 0);
        assert!(b2 > 0);

        // 3. Add root bookmark
        let b3 = bm.add_bookmark("Rust Lang", "https://rust-lang.org", None).unwrap();
        assert!(b3 > 0);

        // 4. Retrieve by folder
        let folder_items = bm.get_bookmarks(Some(folder_id)).unwrap();
        assert_eq!(folder_items.len(), 2);
        assert_eq!(folder_items[0].title, "YouTube");
        assert_eq!(folder_items[1].title, "Netflix");

        // 5. Search
        let search_res = bm.search("netflix").unwrap();
        assert_eq!(search_res.len(), 1);
        assert_eq!(search_res[0].url, "https://netflix.com");

        // 6. Update
        bm.update_bookmark(b1, "YouTube 4K", "https://youtube.com/4k").unwrap();
        let updated = bm.search("YouTube 4K").unwrap();
        assert_eq!(updated[0].url, "https://youtube.com/4k");

        // 7. Delete folder (cascades sub-items)
        bm.delete_bookmark(folder_id).unwrap();
        assert_eq!(bm.get_all().unwrap().len(), 1); // Only root 'Rust Lang' remains
    }
}

