use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::error::Result;
use crate::storage::db::{SqliteConnection, SqliteValue};
use crate::tabs::Tab;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SavedTab {
    pub id: usize,
    pub position: i32,
    pub url: String,
    pub title: String,
    pub is_active: bool,
    pub is_suspended: bool,
}

#[derive(Debug, Clone, Default)]
pub struct SavedSession {
    pub tabs: Vec<SavedTab>,
    pub active_tab_id: Option<usize>,
}

pub struct SessionManager {
    db: Arc<SqliteConnection>,
}

impl SessionManager {
    pub fn new(db: Arc<SqliteConnection>) -> Self {
        Self { db }
    }

    pub fn save_session(&self, tabs: &[Tab], active_tab_id: Option<usize>) -> Result<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        self.db.execute("BEGIN TRANSACTION;")?;
        self.db.execute("DELETE FROM session_tabs;")?;

        for (pos, tab) in tabs.iter().enumerate() {
            let is_active = if Some(tab.id) == active_tab_id { 1 } else { 0 };
            let is_suspended = if tab.is_suspended() { 1 } else { 0 };

            self.db.execute_with_params(
                "INSERT INTO session_tabs (id, position, url, title, is_active, is_suspended, created_at) \
                 VALUES (?, ?, ?, ?, ?, ?, ?);",
                &[
                    SqliteValue::Integer(tab.id as i64),
                    SqliteValue::Integer(pos as i64),
                    SqliteValue::Text(tab.url.clone()),
                    SqliteValue::Text(tab.title.clone()),
                    SqliteValue::Integer(is_active),
                    SqliteValue::Integer(is_suspended),
                    SqliteValue::Integer(now),
                ],
            )?;
        }

        self.db.execute("COMMIT;")?;
        log::info!("Saved session with {} tabs", tabs.len());
        Ok(())
    }

    pub fn load_session(&self) -> Result<SavedSession> {
        let sql = "SELECT id, position, url, title, is_active, is_suspended \
                   FROM session_tabs ORDER BY position ASC;";

        let mut active_id = None;

        let tabs = self.db.query(sql, &[], |row| {
            let id = row.get_i64(0) as usize;
            let is_active = row.get_i32(4) != 0;
            if is_active {
                active_id = Some(id);
            }

            Ok(SavedTab {
                id,
                position: row.get_i32(1),
                url: row.get_text(2),
                title: row.get_text(3),
                is_active,
                is_suspended: row.get_i32(5) != 0,
            })
        })?;

        Ok(SavedSession {
            tabs,
            active_tab_id: active_id,
        })
    }

    pub fn clear_session(&self) -> Result<()> {
        self.db.execute("DELETE FROM session_tabs;")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_save_and_load() {
        let db = Arc::new(SqliteConnection::open_in_memory().unwrap());
        db.migrate().unwrap();

        let sm = SessionManager::new(db);

        let tab1 = Tab::new(1, "https://google.com", Some("Google"));
        let mut tab2 = Tab::new(2, "https://youtube.com", Some("YouTube"));
        tab2.state = crate::tabs::TabState::Inactive;
        assert!(tab2.suspend(), "Inactive tab should suspend");

        let tabs = vec![tab1.clone(), tab2.clone()];
        sm.save_session(&tabs, Some(1)).unwrap();

        let loaded = sm.load_session().unwrap();
        assert_eq!(loaded.tabs.len(), 2);
        assert_eq!(loaded.active_tab_id, Some(1));
        assert_eq!(loaded.tabs[0].url, "https://google.com");
        assert_eq!(loaded.tabs[1].url, "https://youtube.com");
        assert!(loaded.tabs[1].is_suspended);

        // Clear session
        sm.clear_session().unwrap();
        assert_eq!(sm.load_session().unwrap().tabs.len(), 0);
    }
}

