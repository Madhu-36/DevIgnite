use crate::db::{DbState, Runtime};
use crate::doctor::{fix_broken_symlinks, run_diagnostics, AutoFixResult, DiagnosticReport};
use crate::error::Result;
use tauri::State;

#[tauri::command]
pub fn get_installed_runtimes(state: State<'_, DbState>) -> Result<Vec<Runtime>> {
    let conn = state.conn.lock().map_err(|e| crate::error::AppError::Unknown(anyhow::anyhow!("Mutex poisoned: {}", e)))?;
    let mut stmt = conn.prepare("SELECT id, tool_name, version, path, sha256_checksum, is_active, installed_at FROM installed_runtimes")?;
    
    let iter = stmt.query_map([], |row| {
        Ok(Runtime {
            id: row.get(0)?,
            tool_name: row.get(1)?,
            version: row.get(2)?,
            path: row.get(3)?,
            sha256_checksum: row.get(4)?,
            is_active: row.get(5)?,
            installed_at: row.get(6)?,
        })
    })?;

    let mut runtimes = Vec::new();
    for r in iter {
        if let Ok(runtime) = r {
            runtimes.push(runtime);
        }
    }
    
    Ok(runtimes)
}

#[tauri::command]
pub async fn install_runtime(name: String, version: String, state: State<'_, DbState>) -> Result<()> {
    let devignite_dir = crate::cli::get_devignite_root();
    let cache_dir = devignite_dir.join("cache");
    let runtime_dir = devignite_dir.join("runtimes").join(&name).join(&version);
    
    if !cache_dir.exists() {
        std::fs::create_dir_all(&cache_dir)?;
    }
    
    // Resolve URL (Mocking for now based on Node.js/Python, assuming win-x64)
    let url = match name.as_str() {
        "node" => format!("https://nodejs.org/dist/v{0}/node-v{0}-win-x64.zip", version),
        "go" => format!("https://golang.org/dl/go{0}.windows-amd64.zip", version),
        "python" => format!("https://github.com/indygreg/python-build-standalone/releases/download/20240224/cpython-{0}+20240224-x86_64-pc-windows-msvc-shared-install_only.tar.gz", version),
        "rust" => format!("https://static.rust-lang.org/dist/rust-{0}-x86_64-pc-windows-msvc.tar.gz", version),
        _ => return Err(crate::error::AppError::Installation(format!("Unsupported runtime: {}", name))),
    };
    
    let archive_path = cache_dir.join(format!("{}-{}.zip", name, version));
    
    crate::installer::download_and_verify(&url, &archive_path, None).await?;
    crate::installer::unpack_archive(&archive_path, &runtime_dir)?;
    
    // Insert into DB
    let conn = state.conn.lock().map_err(|e| crate::error::AppError::Unknown(anyhow::anyhow!("Mutex poisoned: {}", e)))?;
    conn.execute(
        "INSERT INTO installed_runtimes (tool_name, version, path, sha256_checksum, is_active) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![name, version, runtime_dir.to_string_lossy().to_string(), "unknown", false],
    )?;

    Ok(())
}

#[tauri::command]
pub fn switch_version(name: String, version: String, state: State<'_, DbState>) -> Result<()> {
    let devignite_dir = crate::cli::get_devignite_root();
    let bin_dir = devignite_dir.join("bin");
    let runtime_dir = devignite_dir.join("runtimes").join(&name).join(&version);
    
    // Wire up the symlink
    if runtime_dir.exists() {
        let symlink_path = bin_dir.join(&name);
        crate::sandbox::create_symlink(&runtime_dir, &symlink_path)?;
        crate::sandbox::broadcast_path_change(&bin_dir)?;
        
        let conn = state.conn.lock().map_err(|e| crate::error::AppError::Unknown(anyhow::anyhow!("Mutex poisoned: {}", e)))?;
        
        // Deactivate old
        conn.execute("UPDATE installed_runtimes SET is_active = 0 WHERE tool_name = ?1", rusqlite::params![name])?;
        
        // Activate new
        conn.execute(
            "UPDATE installed_runtimes SET is_active = 1 WHERE tool_name = ?1 AND version = ?2",
            rusqlite::params![name, version],
        )?;
    } else {
        return Err(crate::error::AppError::Installation("Runtime version not found locally".into()));
    }
    
    Ok(())
}

#[tauri::command]
pub fn uninstall_runtime(name: String, version: String, state: State<'_, DbState>) -> Result<()> {
    let devignite_dir = crate::cli::get_devignite_root();
    let runtime_dir = devignite_dir.join("runtimes").join(&name).join(&version);
    
    if runtime_dir.exists() {
        std::fs::remove_dir_all(&runtime_dir)?;
    }
    
    let conn = state.conn.lock().map_err(|e| crate::error::AppError::Unknown(anyhow::anyhow!("Mutex poisoned: {}", e)))?;
    conn.execute(
        "DELETE FROM installed_runtimes WHERE tool_name = ?1 AND version = ?2",
        rusqlite::params![name, version],
    )?;

    Ok(())
}

#[tauri::command]
pub fn get_system_config(key: String, state: State<'_, DbState>) -> Result<String> {
    let conn = state.conn.lock().map_err(|e| crate::error::AppError::Unknown(anyhow::anyhow!("Mutex poisoned: {}", e)))?;
    let mut stmt = conn.prepare("SELECT value FROM system_config WHERE key = ?1")?;
    let value: String = stmt.query_row(rusqlite::params![key], |row| row.get(0)).unwrap_or_default();
    Ok(value)
}

#[tauri::command]
pub fn set_system_config(key: String, value: String, state: State<'_, DbState>) -> Result<()> {
    let conn = state.conn.lock().map_err(|e| crate::error::AppError::Unknown(anyhow::anyhow!("Mutex poisoned: {}", e)))?;
    conn.execute(
        "INSERT INTO system_config (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value",
        rusqlite::params![key, value],
    )?;
    Ok(())
}

#[tauri::command]
pub fn clear_cache() -> Result<()> {
    let devignite_dir = crate::cli::get_devignite_root();
    let cache_dir = devignite_dir.join("cache");
    if cache_dir.exists() {
        std::fs::remove_dir_all(&cache_dir)?;
        std::fs::create_dir_all(&cache_dir)?;
    }
    Ok(())
}

#[tauri::command]
pub fn run_diagnostics_cmd() -> Result<DiagnosticReport> {
    let bin_dir = crate::cli::get_devignite_root().join("bin");
    run_diagnostics(&bin_dir)
}

#[tauri::command]
pub fn auto_fix_diagnostics_cmd() -> Result<AutoFixResult> {
    let bin_dir = crate::cli::get_devignite_root().join("bin");
    fix_broken_symlinks(&bin_dir)
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PathStatus {
    pub devignite_in_path: bool,
    pub path_entries: Vec<String>,
    pub potential_collisions: Vec<String>,
}

#[tauri::command]
pub fn get_system_path_status() -> Result<PathStatus> {
    let bin_dir_str = crate::cli::get_devignite_root().join("bin").to_string_lossy().to_string();

    let path_var = std::env::var("PATH").unwrap_or_default();
    let entries: Vec<String> = std::env::split_paths(&path_var)
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    let devignite_in_path = entries.iter().any(|e| e.contains(&bin_dir_str));
    let mut potential_collisions = Vec::new();

    for entry in &entries {
        if !entry.contains(".devignite") {
            let lower = entry.to_lowercase();
            if lower.contains("nodejs") || lower.contains("python") || lower.contains("go\\bin") || lower.contains("cargo\\bin") {
                potential_collisions.push(entry.clone());
            }
        }
    }

    Ok(PathStatus {
        devignite_in_path,
        path_entries: entries,
        potential_collisions,
    })
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ConfigExportPayload {
    pub export_version: u32,
    pub runtimes: Vec<Runtime>,
}

#[tauri::command]
pub fn export_runtimes_config(state: State<'_, DbState>) -> Result<String> {
    let runtimes = get_installed_runtimes(state)?;
    let payload = ConfigExportPayload {
        export_version: 1,
        runtimes,
    };
    serde_json::to_string_pretty(&payload).map_err(|e| crate::error::AppError::Unknown(e.into()))
}
