use rusqlite::{Connection, Result};

const MIGRATIONS: &[&str] = &[
    // v1: core tables
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

    // v2: download cache
    "CREATE TABLE IF NOT EXISTS download_cache (
        id TEXT PRIMARY KEY NOT NULL,
        url TEXT NOT NULL UNIQUE,
        local_path TEXT NOT NULL,
        sha256 TEXT NOT NULL,
        downloaded_at TEXT NOT NULL,
        size_bytes INTEGER NOT NULL
    );
    CREATE INDEX IF NOT EXISTS idx_cache_url ON download_cache(url);",

    // v3: diagnostics
    "CREATE TABLE IF NOT EXISTS diagnostics (
        id TEXT PRIMARY KEY NOT NULL,
        check_name TEXT NOT NULL,
        status TEXT NOT NULL CHECK(status IN ('ok', 'warning', 'error', 'fixed')),
        message TEXT NOT NULL,
        detected_at TEXT NOT NULL
    );
    CREATE INDEX IF NOT EXISTS idx_diagnostics_check ON diagnostics(check_name);",

    // v4: path state
    "CREATE TABLE IF NOT EXISTS path_state (
        id TEXT PRIMARY KEY NOT NULL,
        sandbox_bin_path TEXT NOT NULL,
        path_injected BOOLEAN NOT NULL DEFAULT 0,
        last_verified_at TEXT NOT NULL
    );",

    // v5: smoke test results
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

    // v6: doctor history
    "CREATE TABLE IF NOT EXISTS doctor_history (
        id TEXT PRIMARY KEY NOT NULL,
        run_at TEXT NOT NULL,
        issues_found INTEGER NOT NULL DEFAULT 0,
        issues_fixed INTEGER NOT NULL DEFAULT 0,
        full_report TEXT NOT NULL
    );",

    // v7: runtime catalog cache
    "CREATE TABLE IF NOT EXISTS runtime_catalog (
        id TEXT PRIMARY KEY NOT NULL,
        language TEXT NOT NULL,
        version TEXT NOT NULL,
        display_name TEXT NOT NULL,
        download_url TEXT NOT NULL,
        checksum_url TEXT NOT NULL,
        sha256 TEXT NOT NULL,
        binary_name TEXT NOT NULL,
        platform TEXT NOT NULL,
        arch TEXT NOT NULL,
        file_size_bytes INTEGER NOT NULL DEFAULT 0,
        release_date TEXT NOT NULL,
        fetched_at TEXT NOT NULL
    );
    CREATE INDEX IF NOT EXISTS idx_catalog_language ON runtime_catalog(language);
    CREATE UNIQUE INDEX IF NOT EXISTS idx_catalog_lang_ver ON runtime_catalog(language, version);",

    // v8: install history log
    "CREATE TABLE IF NOT EXISTS install_history (
        id TEXT PRIMARY KEY NOT NULL,
        language TEXT NOT NULL,
        version TEXT NOT NULL,
        action TEXT NOT NULL CHECK(action IN ('install', 'uninstall', 'switch')),
        status TEXT NOT NULL CHECK(status IN ('success', 'failed')),
        details TEXT,
        performed_at TEXT NOT NULL
    );
    CREATE INDEX IF NOT EXISTS idx_history_lang ON install_history(language);",
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
            log::info!("Applied migration v{}", version);
        }
    }

    Ok(())
}
