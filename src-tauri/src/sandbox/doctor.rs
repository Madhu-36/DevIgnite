use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::symlink_engine::SymlinkEngine;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DiagnosticIssue {
    pub category: String,
    pub severity: IssueSeverity,
    pub description: String,
    pub affected_path: Option<String>,
    pub fixable: bool,
    pub fix_applied: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum IssueSeverity {
    Ok,
    Warning,
    Error,
    Fixed,
}

impl std::fmt::Display for IssueSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IssueSeverity::Ok => write!(f, "ok"),
            IssueSeverity::Warning => write!(f, "warning"),
            IssueSeverity::Error => write!(f, "error"),
            IssueSeverity::Fixed => write!(f, "fixed"),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DoctorReport {
    pub timestamp: String,
    pub issues: Vec<DiagnosticIssue>,
    pub summary: DoctorSummary,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DoctorSummary {
    pub total_checks: usize,
    pub passed: usize,
    pub warnings: usize,
    pub errors: usize,
    pub fixed: usize,
}

pub struct DevIgniteDoctor {
    symlink_engine: SymlinkEngine,
}

impl DevIgniteDoctor {
    pub fn new(symlink_engine: SymlinkEngine) -> Self {
        Self { symlink_engine }
    }

    pub fn run_full_diagnostic(&self) -> Result<DoctorReport> {
        let mut issues = Vec::new();

        issues.extend(self.check_sandbox_integrity()?);
        issues.extend(self.check_symlink_health()?);
        issues.extend(self.check_shadowed_binaries()?);
        issues.extend(self.check_orphaned_temp_files()?);
        issues.extend(self.check_path_state()?);
        issues.extend(self.check_disk_space()?);

        let summary = DoctorSummary {
            total_checks: issues.len(),
            passed: issues
                .iter()
                .filter(|i| matches!(i.severity, IssueSeverity::Ok))
                .count(),
            warnings: issues
                .iter()
                .filter(|i| matches!(i.severity, IssueSeverity::Warning))
                .count(),
            errors: issues
                .iter()
                .filter(|i| matches!(i.severity, IssueSeverity::Error))
                .count(),
            fixed: issues
                .iter()
                .filter(|i| matches!(i.severity, IssueSeverity::Fixed))
                .count(),
        };

        Ok(DoctorReport {
            timestamp: chrono::Utc::now().to_rfc3339(),
            issues,
            summary,
        })
    }

    fn check_sandbox_integrity(&self) -> Result<Vec<DiagnosticIssue>> {
        let mut issues = Vec::new();
        let bin_dir = self.symlink_engine.bin_dir();

        if !bin_dir.exists() {
            issues.push(DiagnosticIssue {
                category: "sandbox_integrity".to_string(),
                severity: IssueSeverity::Error,
                description: "Sandbox bin directory does not exist".to_string(),
                affected_path: Some(bin_dir.display().to_string()),
                fixable: true,
                fix_applied: false,
            });
            fs::create_dir_all(bin_dir)?;
            if let Some(last) = issues.last_mut() {
                last.severity = IssueSeverity::Fixed;
                last.fix_applied = true;
            }
        } else {
            issues.push(DiagnosticIssue {
                category: "sandbox_integrity".to_string(),
                severity: IssueSeverity::Ok,
                description: "Sandbox bin directory exists and is accessible".to_string(),
                affected_path: Some(bin_dir.display().to_string()),
                fixable: false,
                fix_applied: false,
            });
        }

        let runtimes_dir = self.symlink_engine.runtimes_dir();
        if !runtimes_dir.exists() {
            issues.push(DiagnosticIssue {
                category: "sandbox_integrity".to_string(),
                severity: IssueSeverity::Error,
                description: "Runtimes directory does not exist".to_string(),
                affected_path: Some(runtimes_dir.display().to_string()),
                fixable: true,
                fix_applied: false,
            });
            fs::create_dir_all(runtimes_dir)?;
            if let Some(last) = issues.last_mut() {
                last.severity = IssueSeverity::Fixed;
                last.fix_applied = true;
            }
        } else {
            issues.push(DiagnosticIssue {
                category: "sandbox_integrity".to_string(),
                severity: IssueSeverity::Ok,
                description: "Runtimes directory exists".to_string(),
                affected_path: Some(runtimes_dir.display().to_string()),
                fixable: false,
                fix_applied: false,
            });
        }

        Ok(issues)
    }

    fn check_symlink_health(&self) -> Result<Vec<DiagnosticIssue>> {
        let mut issues = Vec::new();

        match self.symlink_engine.verify_symlinks() {
            Ok(health_list) => {
                for health in &health_list {
                    if health.is_valid {
                        issues.push(DiagnosticIssue {
                            category: "symlink_health".to_string(),
                            severity: IssueSeverity::Ok,
                            description: format!(
                                "Symlink OK: {} -> {}",
                                health.symlink_path.display(),
                                health.target_path.display()
                            ),
                            affected_path: Some(health.symlink_path.display().to_string()),
                            fixable: false,
                            fix_applied: false,
                        });
                    } else {
                        issues.push(DiagnosticIssue {
                            category: "symlink_health".to_string(),
                            severity: IssueSeverity::Warning,
                            description: format!(
                                "Broken symlink: {} -> {} (target missing)",
                                health.symlink_path.display(),
                                health.target_path.display()
                            ),
                            affected_path: Some(health.symlink_path.display().to_string()),
                            fixable: true,
                            fix_applied: false,
                        });
                    }
                }
            }
            Err(e) => {
                issues.push(DiagnosticIssue {
                    category: "symlink_health".to_string(),
                    severity: IssueSeverity::Error,
                    description: format!("Failed to verify symlinks: {}", e),
                    affected_path: None,
                    fixable: false,
                    fix_applied: false,
                });
            }
        }

        Ok(issues)
    }

    fn check_shadowed_binaries(&self) -> Result<Vec<DiagnosticIssue>> {
        let mut issues = Vec::new();
        let sandbox_bin = self.symlink_engine.bin_dir();
        let system_path = std::env::var("PATH").unwrap_or_default();

        let mut system_binaries: HashMap<String, PathBuf> = HashMap::new();

        for dir in system_path.split(';') {
            let dir = dir.trim();
            let path = Path::new(dir);
            if !path.exists() || path == sandbox_bin {
                continue;
            }

            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let name = entry.file_name();
                    if let Some(name_str) = name.to_str() {
                        let stem = name_str.trim_end_matches(".exe");
                        system_binaries
                            .insert(stem.to_string(), entry.path());
                    }
                }
            }
        }

        if let Ok(entries) = fs::read_dir(sandbox_bin) {
            for entry in entries.filter_map(|e| e.ok()) {
                let name = entry.file_name();
                if let Some(name_str) = name.to_str() {
                    let stem = name_str.trim_end_matches(".exe");
                    if let Some(system_path) = system_binaries.get(stem) {
                        issues.push(DiagnosticIssue {
                            category: "shadowed_binary".to_string(),
                            severity: IssueSeverity::Warning,
                            description: format!(
                                "System binary '{}' at {} may shadow DevIgnite's managed version",
                                stem,
                                system_path.display()
                            ),
                            affected_path: Some(system_path.display().to_string()),
                            fixable: false,
                            fix_applied: false,
                        });
                    }
                }
            }
        }

        if issues.is_empty() {
            issues.push(DiagnosticIssue {
                category: "shadowed_binary".to_string(),
                severity: IssueSeverity::Ok,
                description: "No shadowed binaries detected".to_string(),
                affected_path: None,
                fixable: false,
                fix_applied: false,
            });
        }

        Ok(issues)
    }

    fn check_orphaned_temp_files(&self) -> Result<Vec<DiagnosticIssue>> {
        let mut issues = Vec::new();
        let sandbox_root = self.symlink_engine.sandbox_root();
        let temp_dir = sandbox_root.join("temp");

        if temp_dir.exists() {
            let mut orphaned_count = 0;
            let mut orphaned_size: u64 = 0;

            for entry in walkdir::WalkDir::new(&temp_dir)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.file_type().is_file() {
                    orphaned_count += 1;
                    if let Ok(meta) = entry.metadata() {
                        orphaned_size += meta.len();
                    }
                }
            }

            if orphaned_count > 0 {
                issues.push(DiagnosticIssue {
                    category: "orphaned_temp".to_string(),
                    severity: IssueSeverity::Warning,
                    description: format!(
                        "Found {} orphaned temp files ({:.2} MB) in {}",
                        orphaned_count,
                        orphaned_size as f64 / 1_048_576.0,
                        temp_dir.display()
                    ),
                    affected_path: Some(temp_dir.display().to_string()),
                    fixable: true,
                    fix_applied: false,
                });

                fs::remove_dir_all(&temp_dir)?;
                fs::create_dir_all(&temp_dir)?;
                if let Some(last) = issues.last_mut() {
                    last.severity = IssueSeverity::Fixed;
                    last.fix_applied = true;
                }
            } else {
                issues.push(DiagnosticIssue {
                    category: "orphaned_temp".to_string(),
                    severity: IssueSeverity::Ok,
                    description: "No orphaned temp files found".to_string(),
                    affected_path: None,
                    fixable: false,
                    fix_applied: false,
                });
            }
        } else {
            issues.push(DiagnosticIssue {
                category: "orphaned_temp".to_string(),
                severity: IssueSeverity::Ok,
                description: "No temp directory exists".to_string(),
                affected_path: None,
                fixable: false,
                fix_applied: false,
            });
        }

        Ok(issues)
    }

    fn check_path_state(&self) -> Result<Vec<DiagnosticIssue>> {
        let mut issues = Vec::new();
        let sandbox_str = self
            .symlink_engine
            .bin_dir()
            .display()
            .to_string();
        let current_path = std::env::var("PATH").unwrap_or_default();

        let in_path = current_path
            .split(';')
            .any(|p| p.trim().eq_ignore_ascii_case(&sandbox_str));

        if in_path {
            issues.push(DiagnosticIssue {
                category: "path_state".to_string(),
                severity: IssueSeverity::Ok,
                description: "Sandbox bin directory is in system PATH".to_string(),
                affected_path: Some(sandbox_str),
                fixable: false,
                fix_applied: false,
            });
        } else {
            issues.push(DiagnosticIssue {
                category: "path_state".to_string(),
                severity: IssueSeverity::Error,
                description: "Sandbox bin directory is NOT in system PATH".to_string(),
                affected_path: Some(sandbox_str),
                fixable: true,
                fix_applied: false,
            });
        }

        Ok(issues)
    }

    fn check_disk_space(&self) -> Result<Vec<DiagnosticIssue>> {
        let mut issues = Vec::new();
        let runtimes_dir = self.symlink_engine.runtimes_dir();

        if runtimes_dir.exists() {
            let mut total_size: u64 = 0;
            for entry in walkdir::WalkDir::new(runtimes_dir)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if let Ok(meta) = entry.metadata() {
                    total_size += meta.len();
                }
            }

            let size_mb = total_size as f64 / 1_048_576.0;
            let severity = if size_mb > 10_000.0 {
                IssueSeverity::Warning
            } else {
                IssueSeverity::Ok
            };

            issues.push(DiagnosticIssue {
                category: "disk_usage".to_string(),
                severity,
                description: format!("Total runtime disk usage: {:.2} MB", size_mb),
                affected_path: Some(runtimes_dir.display().to_string()),
                fixable: false,
                fix_applied: false,
            });
        }

        Ok(issues)
    }
}
