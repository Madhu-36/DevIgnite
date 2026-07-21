use rusqlite::{Connection, Result};
use std::sync::Mutex;

pub struct Database {
    conn: Mutex<Connection>,
}

pub mod schema;
pub mod models;

pub use models::*;

impl Database {
    pub fn new(db_path: &std::path::Path) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        let db = Self {
            conn: Mutex::new(conn),
        };
        db.initialize()?;
        Ok(db)
    }

    fn initialize(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        schema::run_migrations(&conn)
    }

    pub fn insert_runtime(&self, runtime: &InstalledRuntime) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO installed_runtimes (id, language, version, install_path, symlink_path, installed_at, is_active, checksum_verified, binary_name)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                runtime.id,
                runtime.language,
                runtime.version,
                runtime.install_path,
                runtime.symlink_path,
                runtime.installed_at,
                runtime.is_active,
                runtime.checksum_verified,
                runtime.binary_name,
            ],
        )?;
        Ok(())
    }

    pub fn get_active_runtime(&self, language: &str) -> Result<Option<InstalledRuntime>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, language, version, install_path, symlink_path, installed_at, is_active, checksum_verified, binary_name
             FROM installed_runtimes WHERE language = ?1 AND is_active = 1",
        )?;
        let mut rows = stmt.query_map(rusqlite::params![language], |row| {
            Ok(InstalledRuntime {
                id: row.get(0)?,
                language: row.get(1)?,
                version: row.get(2)?,
                install_path: row.get(3)?,
                symlink_path: row.get(4)?,
                installed_at: row.get(5)?,
                is_active: row.get(6)?,
                checksum_verified: row.get(7)?,
                binary_name: row.get(8)?,
            })
        })?;
        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }

    pub fn get_all_runtimes(&self) -> Result<Vec<InstalledRuntime>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, language, version, install_path, symlink_path, installed_at, is_active, checksum_verified, binary_name
             FROM installed_runtimes ORDER BY language, version",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(InstalledRuntime {
                id: row.get(0)?,
                language: row.get(1)?,
                version: row.get(2)?,
                install_path: row.get(3)?,
                symlink_path: row.get(4)?,
                installed_at: row.get(5)?,
                is_active: row.get(6)?,
                checksum_verified: row.get(7)?,
                binary_name: row.get(8)?,
            })
        })?;
        let mut runtimes = Vec::new();
        for row in rows {
            runtimes.push(row?);
        }
        Ok(runtimes)
    }

    pub fn get_runtimes_by_language(&self, language: &str) -> Result<Vec<InstalledRuntime>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, language, version, install_path, symlink_path, installed_at, is_active, checksum_verified, binary_name
             FROM installed_runtimes WHERE language = ?1 ORDER BY version",
        )?;
        let rows = stmt.query_map(rusqlite::params![language], |row| {
            Ok(InstalledRuntime {
                id: row.get(0)?,
                language: row.get(1)?,
                version: row.get(2)?,
                install_path: row.get(3)?,
                symlink_path: row.get(4)?,
                installed_at: row.get(5)?,
                is_active: row.get(6)?,
                checksum_verified: row.get(7)?,
                binary_name: row.get(8)?,
            })
        })?;
        let mut runtimes = Vec::new();
        for row in rows {
            runtimes.push(row?);
        }
        Ok(runtimes)
    }

    pub fn set_active_runtime(&self, language: &str, runtime_id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE installed_runtimes SET is_active = 0 WHERE language = ?1",
            rusqlite::params![language],
        )?;
        conn.execute(
            "UPDATE installed_runtimes SET is_active = 1 WHERE id = ?1",
            rusqlite::params![runtime_id],
        )?;
        Ok(())
    }

    pub fn delete_runtime(&self, runtime_id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM installed_runtimes WHERE id = ?1",
            rusqlite::params![runtime_id],
        )?;
        Ok(())
    }

    pub fn insert_download_cache(&self, cache: &DownloadCacheEntry) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO download_cache (id, url, local_path, sha256, downloaded_at, size_bytes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                cache.id,
                cache.url,
                cache.local_path,
                cache.sha256,
                cache.downloaded_at,
                cache.size_bytes,
            ],
        )?;
        Ok(())
    }

    pub fn get_download_cache(&self, url: &str) -> Result<Option<DownloadCacheEntry>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, url, local_path, sha256, downloaded_at, size_bytes
             FROM download_cache WHERE url = ?1",
        )?;
        let mut rows = stmt.query_map(rusqlite::params![url], |row| {
            Ok(DownloadCacheEntry {
                id: row.get(0)?,
                url: row.get(1)?,
                local_path: row.get(2)?,
                sha256: row.get(3)?,
                downloaded_at: row.get(4)?,
                size_bytes: row.get(5)?,
            })
        })?;
        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }

    pub fn insert_diagnostic_result(&self, result: &DiagnosticResult) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO diagnostics (id, check_name, status, message, detected_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                result.id,
                result.check_name,
                result.status,
                result.message,
                result.detected_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_system_health(&self) -> Result<Vec<DiagnosticResult>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, check_name, status, message, detected_at
             FROM diagnostics ORDER BY detected_at DESC LIMIT 100",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(DiagnosticResult {
                id: row.get(0)?,
                check_name: row.get(1)?,
                status: row.get(2)?,
                message: row.get(3)?,
                detected_at: row.get(4)?,
            })
        })?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    pub fn insert_path_state(&self, state: &PathState) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO path_state (id, sandbox_bin_path, path_injected, last_verified_at)
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![
                state.id,
                state.sandbox_bin_path,
                state.path_injected,
                state.last_verified_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_path_state(&self) -> Result<Option<PathState>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, sandbox_bin_path, path_injected, last_verified_at
             FROM path_state LIMIT 1",
        )?;
        let mut rows = stmt.query_map([], |row| {
            Ok(PathState {
                id: row.get(0)?,
                sandbox_bin_path: row.get(1)?,
                path_injected: row.get(2)?,
                last_verified_at: row.get(3)?,
            })
        })?;
        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }
}
