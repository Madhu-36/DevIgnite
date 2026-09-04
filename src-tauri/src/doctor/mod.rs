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
    pub is_healthy: bool,
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
        error_details.push(format!("Broken symlinks: {:?}", broken_symlinks));
    }
    
    Ok(DiagnosticReport {
        symlink_healthy: broken_symlinks.is_empty(),
        symlink_target: None,
        devignite_in_path: true, // Mocked for now
        permissions_ok: permission_issues.is_empty(),
        error_details,
        auto_fix_available: !broken_symlinks.is_empty(),
        is_healthy,
    })
}
