use std::path::PathBuf;
use std::sync::Arc;
use tauri::{Emitter, State};

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
use smoke_test::{SmokeTestConfig, SmokeTestRunner};

pub struct AppState {
    pub db: Arc<Database>,
    pub symlink_engine: Arc<SymlinkEngine>,
}

// ── Runtime Management Commands ─────────────────────────────────

#[tauri::command]
async fn get_installed_runtimes(state: State<'_, AppState>) -> Result<Vec<InstalledRuntime>, String> {
    state.db.get_all_runtimes().map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_runtimes_by_language(
    state: State<'_, AppState>, language: String,
) -> Result<Vec<InstalledRuntime>, String> {
    state.db.get_runtimes_by_language(&language).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_active_runtime(
    state: State<'_, AppState>, language: String,
) -> Result<Option<InstalledRuntime>, String> {
    state.db.get_active_runtime(&language).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_runtime_counts(
    state: State<'_, AppState>,
) -> Result<std::collections::HashMap<String, i32>, String> {
    let langs = vec!["python", "node", "rust", "go", "java", "gcc", "ruby", "deno"];
    let mut map = std::collections::HashMap::new();
    for lang in langs {
        let count = state.db.count_runtimes_by_language(lang).unwrap_or(0);
        map.insert(lang.to_string(), count);
    }
    Ok(map)
}

#[tauri::command]
async fn install_runtime(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    language: String,
    version: String,
    download_url: String,
    sha256: String,
    binary_name: String,
) -> Result<InstalledRuntime, String> {
    let _ = app.emit("install-progress", serde_json::json!({
        "language": language, "version": version, "stage": "download", "progress": 5, "message": "Starting download..."
    }));

    let downloader = SecureDownloader::new().map_err(|e| e.to_string())?;

    let engine = &state.symlink_engine;
    engine.ensure_runtime_dir(&language, &version).map_err(|e| e.to_string())?;

    let progress_engine = engine.clone();
    let progress_lang = language.clone();
    let progress_ver = version.clone();
    let app_handle = app.clone();

    let runtime_dir = downloader
        .download_and_install(
            &download_url, &sha256, &language, &version, &binary_name,
            Some(Box::new(move |pct, msg| {
                let _ = app_handle.emit("install-progress", serde_json::json!({
                    "language": progress_lang, "version": progress_ver,
                    "stage": if pct < 85 { "download" } else if pct < 95 { "extract" } else { "verify" },
                    "progress": pct, "message": msg
                }));
            })),
        )
        .await
        .map_err(|e| {
            let _ = app.emit("install-progress", serde_json::json!({
                "language": language, "version": version, "stage": "error", "progress": 0, "message": format!("Failed: {}", e)
            }));
            e.to_string()
        })?;

    let symlink_path = engine.create_symlink(&language, &version, &binary_name)
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

    let _ = state.db.set_active_runtime(&language, "");
    state.db.insert_runtime(&runtime).map_err(|e| e.to_string())?;
    state.db.set_active_runtime(&language, &runtime.id).map_err(|e| e.to_string())?;
    state.db.log_install_action(&language, &version, "install", "success", Some(&runtime.install_path)).ok();

    let _ = app.emit("install-progress", serde_json::json!({
        "language": language, "version": version, "stage": "complete", "progress": 100, "message": "Installation complete!"
    }));
    let _ = app.emit("runtimes-changed", ());

    Ok(runtime)
}

#[tauri::command]
async fn switch_version(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    language: String,
    runtime_id: String,
) -> Result<Vec<String>, String> {
    let runtime = state.db.get_runtime_by_id(&runtime_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Runtime {} not found", runtime_id))?;

    let symlinks = state.symlink_engine.switch_version(&language, &runtime.version)
        .map_err(|e| e.to_string())?;

    state.db.set_active_runtime(&language, &runtime_id).map_err(|e| e.to_string())?;
    state.db.log_install_action(&language, &runtime.version, "switch", "success", None).ok();

    let _ = app.emit("runtimes-changed", ());
    Ok(symlinks.into_iter().map(|p| p.display().to_string()).collect())
}

#[tauri::command]
async fn uninstall_runtime(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    language: String,
    runtime_id: String,
) -> Result<(), String> {
    let runtime = state.db.get_runtime_by_id(&runtime_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Runtime {} not found", runtime_id))?;

    state.symlink_engine.cleanup_runtime(&language, &runtime.version)
        .map_err(|e| e.to_string())?;

    state.db.delete_runtime(&runtime_id).map_err(|e| e.to_string())?;
    state.db.log_install_action(&language, &runtime.version, "uninstall", "success", None).ok();

    let _ = app.emit("runtimes-changed", ());
    Ok(())
}

// ── Smoke Testing Commands ──────────────────────────────────────

#[tauri::command]
async fn run_smoke_test(
    state: State<'_, AppState>,
    language: String,
    runtime_id: String,
) -> Result<smoke_test::SmokeTestSuiteResult, String> {
    let runtime = state.db.get_runtime_by_id(&runtime_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Runtime {} not found", runtime_id))?;

    let runner = SmokeTestRunner::new();
    let config = SmokeTestConfig::default_for_language(&language, &runtime.version, &runtime.install_path);
    runner.run_suite(&config).map_err(|e| e.to_string())
}

// ── Doctor Commands ─────────────────────────────────────────────

#[tauri::command]
async fn run_doctor(
    state: State<'_, AppState>,
    auto_fix: bool,
) -> Result<DocReport, String> {
    let engine = SymlinkEngine::new().map_err(|e| e.to_string())?;
    let doctor = DevIgniteDoctor::with_auto_fix(engine, auto_fix);
    let report = doctor.run_full_diagnostic().map_err(|e| e.to_string())?;

    let record = DoctorHistoryRecord {
        id: uuid::Uuid::new_v4().to_string(),
        run_at: report.timestamp.clone(),
        issues_found: (report.summary.warnings + report.summary.errors) as i32,
        issues_fixed: report.summary.fixed as i32,
        full_report: serde_json::to_string(&report).unwrap_or_default(),
    };
    state.db.insert_doctor_report(&record).ok();

    Ok(report)
}

#[tauri::command]
async fn fix_doctor_issue(
    state: State<'_, AppState>,
    issue_id: String,
) -> Result<sandbox::doctor::DiagnosticIssue, String> {
    let engine = SymlinkEngine::new().map_err(|e| e.to_string())?;
    let doctor = DevIgniteDoctor::with_auto_fix(engine, true);
    doctor.fix_specific(&issue_id).map_err(|e| e.to_string())?
        .ok_or_else(|| format!("No fix available for issue: {}", issue_id))
}

// ── Path Management Commands ────────────────────────────────────

#[tauri::command]
async fn get_path_status() -> Result<PathBroadcastStatus, String> {
    let pm = sandbox::path_manager::PathManager::new().map_err(|e| e.to_string())?;
    pm.verify_path_broadcast().map_err(|e| e.to_string())
}

#[tauri::command]
async fn ensure_path_injected() -> Result<bool, String> {
    let pm = sandbox::path_manager::PathManager::new().map_err(|e| e.to_string())?;
    pm.inject_sandbox_into_path().map_err(|e| e.to_string())
}

#[tauri::command]
async fn remove_path_injection() -> Result<(), String> {
    let pm = sandbox::path_manager::PathManager::new().map_err(|e| e.to_string())?;
    pm.remove_sandbox_from_path().map_err(|e| e.to_string())
}

// ── Verification & Utility Commands ─────────────────────────────

#[tauri::command]
async fn verify_checksum(file_path: String, expected_sha256: String) -> Result<bool, String> {
    let downloader = SecureDownloader::new().map_err(|e| e.to_string())?;
    downloader.verify_cached_file(&PathBuf::from(&file_path), &expected_sha256).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_system_health(state: State<'_, AppState>) -> Result<Vec<DiagnosticResult>, String> {
    state.db.get_system_health().map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_sandbox_status(state: State<'_, AppState>) -> Result<SandboxStatus, String> {
    let engine = &state.symlink_engine;
    let runtimes = state.db.get_all_runtimes().map_err(|e| e.to_string())?;
    let symlinks = engine.list_active_binaries();
    let disk = engine.disk_usage();
    let path_status = sandbox::path_manager::PathManager::new()
        .and_then(|pm| pm.verify_path_broadcast())
        .map(|s| s.in_user_path)
        .unwrap_or(false);

    Ok(SandboxStatus {
        home_dir: dirs::home_dir().map(|p| p.display().to_string()).unwrap_or_default(),
        sandbox_root: engine.sandbox_root().display().to_string(),
        bin_dir: engine.bin_dir().display().to_string(),
        runtimes_dir: engine.runtimes_dir().display().to_string(),
        total_runtimes: runtimes.len(),
        total_symlinks: symlinks.len(),
        path_injected: path_status,
        disk_usage_bytes: disk,
    })
}

#[tauri::command]
async fn get_install_history(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<(String, String, String, String, Option<String>, String)>, String> {
    state.db.get_install_history(limit.unwrap_or(50)).map_err(|e| e.to_string())
}

// ── Catalog Commands ────────────────────────────────────────────

#[tauri::command]
async fn get_catalog_for_language(
    state: State<'_, AppState>,
    language: String,
) -> Result<Vec<RuntimeCatalogEntry>, String> {
    let mut entries = state.db.get_catalog_by_language(&language).map_err(|e| e.to_string())?;
    let installed = state.db.get_runtimes_by_language(&language).map_err(|e| e.to_string())?;
    let installed_versions: std::collections::HashSet<String> = installed.into_iter().map(|r| r.version).collect();
    for entry in &mut entries {
        entry.is_installed = installed_versions.contains(&entry.version);
    }
    Ok(entries)
}

// ── App Entry Point ─────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_dir = app.path().app_data_dir().expect("Failed to resolve app data dir");
            std::fs::create_dir_all(&app_dir).ok();

            let db_path = app_dir.join("devignite.db");
            let db = Database::new(&db_path).expect("Failed to initialize database");
            let symlink_engine = SymlinkEngine::new().expect("Failed to initialize symlink engine");

            app.manage(AppState {
                db: Arc::new(db),
                symlink_engine: Arc::new(symlink_engine),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_installed_runtimes,
            get_runtimes_by_language,
            get_active_runtime,
            get_runtime_counts,
            install_runtime,
            switch_version,
            uninstall_runtime,
            run_smoke_test,
            run_doctor,
            fix_doctor_issue,
            get_path_status,
            ensure_path_injected,
            remove_path_injection,
            verify_checksum,
            get_system_health,
            get_sandbox_status,
            get_install_history,
            get_catalog_for_language,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
