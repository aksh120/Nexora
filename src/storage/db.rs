use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_uchar, c_void};
use std::path::Path;
use std::sync::{Arc, Mutex};
use windows::core::{s, w, PCWSTR};
use windows::Win32::Foundation::{FreeLibrary, HMODULE};
use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};

use crate::core::error::{NexoraError, Result};

pub const SQLITE_OK: c_int = 0;
pub const SQLITE_ROW: c_int = 100;
pub const SQLITE_DONE: c_int = 101;
pub const SQLITE_OPEN_READWRITE: c_int = 0x00000002;
pub const SQLITE_OPEN_CREATE: c_int = 0x00000004;
pub const SQLITE_OPEN_FULLMUTEX: c_int = 0x00010000;
pub const SQLITE_TRANSIENT: *mut c_void = -1isize as *mut c_void;

type FnOpenV2 = unsafe extern "system" fn(*const c_char, *mut *mut c_void, c_int, *const c_char) -> c_int;
type FnCloseV2 = unsafe extern "system" fn(*mut c_void) -> c_int;
type FnExec = unsafe extern "system" fn(*mut c_void, *const c_char, *mut c_void, *mut c_void, *mut *mut c_char) -> c_int;
type FnPrepareV2 = unsafe extern "system" fn(*mut c_void, *const c_char, c_int, *mut *mut c_void, *mut *const c_char) -> c_int;
type FnStep = unsafe extern "system" fn(*mut c_void) -> c_int;
type FnFinalize = unsafe extern "system" fn(*mut c_void) -> c_int;
type FnColumnText = unsafe extern "system" fn(*mut c_void, c_int) -> *const c_uchar;
type FnColumnInt64 = unsafe extern "system" fn(*mut c_void, c_int) -> i64;
type FnColumnInt = unsafe extern "system" fn(*mut c_void, c_int) -> c_int;
type FnColumnDouble = unsafe extern "system" fn(*mut c_void, c_int) -> f64;
type FnColumnCount = unsafe extern "system" fn(*mut c_void) -> c_int;
type FnBindText = unsafe extern "system" fn(*mut c_void, c_int, *const c_char, c_int, *mut c_void) -> c_int;
type FnBindInt64 = unsafe extern "system" fn(*mut c_void, c_int, i64) -> c_int;
type FnBindDouble = unsafe extern "system" fn(*mut c_void, c_int, f64) -> c_int;
type FnBindNull = unsafe extern "system" fn(*mut c_void, c_int) -> c_int;
type FnErrmsg = unsafe extern "system" fn(*mut c_void) -> *const c_char;
type FnChanges = unsafe extern "system" fn(*mut c_void) -> c_int;

#[derive(Clone)]
pub struct WinSqliteLib {
    _hmodule: Arc<HmoduleHolder>,
    open_v2: FnOpenV2,
    close_v2: FnCloseV2,
    exec: FnExec,
    prepare_v2: FnPrepareV2,
    step: FnStep,
    finalize: FnFinalize,
    column_text: FnColumnText,
    column_int64: FnColumnInt64,
    column_int: FnColumnInt,
    column_double: FnColumnDouble,
    column_count: FnColumnCount,
    bind_text: FnBindText,
    bind_int64: FnBindInt64,
    bind_double: FnBindDouble,
    bind_null: FnBindNull,
    errmsg: FnErrmsg,
    changes: FnChanges,
}

struct HmoduleHolder(HMODULE);
unsafe impl Send for HmoduleHolder {}
unsafe impl Sync for HmoduleHolder {}

impl Drop for HmoduleHolder {
    fn drop(&mut self) {
        unsafe {
            let _ = FreeLibrary(self.0);
        }
    }
}

impl WinSqliteLib {
    pub fn load() -> Result<Self> {
        unsafe {
            let hmod = LoadLibraryW(w!("winsqlite3.dll"))
                .map_err(|e| NexoraError::Storage(format!("Failed to load winsqlite3.dll: {}", e)))?;

            macro_rules! get_sym {
                ($name:ident, $str:expr, $ty:ty) => {{
                    let addr = GetProcAddress(hmod, $str)
                        .ok_or_else(|| NexoraError::Storage(format!("Missing SQLite export: {}", stringify!($name))))?;
                    std::mem::transmute::<_, $ty>(addr)
                }};
            }

            let open_v2 = get_sym!(sqlite3_open_v2, s!("sqlite3_open_v2"), FnOpenV2);
            let close_v2 = get_sym!(sqlite3_close_v2, s!("sqlite3_close_v2"), FnCloseV2);
            let exec = get_sym!(sqlite3_exec, s!("sqlite3_exec"), FnExec);
            let prepare_v2 = get_sym!(sqlite3_prepare_v2, s!("sqlite3_prepare_v2"), FnPrepareV2);
            let step = get_sym!(sqlite3_step, s!("sqlite3_step"), FnStep);
            let finalize = get_sym!(sqlite3_finalize, s!("sqlite3_finalize"), FnFinalize);
            let column_text = get_sym!(sqlite3_column_text, s!("sqlite3_column_text"), FnColumnText);
            let column_int64 = get_sym!(sqlite3_column_int64, s!("sqlite3_column_int64"), FnColumnInt64);
            let column_int = get_sym!(sqlite3_column_int, s!("sqlite3_column_int"), FnColumnInt);
            let column_double = get_sym!(sqlite3_column_double, s!("sqlite3_column_double"), FnColumnDouble);
            let column_count = get_sym!(sqlite3_column_count, s!("sqlite3_column_count"), FnColumnCount);
            let bind_text = get_sym!(sqlite3_bind_text, s!("sqlite3_bind_text"), FnBindText);
            let bind_int64 = get_sym!(sqlite3_bind_int64, s!("sqlite3_bind_int64"), FnBindInt64);
            let bind_double = get_sym!(sqlite3_bind_double, s!("sqlite3_bind_double"), FnBindDouble);
            let bind_null = get_sym!(sqlite3_bind_null, s!("sqlite3_bind_null"), FnBindNull);
            let errmsg = get_sym!(sqlite3_errmsg, s!("sqlite3_errmsg"), FnErrmsg);
            let changes = get_sym!(sqlite3_changes, s!("sqlite3_changes"), FnChanges);

            Ok(Self {
                _hmodule: Arc::new(HmoduleHolder(hmod)),
                open_v2,
                close_v2,
                exec,
                prepare_v2,
                step,
                finalize,
                column_text,
                column_int64,
                column_int,
                column_double,
                column_count,
                bind_text,
                bind_int64,
                bind_double,
                bind_null,
                errmsg,
                changes,
            })
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SqliteValue {
    Text(String),
    Integer(i64),
    Real(f64),
    Null,
}

pub struct SqliteRow<'a> {
    db: &'a SqliteConnection,
    stmt: *mut c_void,
}

impl<'a> SqliteRow<'a> {
    pub fn get_text(&self, col: i32) -> String {
        unsafe {
            let ptr = (self.db.lib.column_text)(self.stmt, col);
            if ptr.is_null() {
                String::new()
            } else {
                CStr::from_ptr(ptr as *const c_char)
                    .to_string_lossy()
                    .to_string()
            }
        }
    }

    pub fn get_i64(&self, col: i32) -> i64 {
        unsafe { (self.db.lib.column_int64)(self.stmt, col) }
    }

    pub fn get_i32(&self, col: i32) -> i32 {
        unsafe { (self.db.lib.column_int)(self.stmt, col) }
    }

    pub fn get_f64(&self, col: i32) -> f64 {
        unsafe { (self.db.lib.column_double)(self.stmt, col) }
    }
}

pub struct SqliteConnection {
    lib: WinSqliteLib,
    handle: Mutex<*mut c_void>,
}

unsafe impl Send for SqliteConnection {}
unsafe impl Sync for SqliteConnection {}

impl Drop for SqliteConnection {
    fn drop(&mut self) {
        if let Ok(guard) = self.handle.lock() {
            if !(*guard).is_null() {
                unsafe {
                    let _ = (self.lib.close_v2)(*guard);
                }
            }
        }
    }
}

impl SqliteConnection {
    pub fn open(path: &Path) -> Result<Self> {
        let lib = WinSqliteLib::load()?;
        let path_str = path.to_string_lossy().to_string();
        let c_path = CString::new(path_str)
            .map_err(|e| NexoraError::Storage(format!("Invalid path string: {}", e)))?;

        let mut db_handle: *mut c_void = std::ptr::null_mut();
        let flags = SQLITE_OPEN_READWRITE | SQLITE_OPEN_CREATE | SQLITE_OPEN_FULLMUTEX;

        let res = unsafe {
            (lib.open_v2)(c_path.as_ptr(), &mut db_handle, flags, std::ptr::null())
        };

        if res != SQLITE_OK || db_handle.is_null() {
            return Err(NexoraError::Storage(format!("Failed to open SQLite database: error code {}", res)));
        }

        let conn = Self {
            lib,
            handle: Mutex::new(db_handle),
        };

        // Configure WAL mode, synchronous=NORMAL, foreign_keys=ON, busy_timeout=5000
        conn.execute("PRAGMA journal_mode = WAL;")?;
        conn.execute("PRAGMA synchronous = NORMAL;")?;
        conn.execute("PRAGMA foreign_keys = ON;")?;
        conn.execute("PRAGMA busy_timeout = 5000;")?;

        log::info!("SQLite connection established to {:?} (WAL mode active)", path);

        Ok(conn)
    }

    pub fn open_in_memory() -> Result<Self> {
        Self::open(Path::new(":memory:"))
    }

    pub fn execute(&self, sql: &str) -> Result<()> {
        let guard = self.handle.lock().map_err(|_| NexoraError::Storage("Mutex poison".into()))?;
        let c_sql = CString::new(sql)
            .map_err(|e| NexoraError::Storage(format!("Invalid SQL string: {}", e)))?;

        let res = unsafe {
            (self.lib.exec)(
                *guard,
                c_sql.as_ptr(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        };

        if res != SQLITE_OK {
            let err_msg = unsafe {
                let err_ptr = (self.lib.errmsg)(*guard);
                if err_ptr.is_null() {
                    "Unknown SQLite error".to_string()
                } else {
                    CStr::from_ptr(err_ptr).to_string_lossy().to_string()
                }
            };
            return Err(NexoraError::Storage(format!("SQL execute failed: {} (code {})", err_msg, res)));
        }

        Ok(())
    }

    pub fn execute_with_params(&self, sql: &str, params: &[SqliteValue]) -> Result<i32> {
        let guard = self.handle.lock().map_err(|_| NexoraError::Storage("Mutex poison".into()))?;
        let c_sql = CString::new(sql)
            .map_err(|e| NexoraError::Storage(format!("Invalid SQL string: {}", e)))?;

        let mut stmt: *mut c_void = std::ptr::null_mut();
        let res = unsafe {
            (self.lib.prepare_v2)(
                *guard,
                c_sql.as_ptr(),
                -1,
                &mut stmt,
                std::ptr::null_mut(),
            )
        };

        if res != SQLITE_OK || stmt.is_null() {
            let err = self.last_error(*guard);
            return Err(NexoraError::Storage(format!("Prepare failed: {}", err)));
        }

        self.bind_params(stmt, params)?;

        let step_res = unsafe { (self.lib.step)(stmt) };
        unsafe { let _ = (self.lib.finalize)(stmt); }

        if step_res != SQLITE_DONE && step_res != SQLITE_ROW {
            let err = self.last_error(*guard);
            return Err(NexoraError::Storage(format!("Step failed: {}", err)));
        }

        let changes = unsafe { (self.lib.changes)(*guard) };
        Ok(changes)
    }

    pub fn query<T, F>(&self, sql: &str, params: &[SqliteValue], mut f: F) -> Result<Vec<T>>
    where
        F: FnMut(&SqliteRow) -> Result<T>,
    {
        let guard = self.handle.lock().map_err(|_| NexoraError::Storage("Mutex poison".into()))?;
        let c_sql = CString::new(sql)
            .map_err(|e| NexoraError::Storage(format!("Invalid SQL string: {}", e)))?;

        let mut stmt: *mut c_void = std::ptr::null_mut();
        let res = unsafe {
            (self.lib.prepare_v2)(
                *guard,
                c_sql.as_ptr(),
                -1,
                &mut stmt,
                std::ptr::null_mut(),
            )
        };

        if res != SQLITE_OK || stmt.is_null() {
            let err = self.last_error(*guard);
            return Err(NexoraError::Storage(format!("Prepare failed: {}", err)));
        }

        self.bind_params(stmt, params)?;

        let mut results = Vec::new();
        loop {
            let step_res = unsafe { (self.lib.step)(stmt) };
            if step_res == SQLITE_ROW {
                let row = SqliteRow { db: self, stmt };
                let val = f(&row)?;
                results.push(val);
            } else if step_res == SQLITE_DONE {
                break;
            } else {
                let err = self.last_error(*guard);
                unsafe { let _ = (self.lib.finalize)(stmt); }
                return Err(NexoraError::Storage(format!("Query step error: {}", err)));
            }
        }

        unsafe { let _ = (self.lib.finalize)(stmt); }
        Ok(results)
    }

    fn bind_params(&self, stmt: *mut c_void, params: &[SqliteValue]) -> Result<()> {
        for (i, p) in params.iter().enumerate() {
            let idx = (i + 1) as c_int;
            unsafe {
                match p {
                    SqliteValue::Text(s) => {
                        let c = CString::new(s.as_str())
                            .map_err(|e| NexoraError::Storage(format!("Invalid text param: {}", e)))?;
                        (self.lib.bind_text)(stmt, idx, c.as_ptr(), -1, SQLITE_TRANSIENT);
                    }
                    SqliteValue::Integer(v) => {
                        (self.lib.bind_int64)(stmt, idx, *v);
                    }
                    SqliteValue::Real(v) => {
                        (self.lib.bind_double)(stmt, idx, *v);
                    }
                    SqliteValue::Null => {
                        (self.lib.bind_null)(stmt, idx);
                    }
                }
            }
        }
        Ok(())
    }

    fn last_error(&self, handle: *mut c_void) -> String {
        unsafe {
            let err_ptr = (self.lib.errmsg)(handle);
            if err_ptr.is_null() {
                "Unknown SQLite error".to_string()
            } else {
                CStr::from_ptr(err_ptr).to_string_lossy().to_string()
            }
        }
    }

    pub fn migrate(&self) -> Result<()> {
        let version: i32 = self.query("PRAGMA user_version;", &[], |row| Ok(row.get_i32(0)))?
            .first()
            .copied()
            .unwrap_or(0);

        log::info!("Current database schema user_version: {}", version);

        if version < 1 {
            log::info!("Applying schema migration v1...");
            self.execute("BEGIN TRANSACTION;")?;

            // 1. History Table
            self.execute(
                "CREATE TABLE IF NOT EXISTS history (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    url TEXT NOT NULL,
                    title TEXT NOT NULL,
                    visit_time INTEGER NOT NULL,
                    visit_count INTEGER NOT NULL DEFAULT 1,
                    typed_count INTEGER NOT NULL DEFAULT 0
                );"
            )?;
            self.execute("CREATE INDEX IF NOT EXISTS idx_history_url ON history(url);")?;
            self.execute("CREATE INDEX IF NOT EXISTS idx_history_visit_time ON history(visit_time DESC);")?;

            // 2. Bookmarks Table
            self.execute(
                "CREATE TABLE IF NOT EXISTS bookmarks (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    parent_id INTEGER,
                    title TEXT NOT NULL,
                    url TEXT NOT NULL,
                    is_folder INTEGER NOT NULL DEFAULT 0,
                    position INTEGER NOT NULL DEFAULT 0,
                    created_at INTEGER NOT NULL
                );"
            )?;
            self.execute("CREATE INDEX IF NOT EXISTS idx_bookmarks_parent ON bookmarks(parent_id);")?;

            // 3. Saved Session Tabs Table
            self.execute(
                "CREATE TABLE IF NOT EXISTS session_tabs (
                    id INTEGER PRIMARY KEY,
                    position INTEGER NOT NULL,
                    url TEXT NOT NULL,
                    title TEXT NOT NULL,
                    is_active INTEGER NOT NULL DEFAULT 0,
                    is_suspended INTEGER NOT NULL DEFAULT 0,
                    created_at INTEGER NOT NULL
                );"
            )?;

            self.execute("PRAGMA user_version = 1;")?;
            self.execute("COMMIT;")?;
            log::info!("Database schema migration v1 applied successfully.");
        }

        if version < 2 {
            log::info!("Applying schema migration v2...");
            self.execute("BEGIN TRANSACTION;")?;

            // 4. Site Permissions Table
            self.execute(
                "CREATE TABLE IF NOT EXISTS site_permissions (
                    origin TEXT NOT NULL,
                    permission_type TEXT NOT NULL,
                    state TEXT NOT NULL,
                    updated_at INTEGER NOT NULL,
                    PRIMARY KEY (origin, permission_type)
                );"
            )?;
            self.execute("CREATE INDEX IF NOT EXISTS idx_site_permissions_origin ON site_permissions(origin);")?;

            self.execute("PRAGMA user_version = 2;")?;
            self.execute("COMMIT;")?;
            log::info!("Database schema migration v2 applied successfully.");
        }

        if version < 3 {
            log::info!("Applying schema migration v3 (Per-Site Zoom & Pinned Tab Persistence)...");
            self.execute("BEGIN TRANSACTION;")?;

            // 5. Site Zoom Table (Feature 3)
            self.execute(
                "CREATE TABLE IF NOT EXISTS site_zoom (
                    origin TEXT PRIMARY KEY,
                    zoom_factor REAL NOT NULL,
                    updated_at INTEGER NOT NULL
                );"
            )?;

            // 6. Extend session_tabs if columns missing
            let _ = self.execute("ALTER TABLE session_tabs ADD COLUMN is_pinned INTEGER NOT NULL DEFAULT 0;");
            let _ = self.execute("ALTER TABLE session_tabs ADD COLUMN zoom_factor REAL NOT NULL DEFAULT 1.0;");

            self.execute("PRAGMA user_version = 3;")?;
            self.execute("COMMIT;")?;
            log::info!("Database schema migration v3 applied successfully.");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sqlite_in_memory_connection_and_crud() {
        let db = SqliteConnection::open_in_memory()
            .expect("Failed to open in-memory SQLite database via winsqlite3.dll");

        db.execute("CREATE TABLE test_table (id INTEGER PRIMARY KEY, name TEXT, score INTEGER);")
            .expect("Failed to create table");

        let changes = db.execute_with_params(
            "INSERT INTO test_table (name, score) VALUES (?, ?);",
            &[SqliteValue::Text("Nexora".into()), SqliteValue::Integer(100)],
        )
        .expect("Failed to insert record");
        assert_eq!(changes, 1);

        let rows: Vec<(i64, String, i32)> = db
            .query("SELECT id, name, score FROM test_table WHERE score >= ?;", &[SqliteValue::Integer(50)], |row| {
                Ok((row.get_i64(0), row.get_text(1), row.get_i32(2)))
            })
            .expect("Query failed");

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].1, "Nexora");
        assert_eq!(rows[0].2, 100);
    }

    #[test]
    fn test_sqlite_migration() {
        let db = SqliteConnection::open_in_memory().expect("Open in memory failed");
        db.migrate().expect("Migration failed");

        let tables = db.query(
            "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name;",
            &[],
            |r| Ok(r.get_text(0)),
        ).expect("Query sqlite_master failed");

        assert!(tables.contains(&"history".to_string()));
        assert!(tables.contains(&"bookmarks".to_string()));
        assert!(tables.contains(&"session_tabs".to_string()));
        assert!(tables.contains(&"site_permissions".to_string()));
    }
}
