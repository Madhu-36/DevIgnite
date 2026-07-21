use rusqlite::{Connection, Result};
use std::sync::Mutex;

pub mod models;
pub mod schema;

pub use models::*;

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new(db_path: &std::path::Path) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
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

    // ── Installed Runtimes ──────────────────────────────────────

    pub fn insert_runtime(&self, rt: &InstalledRuntime) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO installed_runtimes (id, language, version, install_path, symlink_path, installed_at, is_active, checksum_verified, binary_name)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![rt.id, rt.language, rt.version, rt.install_path, rt.symlink_path, rt.installed_at, rt.is_active, rt.checksum_verified, rt.binary_name],
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
        rows.collect()
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
        rows.collect()
    }

    pub fn get_runtime_by_id(&self, runtime_id: &str) -> Result<Option<InstalledRuntime>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, language, version, install_path, symlink_path, installed_at, is_active, checksum_verified, binary_name
             FROM installed_runtimes WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(rusqlite::params![runtime_id], |row| {
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

    pub fn set_active_runtime(&self, language: &str, runtime_id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE installed_runtimes SET is_active = 0 WHERE language = ?1",
            rusqlite::params![language],
        )?;
        if !runtime_id.is_empty() {
            conn.execute(
                "UPDATE installed_runtimes SET is_active = 1 WHERE id = ?1",
                rusqlite::params![runtime_id],
            )?;
        }
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

    pub fn count_runtimes(&self) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        conn.query_row("SELECT COUNT(*) FROM installed_runtimes", [], |row| row.get(0))
    }

    pub fn count_runtimes_by_language(&self, language: &str) -> Result<i32> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT COUNT(*) FROM installed_runtimes WHERE language = ?1",
            rusqlite::params![language],
            |row| row.get(0),
        )
    }

    // ── Runtime Catalog ─────────────────────────────────────────

    pub fn upsert_catalog_entry(&self, entry: &RuntimeCatalogEntry) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO runtime_catalog (id, language, version, display_name, download_url, checksum_url, sha256, binary_name, platform, arch, file_size_bytes, release_date, fetched_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            rusqlite::params![entry.id, entry.language, entry.version, entry.display_name, entry.download_url, entry.checksum_url, entry.sha256, entry.binary_name, entry.platform, entry.arch, entry.file_size_bytes, entry.release_date, chrono::Utc::now().to_rfc3339()],
        )?;
        Ok(())
    }

    pub fn get_catalog_by_language(&self, language: &str) -> Result<Vec<RuntimeCatalogEntry>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, language, version, display_name, download_url, checksum_url, sha256, binary_name, platform, arch, file_size_bytes, release_date, 0
             FROM runtime_catalog WHERE language = ?1 ORDER BY version DESC",
        )?;
        let rows = stmt.query_map(rusqlite::params![language], |row| {
            Ok(RuntimeCatalogEntry {
                id: row.get(0)?,
                language: row.get(1)?,
                version: row.get(2)?,
                display_name: row.get(3)?,
                download_url: row.get(4)?,
                checksum_url: row.get(5)?,
                sha256: row.get(6)?,
                binary_name: row.get(7)?,
                platform: row.get(8)?,
                arch: row.get(9)?,
                file_size_bytes: row.get(10)?,
                release_date: row.get(11)?,
                is_installed: false,
            })
        })?;
        rows.collect()
    }

    pub fn clear_catalog(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM runtime_catalog", [])?;
        Ok(())
    }

    // ── Download Cache ──────────────────────────────────────────

    pub fn insert_download_cache(&self, cache: &DownloadCacheEntry) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO download_cache (id, url, local_path, sha256, downloaded_at, size_bytes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![cache.id, cache.url, cache.local_path, cache.sha256, cache.downloaded_at, cache.size_bytes],
        )?;
        Ok(())
    }

    pub fn get_download_cache(&self, url: &str) -> Result<Option<DownloadCacheEntry>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, url, local_path, sha256, downloaded_at, size_bytes FROM download_cache WHERE url = ?1",
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

    // ── Diagnostics ─────────────────────────────────────────────

    pub fn insert_diagnostic_result(&self, result: &DiagnosticResult) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO diagnostics (id, check_name, status, message, detected_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![result.id, result.check_name, result.status, result.message, result.detected_at],
        )?;
        Ok(())
    }

    pub fn get_system_health(&self) -> Result<Vec<DiagnosticResult>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, check_name, status, message, detected_at FROM diagnostics ORDER BY detected_at DESC LIMIT 100",
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
        rows.collect()
    }

    // ── Path State ──────────────────────────────────────────────

    pub fn insert_path_state(&self, state: &PathState) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO path_state (id, sandbox_bin_path, path_injected, last_verified_at) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![state.id, state.sandbox_bin_path, state.path_injected, state.last_verified_at],
        )?;
        Ok(())
    }

    pub fn get_path_state(&self) -> Result<Option<PathState>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, sandbox_bin_path, path_injected, last_verified_at FROM path_state LIMIT 1",
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

    // ── Install History ─────────────────────────────────────────

    pub fn log_install_action(
        &self,
        language: &str,
        version: &str,
        action: &str,
        status: &str,
        details: Option<&str>,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO install_history (id, language, version, action, status, details, performed_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                uuid::Uuid::new_v4().to_string(),
                language,
                version,
                action,
                status,
                details,
                chrono::Utc::now().to_rfc3339()
            ],
        )?;
        Ok(())
    }

    pub fn get_install_history(&self, limit: usize) -> Result<Vec<(String, String, String, String, Option<String>, String)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT language, version, action, status, details, performed_at FROM install_history ORDER BY performed_at DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(rusqlite::params![limit as i64], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
            ))
        })?;
        rows.collect()
    }

    // ── Doctor History ──────────────────────────────────────────

    pub fn insert_doctor_report(&self, report: &DoctorHistoryRecord) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO doctor_history (id, run_at, issues_found, issues_fixed, full_report) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![report.id, report.run_at, report.issues_found, report.issues_fixed, report.full_report],
        )?;
        Ok(())
    }
}
