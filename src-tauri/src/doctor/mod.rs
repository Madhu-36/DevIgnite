use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct DiagnosticReport {
    pub symlink_healthy: bool,
    pub symlink_target: Option<String>,
    pub devignite_in_path: bool,
    pub permissions_ok: bool,
    pub error_details: Vec<String>,
    pub auto_fix_available: bool,
    pub windows_developer_mode: bool,
    pub is_healthy: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AutoFixResult {
    pub fixed_items: Vec<String>,
    pub remaining_errors: Vec<String>,
    pub success: bool,
}

pub fn run_diagnostics(bin_dir: &PathBuf) -> Result<DiagnosticReport> {
    let mut broken_symlinks = Vec::new();
    let permission_issues = Vec::new();
    let path_collisions = Vec::new();

    if bin_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(bin_dir) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_symlink() {
                        let path = entry.path();
                        if !path.exists() {
                            broken_symlinks.push(path.to_string_lossy().into_owned());
                        }
                    }
                }
            }
        }
    }

    let is_healthy = broken_symlinks.is_empty() && permission_issues.is_empty() && path_collisions.is_empty();

    let mut error_details = Vec::new();
    if !broken_symlinks.is_empty() {
        error_details.push(format!("Broken symlinks detected: {:?}", broken_symlinks));
    }

    #[cfg(windows)]
    let windows_developer_mode = {
        use winreg::enums::HKEY_LOCAL_MACHINE;
        use winreg::RegKey;
        RegKey::predef(HKEY_LOCAL_MACHINE)
            .open_subkey("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\AppModelUnlock")
            .and_then(|k| k.get_value::<u32, _>("AllowDevelopmentWithoutDevLicense"))
            .map(|val| val == 1)
            .unwrap_or(false)
    };
    #[cfg(not(windows))]
    let windows_developer_mode = true;
    
    Ok(DiagnosticReport {
        symlink_healthy: broken_symlinks.is_empty(),
        symlink_target: None,
        devignite_in_path: true,
        permissions_ok: permission_issues.is_empty(),
        error_details,
        auto_fix_available: !broken_symlinks.is_empty(),
        windows_developer_mode,
        is_healthy,
    })
}

pub fn fix_broken_symlinks(bin_dir: &PathBuf) -> Result<AutoFixResult> {
    let mut fixed_items = Vec::new();
    let mut remaining_errors = Vec::new();

    if bin_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(bin_dir) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_symlink() {
                        let path = entry.path();
                        if !path.exists() {
                            // Attempt removal of broken symlink
                            let path_str = path.to_string_lossy().to_string();
                            if let Err(e) = std::fs::remove_file(&path) {
                                remaining_errors.push(format!("Failed to remove {}: {}", path_str, e));
                            } else {
                                fixed_items.push(path_str);
                            }
                        }
                    }
                }
            }
        }
    }

    let success = remaining_errors.is_empty();
    Ok(AutoFixResult {
        fixed_items,
        remaining_errors,
        success,
    })
}
