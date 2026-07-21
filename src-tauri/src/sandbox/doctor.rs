use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::symlink_engine::SymlinkEngine;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DiagnosticIssue {
    pub id: String,
    pub category: String,
    pub severity: IssueSeverity,
    pub description: String,
    pub affected_path: Option<String>,
    pub fixable: bool,
    pub fix_applied: bool,
    pub fix_details: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
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
    auto_fix: bool,
}

impl DevIgniteDoctor {
    pub fn new(symlink_engine: SymlinkEngine) -> Self {
        Self { symlink_engine, auto_fix: true }
    }

    pub fn with_auto_fix(symlink_engine: SymlinkEngine, auto_fix: bool) -> Self {
        Self { symlink_engine, auto_fix }
    }

    pub fn run_full_diagnostic(&self) -> Result<DoctorReport> {
        let mut issues = Vec::new();

        issues.extend(self.check_sandbox_integrity()?);
        issues.extend(self.check_symlink_health()?);
        issues.extend(self.check_shadowed_binaries()?);
        issues.extend(self.check_orphaned_temp_files()?);
        issues.extend(self.check_path_state()?);
        issues.extend(self.check_disk_space()?);
        issues.extend(self.check_db_integrity()?);

        let summary = DoctorSummary {
            total_checks: issues.len(),
            passed: issues.iter().filter(|i| i.severity == IssueSeverity::Ok).count(),
            warnings: issues.iter().filter(|i| i.severity == IssueSeverity::Warning).count(),
            errors: issues.iter().filter(|i| i.severity == IssueSeverity::Error).count(),
            fixed: issues.iter().filter(|i| i.severity == IssueSeverity::Fixed).count(),
        };

        Ok(DoctorReport { timestamp: chrono::Utc::now().to_rfc3339(), issues, summary })
    }

    pub fn fix_specific(&self, issue_id: &str) -> Result<Option<DiagnosticIssue>> {
        if issue_id.starts_with("sandbox_integrity") {
            let bin_dir = self.symlink_engine.bin_dir();
            let runtimes_dir = self.symlink_engine.runtimes_dir();
            if !bin_dir.exists() { fs::create_dir_all(bin_dir)?; }
            if !runtimes_dir.exists() { fs::create_dir_all(runtimes_dir)?; }
            return Ok(Some(DiagnosticIssue {
                id: issue_id.to_string(),
                category: "sandbox_integrity".to_string(),
                severity: IssueSeverity::Fixed,
                description: "Sandbox directories recreated".to_string(),
                affected_path: Some(bin_dir.display().to_string()),
                fixable: true,
                fix_applied: true,
                fix_details: Some("Created missing directories".to_string()),
            }));
        }

        if issue_id.starts_with("broken_symlink") {
            let parts: Vec<&str> = issue_id.split('|').collect();
            if parts.len() >= 2 {
                let symlink = Path::new(parts[1]);
                let _ = super::symlink_engine::remove_symlink_or_file(symlink);
                return Ok(Some(DiagnosticIssue {
                    id: issue_id.to_string(),
                    category: "symlink_health".to_string(),
                    severity: IssueSeverity::Fixed,
                    description: format!("Removed broken symlink: {}", symlink.display()),
                    affected_path: Some(symlink.display().to_string()),
                    fixable: true,
                    fix_applied: true,
                    fix_details: Some("Deleted orphaned symlink".to_string()),
                }));
            }
        }

        if issue_id.starts_with("orphaned_temp") {
            let temp_dir = self.symlink_engine.sandbox_root().join("temp");
            if temp_dir.exists() {
                let count = walkdir::WalkDir::new(&temp_dir).into_iter().filter_map(|e| e.ok()).filter(|e| e.file_type().is_file()).count();
                fs::remove_dir_all(&temp_dir)?;
                fs::create_dir_all(&temp_dir)?;
                return Ok(Some(DiagnosticIssue {
                    id: issue_id.to_string(),
                    category: "orphaned_temp".to_string(),
                    severity: IssueSeverity::Fixed,
                    description: format!("Cleaned {} orphaned temp files", count),
                    affected_path: Some(temp_dir.display().to_string()),
                    fixable: true,
                    fix_applied: true,
                    fix_details: Some("Temp directory cleared and recreated".to_string()),
                }));
            }
        }

        Ok(None)
    }

    fn check_sandbox_integrity(&self) -> Result<Vec<DiagnosticIssue>> {
        let mut issues = Vec::new();
        let bin_dir = self.symlink_engine.bin_dir();
        let runtimes_dir = self.symlink_engine.runtimes_dir();

        if bin_dir.exists() {
            issues.push(make_issue("sandbox_integrity_bin", "sandbox_integrity", IssueSeverity::Ok,
                "Sandbox bin directory exists", Some(bin_dir.display().to_string()), false));
        } else if self.auto_fix {
            fs::create_dir_all(bin_dir)?;
            issues.push(make_issue("sandbox_integrity_bin", "sandbox_integrity", IssueSeverity::Fixed,
                "Sandbox bin directory was missing — created", Some(bin_dir.display().to_string()), true));
        } else {
            issues.push(make_issue("sandbox_integrity_bin", "sandbox_integrity", IssueSeverity::Error,
                "Sandbox bin directory does not exist", Some(bin_dir.display().to_string()), true));
        }

        if runtimes_dir.exists() {
            issues.push(make_issue("sandbox_integrity_rt", "sandbox_integrity", IssueSeverity::Ok,
                "Runtimes directory exists", Some(runtimes_dir.display().to_string()), false));
        } else if self.auto_fix {
            fs::create_dir_all(runtimes_dir)?;
            issues.push(make_issue("sandbox_integrity_rt", "sandbox_integrity", IssueSeverity::Fixed,
                "Runtimes directory was missing — created", Some(runtimes_dir.display().to_string()), true));
        } else {
            issues.push(make_issue("sandbox_integrity_rt", "sandbox_integrity", IssueSeverity::Error,
                "Runtimes directory does not exist", Some(runtimes_dir.display().to_string()), true));
        }

        Ok(issues)
    }

    fn check_symlink_health(&self) -> Result<Vec<DiagnosticIssue>> {
        let mut issues = Vec::new();
        match self.symlink_engine.verify_symlinks() {
            Ok(health_list) => {
                if health_list.is_empty() {
                    issues.push(make_issue("symlink_empty", "symlink_health", IssueSeverity::Ok,
                        "No symlinks found (clean state)", None, false));
                }
                for h in &health_list {
                    if h.is_valid {
                        issues.push(make_issue(&format!("symlink_ok_{}", h.binary_name), "symlink_health", IssueSeverity::Ok,
                            format!("{} -> {}", h.symlink_path.display(), h.target_path.display()),
                            Some(h.symlink_path.display().to_string()), false));
                    } else {
                        let fix_id = format!("broken_symlink|{}", h.symlink_path.display());
                        issues.push(make_issue(&fix_id, "symlink_health", IssueSeverity::Warning,
                            format!("Broken: {} (target missing)", h.symlink_path.display()),
                            Some(h.symlink_path.display().to_string()), true));
                    }
                }
            }
            Err(e) => {
                issues.push(make_issue("symlink_err", "symlink_health", IssueSeverity::Error,
                    format!("Failed to verify symlinks: {}", e), None, false));
            }
        }
        Ok(issues)
    }

    fn check_shadowed_binaries(&self) -> Result<Vec<DiagnosticIssue>> {
        let mut issues = Vec::new();
        let sandbox_bin = self.symlink_engine.bin_dir();
        let system_path = std::env::var("PATH").unwrap_or_default();
        let mut system_bins: HashMap<String, PathBuf> = HashMap::new();

        for dir in system_path.split(';') {
            let dir = dir.trim();
            let path = Path::new(dir);
            if !path.exists() || path == sandbox_bin { continue; }
            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.filter_map(|e| e.ok()) {
                    if let Some(name) = entry.file_name().to_str() {
                        let stem = name.trim_end_matches(".exe");
                        system_bins.insert(stem.to_string(), entry.path());
                    }
                }
            }
        }

        let mut shadowed_count = 0;
        if let Ok(entries) = fs::read_dir(sandbox_bin) {
            for entry in entries.filter_map(|e| e.ok()) {
                if let Some(name) = entry.file_name().to_str() {
                    let stem = name.trim_end_matches(".exe");
                    if let Some(sys_path) = system_bins.get(stem) {
                        shadowed_count += 1;
                        issues.push(make_issue(
                            &format!("shadow_{}_{}", stem, shadowed_count),
                            "shadowed_binary", IssueSeverity::Warning,
                            format!("System '{}' at {} may shadow sandboxed version", stem, sys_path.display()),
                            Some(sys_path.display().to_string()), false));
                    }
                }
            }
        }

        if issues.is_empty() {
            issues.push(make_issue("shadow_none", "shadowed_binary", IssueSeverity::Ok,
                "No shadowed binaries detected", None, false));
        }
        Ok(issues)
    }

    fn check_orphaned_temp_files(&self) -> Result<Vec<DiagnosticIssue>> {
        let mut issues = Vec::new();
        let temp_dir = self.symlink_engine.sandbox_root().join("temp");
        if temp_dir.exists() {
            let (count, size) = count_temp_files(&temp_dir);
            if count > 0 {
                let fix_id = format!("orphaned_temp|{}", temp_dir.display());
                issues.push(make_issue(&fix_id, "orphaned_temp", IssueSeverity::Warning,
                    format!("{} orphaned temp files ({:.2} MB)", count, size as f64 / 1_048_576.0),
                    Some(temp_dir.display().to_string()), true));
            } else {
                issues.push(make_issue("temp_clean", "orphaned_temp", IssueSeverity::Ok,
                    "No orphaned temp files", None, false));
            }
        } else {
            issues.push(make_issue("temp_none", "orphaned_temp", IssueSeverity::Ok,
                "No temp directory exists", None, false));
        }
        Ok(issues)
    }

    fn check_path_state(&self) -> Result<Vec<DiagnosticIssue>> {
        let mut issues = Vec::new();
        let sandbox_str = self.symlink_engine.bin_dir().display().to_string();
        let current = std::env::var("PATH").unwrap_or_default();
        if current.split(';').any(|p| p.trim().eq_ignore_ascii_case(&sandbox_str)) {
            issues.push(make_issue("path_ok", "path_state", IssueSeverity::Ok,
                "Sandbox bin is in system PATH", Some(sandbox_str), false));
        } else {
            issues.push(make_issue("path_missing", "path_state", IssueSeverity::Error,
                "Sandbox bin is NOT in system PATH", Some(sandbox_str), true));
        }
        Ok(issues)
    }

    fn check_disk_space(&self) -> Result<Vec<DiagnosticIssue>> {
        let mut issues = Vec::new();
        let usage = self.symlink_engine.disk_usage();
        let mb = usage as f64 / 1_048_576.0;
        let sev = if mb > 10_000.0 { IssueSeverity::Warning } else { IssueSeverity::Ok };
        issues.push(make_issue("disk_usage", "disk_usage", sev,
            format!("Total runtime disk usage: {:.2} MB", mb),
            Some(self.symlink_engine.runtimes_dir().display().to_string()), false));
        Ok(issues)
    }

    fn check_db_integrity(&self) -> Result<Vec<DiagnosticIssue>> {
        let mut issues = Vec::new();
        let db_path = dirs::home_dir()
            .map(|h| h.join(".devignite").join("devignite.db"))
            .unwrap_or_default();
        if db_path.exists() {
            issues.push(make_issue("db_exists", "database", IssueSeverity::Ok,
                "SQLite database file exists", Some(db_path.display().to_string()), false));
        } else {
            issues.push(make_issue("db_missing", "database", IssueSeverity::Ok,
                "No database file yet (will be created on first run)", None, false));
        }
        Ok(issues)
    }
}

fn make_issue(id: &str, category: &str, severity: IssueSeverity, description: String, affected_path: Option<String>, fixable: bool) -> DiagnosticIssue {
    DiagnosticIssue {
        id: id.to_string(),
        category: category.to_string(),
        severity,
        description,
        affected_path,
        fixable,
        fix_applied: severity == IssueSeverity::Fixed,
        fix_details: if severity == IssueSeverity::Fixed { Some("Auto-fixed".to_string()) } else { None },
    }
}

fn count_temp_files(dir: &Path) -> (usize, u64) {
    let mut count = 0;
    let mut size = 0u64;
    for entry in walkdir::WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            count += 1;
            if let Ok(meta) = entry.metadata() { size += meta.len(); }
        }
    }
    (count, size)
}
