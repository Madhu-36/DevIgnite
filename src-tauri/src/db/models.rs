use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledRuntime {
    pub id: String,
    pub language: String,
    pub version: String,
    pub install_path: String,
    pub symlink_path: String,
    pub installed_at: String,
    pub is_active: bool,
    pub checksum_verified: bool,
    pub binary_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadCacheEntry {
    pub id: String,
    pub url: String,
    pub local_path: String,
    pub sha256: String,
    pub downloaded_at: String,
    pub size_bytes: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticResult {
    pub id: String,
    pub check_name: String,
    pub status: String,
    pub message: String,
    pub detected_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathState {
    pub id: String,
    pub sandbox_bin_path: String,
    pub path_injected: bool,
    pub last_verified_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmokeTestResult {
    pub id: String,
    pub runtime_id: String,
    pub test_command: String,
    pub exit_code: i32,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub passed: bool,
    pub tested_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorReport {
    pub id: String,
    pub run_at: String,
    pub issues_found: i32,
    pub issues_fixed: i32,
    pub full_report: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeManifest {
    pub language: String,
    pub versions: Vec<RuntimeVersion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeVersion {
    pub version: String,
    pub download_url: String,
    pub checksum_url: String,
    pub sha256: String,
    pub binary_name: String,
    pub platform: String,
    pub arch: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallProgress {
    pub language: String,
    pub version: String,
    pub stage: String,
    pub progress_percent: u8,
    pub message: String,
}
