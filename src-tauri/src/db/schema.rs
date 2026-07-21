use rusqlite::{Connection, Result};

const MIGRATIONS: &[&str] = &[
    "CREATE TABLE IF NOT EXISTS installed_runtimes (
        id TEXT PRIMARY KEY NOT NULL,
        language TEXT NOT NULL,
        version TEXT NOT NULL,
        install_path TEXT NOT NULL,
        symlink_path TEXT NOT NULL,
        installed_at TEXT NOT NULL,
        is_active BOOLEAN NOT NULL DEFAULT 0,
        checksum_verified BOOLEAN NOT NULL DEFAULT 0,
        binary_name TEXT NOT NULL
    );

    CREATE INDEX IF NOT EXISTS idx_runtimes_language ON installed_runtimes(language);
    CREATE INDEX IF NOT EXISTS idx_runtimes_active ON installed_runtimes(language, is_active);",

    "CREATE TABLE IF NOT EXISTS download_cache (
        id TEXT PRIMARY KEY NOT NULL,
        url TEXT NOT NULL UNIQUE,
        local_path TEXT NOT NULL,
        sha256 TEXT NOT NULL,
        downloaded_at TEXT NOT NULL,
        size_bytes INTEGER NOT NULL
    );

    CREATE INDEX IF NOT EXISTS idx_cache_url ON download_cache(url);",

    "CREATE TABLE IF NOT EXISTS diagnostics (
        id TEXT PRIMARY KEY NOT NULL,
        check_name TEXT NOT NULL,
        status TEXT NOT NULL CHECK(status IN ('ok', 'warning', 'error', 'fixed')),
        message TEXT NOT NULL,
        detected_at TEXT NOT NULL
    );

    CREATE INDEX IF NOT EXISTS idx_diagnostics_check ON diagnostics(check_name);",

    "CREATE TABLE IF NOT EXISTS path_state (
        id TEXT PRIMARY KEY NOT NULL,
        sandbox_bin_path TEXT NOT NULL,
        path_injected BOOLEAN NOT NULL DEFAULT 0,
        last_verified_at TEXT NOT NULL
    );",

    "CREATE TABLE IF NOT EXISTS smoke_test_results (
        id TEXT PRIMARY KEY NOT NULL,
        runtime_id TEXT NOT NULL,
        test_command TEXT NOT NULL,
        exit_code INTEGER NOT NULL,
        stdout TEXT,
        stderr TEXT,
        passed BOOLEAN NOT NULL,
        tested_at TEXT NOT NULL,
        FOREIGN KEY (runtime_id) REFERENCES installed_runtimes(id) ON DELETE CASCADE
    );

    CREATE INDEX IF NOT EXISTS idx_smoke_runtime ON smoke_test_results(runtime_id);",

    "CREATE TABLE IF NOT EXISTS doctor_history (
        id TEXT PRIMARY KEY NOT NULL,
        run_at TEXT NOT NULL,
        issues_found INTEGER NOT NULL DEFAULT 0,
        issues_fixed INTEGER NOT NULL DEFAULT 0,
        full_report TEXT NOT NULL
    );",
];

pub fn run_migrations(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY NOT NULL,
            applied_at TEXT NOT NULL
        );",
    )?;

    let current_version: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    for (idx, migration) in MIGRATIONS.iter().enumerate() {
        let version = (idx + 1) as i32;
        if version > current_version {
            conn.execute_batch(migration)?;
            conn.execute(
                "INSERT INTO schema_migrations (version, applied_at) VALUES (?1, ?2)",
                rusqlite::params![version, chrono::Utc::now().to_rfc3339()],
            )?;
            log::info!("Applied migration version {}", version);
        }
    }

    Ok(())
}
