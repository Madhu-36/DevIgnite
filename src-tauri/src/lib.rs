pub mod cli;
pub mod commands;
pub mod db;
pub mod doctor;
pub mod error;
pub mod installer;
pub mod sandbox;

use db::DbState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let devignite_dir = cli::get_devignite_root();
    
    let db_path = devignite_dir.join("devignite.db");
    
    let db_conn = db::init_db(db_path).expect("Failed to initialize database");
    
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(DbState {
            conn: std::sync::Mutex::new(db_conn),
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_installed_runtimes,
            commands::install_runtime,
            commands::switch_version,
            commands::uninstall_runtime,
            commands::get_system_config,
            commands::set_system_config,
            commands::clear_cache,
            commands::run_diagnostics_cmd,
            commands::auto_fix_diagnostics_cmd,
            commands::get_system_path_status,
            commands::export_runtimes_config,
            commands::list_available_remote_versions,
            commands::get_disk_usage_stats,
            commands::get_switch_history,
            commands::optimize_database
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
