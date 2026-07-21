use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;

pub mod db;
pub mod download;
pub mod sandbox;
pub mod smoke_test;

use db::models::*;
use db::Database;
use download::SecureDownloader;
use sandbox::doctor::{DevIgniteDoctor, DoctorReport as DocReport};
use sandbox::path_manager::PathBroadcastStatus;
use sandbox::SymlinkEngine;
use smoke_test::SmokeTestRunner;
use smoke_test::SmokeTestConfig;

pub struct AppState {
    pub db: Arc<Database>,
    pub symlink_engine: Arc<SymlinkEngine>,
}

#[tauri::command]
async fn get_installed_runtimes(state: State<'_, AppState>) -> Result<Vec<InstalledRuntime>, String> {
    state
        .db
        .get_all_runtimes()
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_runtimes_by_language(
    state: State<'_, AppState>,
    language: String,
) -> Result<Vec<InstalledRuntime>, String> {
    state
        .db
        .get_runtimes_by_language(&language)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_active_runtime(
    state: State<'_, AppState>,
    language: String,
) -> Result<Option<InstalledRuntime>, String> {
    state
        .db
        .get_active_runtime(&language)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn install_runtime(
    state: State<'_, AppState>,
    language: String,
    version: String,
    download_url: String,
    sha256: String,
    binary_name: String,
) -> Result<InstalledRuntime, String> {
    let downloader = SecureDownloader::new().map_err(|e| e.to_string())?;

    let runtime_dir = downloader
        .download_and_install(
            &download_url,
            &sha256,
            &language,
            &version,
            &binary_name,
            None,
        )
        .await
        .map_err(|e| e.to_string())?;

    let symlink_engine = &state.symlink_engine;
    let symlink_path = symlink_engine
        .create_symlink(&language, &version, &binary_name)
        .map_err(|e| e.to_string())?;

    let runtime = InstalledRuntime {
        id: uuid::Uuid::new_v4().to_string(),
        language: language.clone(),
        version: version.clone(),
        install_path: runtime_dir.display().to_string(),
        symlink_path: symlink_path.display().to_string(),
        installed_at: chrono::Utc::now().to_rfc3339(),
        is_active: true,
        checksum_verified: true,
        binary_name: binary_name.clone(),
    };

    state
        .db
        .set_active_runtime(&language, "")
        .map_err(|e| e.to_string())?;

    state
        .db
        .insert_runtime(&runtime)
        .map_err(|e| e.to_string())?;

    state
        .db
        .set_active_runtime(&language, &runtime.id)
        .map_err(|e| e.to_string())?;

    Ok(runtime)
}

#[tauri::command]
async fn switch_version(
    state: State<'_, AppState>,
    language: String,
    runtime_id: String,
    version: String,
) -> Result<Vec<String>, String> {
    let _runtime = state
        .db
        .get_runtimes_by_language(&language)
        .map_err(|e| e.to_string())?
        .into_iter()
        .find(|r| r.id == runtime_id)
        .ok_or_else(|| format!("Runtime {} not found", runtime_id))?;

    let symlink_engine = &state.symlink_engine;
    let symlinks = symlink_engine
        .switch_version(&language, &version)
        .map_err(|e| e.to_string())?;

    state
        .db
        .set_active_runtime(&language, &runtime_id)
        .map_err(|e| e.to_string())?;

    Ok(symlinks.into_iter().map(|p| p.display().to_string()).collect())
}

#[tauri::command]
async fn uninstall_runtime(
    state: State<'_, AppState>,
    language: String,
    runtime_id: String,
) -> Result<(), String> {
    let runtimes = state
        .db
        .get_runtimes_by_language(&language)
        .map_err(|e| e.to_string())?;

    let runtime = runtimes
        .into_iter()
        .find(|r| r.id == runtime_id)
        .ok_or_else(|| format!("Runtime {} not found", runtime_id))?;

    let symlink_engine = &state.symlink_engine;
    symlink_engine
        .cleanup_runtime(&runtime.language, &runtime.version)
        .map_err(|e| e.to_string())?;

    state
        .db
        .delete_runtime(&runtime_id)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
async fn run_smoke_test(
    state: State<'_, AppState>,
    language: String,
    runtime_id: String,
) -> Result<smoke_test::SmokeTestSuiteResult, String> {
    let runtime = state
        .db
        .get_runtimes_by_language(&language)
        .map_err(|e| e.to_string())?
        .into_iter()
        .find(|r| r.id == runtime_id)
        .ok_or_else(|| format!("Runtime {} not found", runtime_id))?;

    let runner = SmokeTestRunner::new();
    let config = SmokeTestConfig::default_for_language(
        &runtime.language,
        &runtime.version,
        &runtime.install_path,
    );

    runner.run_suite(&config).map_err(|e| e.to_string())
}

#[tauri::command]
async fn run_doctor(state: State<'_, AppState>) -> Result<DocReport, String> {
    let doctor = DevIgniteDoctor::new(
        SymlinkEngine::new().map_err(|e| e.to_string())?,
    );
    doctor.run_full_diagnostic().map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_path_status() -> Result<PathBroadcastStatus, String> {
    let path_manager =
        sandbox::path_manager::PathManager::new().map_err(|e| e.to_string())?;
    path_manager
        .verify_path_broadcast()
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn ensure_path_injected() -> Result<bool, String> {
    let path_manager =
        sandbox::path_manager::PathManager::new().map_err(|e| e.to_string())?;
    path_manager
        .inject_sandbox_into_path()
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn verify_checksum(
    file_path: String,
    expected_sha256: String,
) -> Result<bool, String> {
    let downloader = SecureDownloader::new().map_err(|e| e.to_string())?;
    downloader
        .verify_cached_file(&PathBuf::from(&file_path), &expected_sha256)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_system_health(
    state: State<'_, AppState>,
) -> Result<Vec<DiagnosticResult>, String> {
    state
        .db
        .get_system_health()
        .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to resolve app data directory");
            std::fs::create_dir_all(&app_dir).ok();

            let db_path = app_dir.join("devignite.db");
            let db = Database::new(&db_path).expect("Failed to initialize database");

            let symlink_engine =
                SymlinkEngine::new().expect("Failed to initialize symlink engine");

            let state = AppState {
                db: Arc::new(db),
                symlink_engine: Arc::new(symlink_engine),
            };

            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_installed_runtimes,
            get_runtimes_by_language,
            get_active_runtime,
            install_runtime,
            switch_version,
            uninstall_runtime,
            run_smoke_test,
            run_doctor,
            get_path_status,
            ensure_path_injected,
            verify_checksum,
            get_system_health,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
