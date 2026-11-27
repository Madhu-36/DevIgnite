use crate::error::Result;
use rusqlite::Connection;
use std::path::PathBuf;

pub struct DbState {
    pub conn: std::sync::Mutex<Connection>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct Runtime {
    pub id: i64,
    pub tool_name: String,
    pub version: String,
    pub path: String,
    pub sha256_checksum: String,
    pub is_active: bool,
    pub installed_at: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct SwitchHistoryEntry {
    pub id: i64,
    pub tool_name: String,
    pub version: String,
    pub switched_at: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct DiskUsageStats {
    pub runtimes_bytes: u64,
    pub cache_bytes: u64,
    pub db_bytes: u64,
    pub total_bytes: u64,
}

pub fn calculate_dir_size(path: &std::path::Path) -> u64 {
    let mut total = 0;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata() {
                if meta.is_dir() {
                    total += calculate_dir_size(&entry.path());
                } else {
                    total += meta.len();
                }
            }
        }
    }
    total
}

pub fn get_disk_usage(root: &std::path::Path) -> DiskUsageStats {
    let runtimes_bytes = calculate_dir_size(&root.join("runtimes"));
    let cache_bytes = calculate_dir_size(&root.join("cache"));
    let db_bytes = std::fs::metadata(root.join("devignite.db"))
        .map(|m| m.len())
        .unwrap_or(0);

    DiskUsageStats {
        runtimes_bytes,
        cache_bytes,
        db_bytes,
        total_bytes: runtimes_bytes + cache_bytes + db_bytes,
    }
}

pub fn run_migrations(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS installed_runtimes (
            id INTEGER PRIMARY KEY,
            tool_name TEXT NOT NULL,
            version TEXT NOT NULL,
            path TEXT NOT NULL,
            sha256_checksum TEXT NOT NULL,
            is_active BOOLEAN NOT NULL DEFAULT 0,
            installed_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS system_config (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
        [],
    )?;

    // Create index on tool_name and version for fast query lookups
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_runtimes_tool_version ON installed_runtimes(tool_name, version)",
        [],
    )?;

    // Mark migration v1 as applied
    conn.execute(
        "INSERT OR IGNORE INTO schema_migrations (version) VALUES (1)",
        [],
    )?;

    // Migration v2: runtime_switch_history
    conn.execute(
        "CREATE TABLE IF NOT EXISTS runtime_switch_history (
            id INTEGER PRIMARY KEY,
            tool_name TEXT NOT NULL,
            version TEXT NOT NULL,
            switched_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    conn.execute(
        "INSERT OR IGNORE INTO schema_migrations (version) VALUES (2)",
        [],
    )?;

    Ok(())
}

pub fn log_switch(conn: &Connection, tool: &str, version: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO runtime_switch_history (tool_name, version) VALUES (?1, ?2)",
        rusqlite::params![tool, version],
    )?;
    Ok(())
}

pub fn init_db(db_path: PathBuf) -> Result<Connection> {
    if let Some(parent) = db_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }

    let conn = Connection::open(db_path)?;
    run_migrations(&conn)?;

    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_in_memory_db_migrations() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        let count: i64 = conn
            .query_row("SELECT count(*) FROM schema_migrations", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_system_config_insert_and_update() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        conn.execute(
            "INSERT INTO system_config (key, value) VALUES (?1, ?2)",
            rusqlite::params!["theme", "dark"],
        )
        .unwrap();

        let val: String = conn
            .query_row(
                "SELECT value FROM system_config WHERE key = ?1",
                rusqlite::params!["theme"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(val, "dark");
    }

    #[test]
    fn test_installed_runtime_lifecycle() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        conn.execute(
            "INSERT INTO installed_runtimes (tool_name, version, path, sha256_checksum, is_active) 
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params!["node", "20.11.0", "/dummy/path", "fakehash", true],
        )
        .unwrap();

        let active: bool = conn
            .query_row(
                "SELECT is_active FROM installed_runtimes WHERE tool_name = 'node' AND version = '20.11.0'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(active);
    }
}
