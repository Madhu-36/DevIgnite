use crate::error::Result;
use rusqlite::Connection;
use std::path::PathBuf;

pub struct DbState {
    pub conn: std::sync::Mutex<Connection>,
}

#[derive(Debug, serde::Serialize)]
pub struct Runtime {
    pub id: i64,
    pub tool_name: String,
    pub version: String,
    pub path: String,
    pub sha256_checksum: String,
    pub is_active: bool,
    pub installed_at: String,
}

pub fn init_db(db_path: PathBuf) -> Result<Connection> {
    if let Some(parent) = db_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }

    let conn = Connection::open(db_path)?;

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

    Ok(conn)
}
