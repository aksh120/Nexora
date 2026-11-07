use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::core::error::{NexoraError, Result};
use crate::storage::db::{SqliteConnection, SqliteValue};

/// Types of browser permissions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PermissionKind {
    Geolocation,
    Camera,
    Microphone,
    Notifications,
    Autoplay,
    ClipboardRead,
}

impl PermissionKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Geolocation => "geolocation",
            Self::Camera => "camera",
            Self::Microphone => "microphone",
            Self::Notifications => "notifications",
            Self::Autoplay => "autoplay",
            Self::ClipboardRead => "clipboard_read",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "geolocation" => Some(Self::Geolocation),
            "camera" => Some(Self::Camera),
            "microphone" => Some(Self::Microphone),
            "notifications" => Some(Self::Notifications),
            "autoplay" => Some(Self::Autoplay),
            "clipboard_read" => Some(Self::ClipboardRead),
            _ => None,
        }
    }
}

/// The state of a site permission
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionState {
    Allow,
    Block,
    Prompt,
}

impl PermissionState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Allow => "allow",
            Self::Block => "block",
            Self::Prompt => "prompt",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "allow" => Some(Self::Allow),
            "block" => Some(Self::Block),
            "prompt" => Some(Self::Prompt),
            _ => None,
        }
    }
}

/// Persistent site permission record
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SitePermission {
    pub origin: String,
    pub kind: PermissionKind,
    pub state: PermissionState,
    pub updated_at: i64,
}

/// Per-Site Permissions Manager managing storage, retrieval, and defaults
pub struct PermissionsManager {
    db: Arc<SqliteConnection>,
}

impl PermissionsManager {
    pub fn new(db: Arc<SqliteConnection>) -> Self {
        Self { db }
    }

    /// Normalizes a URL into standard scheme://domain:port origin format
    pub fn normalize_origin(url_or_origin: &str) -> String {
        let trimmed = url_or_origin.trim();
        if let Some(scheme_idx) = trimmed.find("://") {
            let scheme = &trimmed[..scheme_idx];
            let rest = &trimmed[scheme_idx + 3..];
            let host_and_port = match rest.find('/') {
                Some(slash_idx) => &rest[..slash_idx],
                None => rest,
            };
            format!("{}://{}", scheme.to_lowercase(), host_and_port.to_lowercase())
        } else {
            trimmed.to_lowercase()
        }
    }

    /// Retrieve the permission state for a specific origin and permission kind.
    /// If no explicit rule has been saved, returns the recommended privacy-conscious default.
    pub fn get_permission(&self, origin: &str, kind: PermissionKind) -> Result<PermissionState> {
        let norm_origin = Self::normalize_origin(origin);

        let rows = self.db.query(
            "SELECT state FROM site_permissions WHERE origin = ? AND permission_type = ? LIMIT 1;",
            &[
                SqliteValue::Text(norm_origin),
                SqliteValue::Text(kind.as_str().to_string()),
            ],
            |r| Ok(r.get_text(0)),
        )?;

        if let Some(state_str) = rows.first() {
            if let Some(state) = PermissionState::from_str(state_str) {
                return Ok(state);
            }
        }

        // Privacy-conscious defaults:
        // - Autoplay: Allow (essential for seamless video streaming)
        // - Notifications: Block (prevents website notification spam)
        // - Camera, Mic, Geolocation, Clipboard: Prompt
        let default_state = match kind {
            PermissionKind::Autoplay => PermissionState::Allow,
            PermissionKind::Notifications => PermissionState::Block,
            PermissionKind::Camera
            | PermissionKind::Microphone
            | PermissionKind::Geolocation
            | PermissionKind::ClipboardRead => PermissionState::Prompt,
        };

        Ok(default_state)
    }

    /// Sets or updates the permission state for an origin
    pub fn set_permission(
        &self,
        origin: &str,
        kind: PermissionKind,
        state: PermissionState,
    ) -> Result<()> {
        let norm_origin = Self::normalize_origin(origin);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        self.db.execute_with_params(
            "INSERT INTO site_permissions (origin, permission_type, state, updated_at) \
             VALUES (?, ?, ?, ?) \
             ON CONFLICT(origin, permission_type) DO UPDATE SET \
             state = excluded.state, updated_at = excluded.updated_at;",
            &[
                SqliteValue::Text(norm_origin.clone()),
                SqliteValue::Text(kind.as_str().to_string()),
                SqliteValue::Text(state.as_str().to_string()),
                SqliteValue::Integer(now),
            ],
        )?;

        log::info!(
            "PermissionsManager: Set permission {}={:?} for origin '{}'",
            kind.as_str(),
            state,
            norm_origin
        );

        Ok(())
    }

    /// Clears an explicit permission setting for an origin
    pub fn clear_permission(&self, origin: &str, kind: PermissionKind) -> Result<()> {
        let norm_origin = Self::normalize_origin(origin);
        self.db.execute_with_params(
            "DELETE FROM site_permissions WHERE origin = ? AND permission_type = ?;",
            &[
                SqliteValue::Text(norm_origin),
                SqliteValue::Text(kind.as_str().to_string()),
            ],
        )?;
        Ok(())
    }

    /// Clears all permission settings for an origin
    pub fn clear_all_for_origin(&self, origin: &str) -> Result<()> {
        let norm_origin = Self::normalize_origin(origin);
        self.db.execute_with_params(
            "DELETE FROM site_permissions WHERE origin = ?;",
            &[SqliteValue::Text(norm_origin)],
        )?;
        Ok(())
    }

    /// Lists all explicit permission records stored in the database
    pub fn get_all_permissions(&self) -> Result<Vec<SitePermission>> {
        self.db.query(
            "SELECT origin, permission_type, state, updated_at FROM site_permissions ORDER BY origin ASC;",
            &[],
            |r| {
                let origin = r.get_text(0);
                let type_str = r.get_text(1);
                let state_str = r.get_text(2);
                let updated_at = r.get_i64(3);

                let kind = PermissionKind::from_str(&type_str)
                    .ok_or_else(|| NexoraError::Storage(format!("Unknown permission type: {}", type_str)))?;
                let state = PermissionState::from_str(&state_str)
                    .ok_or_else(|| NexoraError::Storage(format!("Unknown permission state: {}", state_str)))?;

                Ok(SitePermission {
                    origin,
                    kind,
                    state,
                    updated_at,
                })
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_origin() {
        assert_eq!(
            PermissionsManager::normalize_origin("https://www.youtube.com/watch?v=123"),
            "https://www.youtube.com"
        );
        assert_eq!(
            PermissionsManager::normalize_origin("HTTP://Example.COM:8080/path/to/page"),
            "http://example.com:8080"
        );
        assert_eq!(
            PermissionsManager::normalize_origin("netflix.com"),
            "netflix.com"
        );
    }

    #[test]
    fn test_permissions_crud_and_defaults() {
        let db = Arc::new(SqliteConnection::open_in_memory().expect("open memory db"));
        db.migrate().expect("migration failed");

        let mgr = PermissionsManager::new(db);

        // Test defaults
        let autoplay = mgr.get_permission("https://youtube.com", PermissionKind::Autoplay).unwrap();
        assert_eq!(autoplay, PermissionState::Allow);

        let notif = mgr.get_permission("https://news.com", PermissionKind::Notifications).unwrap();
        assert_eq!(notif, PermissionState::Block);

        let geo = mgr.get_permission("https://maps.google.com", PermissionKind::Geolocation).unwrap();
        assert_eq!(geo, PermissionState::Prompt);

        // Set explicit permission
        mgr.set_permission("https://maps.google.com", PermissionKind::Geolocation, PermissionState::Allow).unwrap();
        let geo_updated = mgr.get_permission("https://maps.google.com", PermissionKind::Geolocation).unwrap();
        assert_eq!(geo_updated, PermissionState::Allow);

        // Clear permission
        mgr.clear_permission("https://maps.google.com", PermissionKind::Geolocation).unwrap();
        let geo_reverted = mgr.get_permission("https://maps.google.com", PermissionKind::Geolocation).unwrap();
        assert_eq!(geo_reverted, PermissionState::Prompt);
    }
}
