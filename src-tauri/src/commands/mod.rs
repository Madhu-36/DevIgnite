use crate::db::{DbState, Runtime};
use crate::doctor::{run_diagnostics, DiagnosticReport};
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
    let home_dir = directories::UserDirs::new()
        .map(|dirs| dirs.home_dir().to_path_buf())
        .ok_or_else(|| crate::error::AppError::Installation("No home directory found".into()))?;
        
    let devignite_dir = home_dir.join(".devignite");
    let cache_dir = devignite_dir.join("cache");
    let runtime_dir = devignite_dir.join("runtimes").join(&name).join(&version);
    
    if !cache_dir.exists() {
        std::fs::create_dir_all(&cache_dir)?;
    }
    
    // Resolve URL (Mocking for now based on Node.js/Python, assuming win-x64)
    let url = match name.as_str() {
        "node" => format!("https://nodejs.org/dist/v{0}/node-v{0}-win-x64.zip", version),
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
pub fn switch_version(name: String, version: String, _state: State<'_, DbState>) -> Result<()> {
    // Stub for symlink switch, PATH broadcast, db update
    println!("Switching {} to version {}", name, version);
    Ok(())
}

#[tauri::command]
pub fn uninstall_runtime(name: String, version: String, _state: State<'_, DbState>) -> Result<()> {
    // Stub for removal, db update
    println!("Uninstalling {} version {}", name, version);
    Ok(())
}

#[tauri::command]
pub fn run_diagnostics_cmd() -> Result<DiagnosticReport> {
    let bin_dir = directories::UserDirs::new()
        .map(|dirs| dirs.home_dir().join(".devignite").join("bin"))
        .unwrap_or_else(|| std::path::PathBuf::from(".devignite/bin"));
        
    run_diagnostics(&bin_dir)
}
