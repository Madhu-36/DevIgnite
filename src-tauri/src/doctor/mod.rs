use crate::error::Result;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Serialize)]
pub struct DiagnosticReport {
    pub path_collisions: Vec<String>,
    pub broken_symlinks: Vec<String>,
    pub permission_issues: Vec<String>,
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

    Ok(DiagnosticReport {
        path_collisions,
        broken_symlinks,
        permission_issues,
        is_healthy,
    })
}
