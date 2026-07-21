use anyhow::{bail, Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub struct SymlinkEngine {
    sandbox_root: PathBuf,
    bin_dir: PathBuf,
    runtimes_dir: PathBuf,
}

#[derive(Debug, Clone)]
pub struct SymlinkHealth {
    pub symlink_path: PathBuf,
    pub target_path: PathBuf,
    pub is_valid: bool,
    pub binary_name: String,
}

#[derive(Debug, Clone)]
pub struct SwitchPlan {
    pub language: String,
    pub from_version: Option<String>,
    pub to_version: String,
    pub binaries: Vec<String>,
    pub symlinks_to_create: Vec<(String, PathBuf)>,
}

impl SymlinkEngine {
    pub fn new() -> Result<Self> {
        let sandbox_root = dirs::home_dir()
            .context("Failed to resolve home directory")?
            .join(".devignite");
        let bin_dir = sandbox_root.join("bin");
        let runtimes_dir = sandbox_root.join("runtimes");

        let engine = Self {
            sandbox_root,
            bin_dir,
            runtimes_dir,
        };
        engine.ensure_directories()?;
        Ok(engine)
    }

    pub fn with_root(root: PathBuf) -> Result<Self> {
        let bin_dir = root.join("bin");
        let runtimes_dir = root.join("runtimes");
        let engine = Self {
            sandbox_root: root,
            bin_dir,
            runtimes_dir,
        };
        engine.ensure_directories()?;
        Ok(engine)
    }

    fn ensure_directories(&self) -> Result<()> {
        fs::create_dir_all(&self.bin_dir)
            .context("Failed to create sandbox bin directory")?;
        fs::create_dir_all(&self.runtimes_dir)
            .context("Failed to create runtimes directory")?;
        Ok(())
    }

    pub fn sandbox_root(&self) -> &Path { &self.sandbox_root }
    pub fn bin_dir(&self) -> &Path { &self.bin_dir }
    pub fn runtimes_dir(&self) -> &Path { &self.runtimes_dir }

    pub fn runtime_path(&self, language: &str, version: &str) -> PathBuf {
        self.runtimes_dir.join(language).join(version)
    }

    pub fn ensure_runtime_dir(&self, language: &str, version: &str) -> Result<PathBuf> {
        let dir = self.runtime_path(language, version);
        fs::create_dir_all(&dir).context(format!(
            "Failed to create runtime directory: {}", dir.display()
        ))?;
        Ok(dir)
    }

    // ── Atomic Symlink Creation ─────────────────────────────────

    pub fn create_symlink(
        &self,
        language: &str,
        version: &str,
        binary_name: &str,
    ) -> Result<PathBuf> {
        let runtime_dir = self.runtime_path(language, version);
        if !runtime_dir.exists() {
            bail!("Runtime directory does not exist: {}", runtime_dir.display());
        }

        let source_binary = find_binary_in_dir(&runtime_dir, binary_name)
            .context(format!("Binary '{}' not found in {}", binary_name, runtime_dir.display()))?;

        let symlink_target = self.bin_dir.join(binary_name);

        if symlink_target.exists() || symlink_target.symlink_metadata().is_ok() {
            remove_symlink_or_file(&symlink_target)
                .context(format!("Failed to remove existing entry: {}", symlink_target.display()))?;
        }

        create_platform_symlink(&source_binary, &symlink_target)?;

        log::info!("Symlink: {} -> {}", symlink_target.display(), source_binary.display());
        Ok(symlink_target)
    }

    pub fn remove_symlink(&self, binary_name: &str) -> Result<()> {
        let path = self.bin_dir.join(binary_name);
        if path.symlink_metadata().is_ok() {
            remove_symlink_or_file(&path)?;
            log::info!("Removed symlink: {}", path.display());
        }
        Ok(())
    }

    // ── Atomic Switch with Rollback ─────────────────────────────

    pub fn plan_switch(&self, language: &str, new_version: &str) -> Result<SwitchPlan> {
        let new_dir = self.runtime_path(language, new_version);
        if !new_dir.exists() {
            bail!("Target runtime directory does not exist: {}", new_dir.display());
        }

        let binaries = discover_runtime_binaries(&new_dir)?;
        if binaries.is_empty() {
            bail!("No executables found in {}", new_dir.display());
        }

        let current = self.get_active_version(language);
        let symlinks: Vec<(String, PathBuf)> = binaries
            .iter()
            .map(|b| (b.clone(), self.bin_dir.join(b)))
            .collect();

        Ok(SwitchPlan {
            language: language.to_string(),
            from_version: current,
            to_version: new_version.to_string(),
            binaries,
            symlinks_to_create: symlinks,
        })
    }

    pub fn execute_switch(&self, plan: &SwitchPlan) -> Result<Vec<PathBuf>> {
        let mut backup: Vec<(PathBuf, Option<PathBuf>)> = Vec::new();
        let mut created = Vec::new();

        for (binary_name, symlink_path) in &plan.symlinks_to_create {
            let old_target = if symlink_path.symlink_metadata().is_ok() {
                let t = fs::read_link(symlink_path).ok();
                remove_symlink_or_file(symlink_path)?;
                t
            } else {
                None
            };

            backup.push((symlink_path.clone(), old_target));

            match self.create_symlink(&plan.language, &plan.to_version, binary_name) {
                Ok(p) => created.push(p),
                Err(e) => {
                    log::error!("Symlink creation failed during switch: {}. Rolling back.", e);
                    self.rollback(backup)?;
                    bail!("Version switch failed and was rolled back: {}", e);
                }
            }
        }

        log::info!(
            "Switched {} {} -> {} ({} symlinks)",
            plan.language,
            plan.from_version.as_deref().unwrap_or("none"),
            plan.to_version,
            created.len()
        );

        Ok(created)
    }

    fn rollback(&self, backup: Vec<(PathBuf, Option<PathBuf>)>) -> Result<()> {
        for (symlink_path, old_target) in backup {
            let _ = remove_symlink_or_file(&symlink_path);
            if let Some(target) = old_target {
                let _ = create_platform_symlink(&target, &symlink_path);
            }
        }
        log::warn!("Symlink rollback completed");
        Ok(())
    }

    pub fn switch_version(&self, language: &str, new_version: &str) -> Result<Vec<PathBuf>> {
        let plan = self.plan_switch(language, new_version)?;
        self.execute_switch(&plan)
    }

    // ── Discovery & Health ──────────────────────────────────────

    pub fn get_active_version(&self, language: &str) -> Option<String> {
        let mut version_map: HashMap<String, String> = HashMap::new();

        if !self.bin_dir.exists() {
            return None;
        }

        for entry in fs::read_dir(&self.bin_dir).ok()? {
            let entry = entry.ok()?;
            let path = entry.path();
            if let Ok(target) = fs::read_link(&path) {
                if let Some(components) = target.components().rev().take(3).collect::<Vec<_>>().get(2) {
                    if let Some(ver) = components.to_str() {
                        if let Some(name) = path.file_stem().and_then(|n| n.to_str()) {
                            version_map.insert(name.to_string(), ver.to_string());
                        }
                    }
                }
            }
        }

        let runtime_dir = self.runtimes_dir.join(language);
        if !runtime_dir.exists() {
            return None;
        }

        for entry in fs::read_dir(&runtime_dir).ok()? {
            let entry = entry.ok()?;
            if entry.metadata().ok()?.is_dir() {
                if let Some(ver) = entry.file_name().to_str() {
                    let ver_dir = entry.path();
                    let binaries = discover_runtime_binaries(&ver_dir).unwrap_or_default();
                    if binaries.iter().any(|b| version_map.get(b.as_str()) == Some(&ver.to_string())) {
                        return Some(ver.to_string());
                    }
                }
            }
        }

        None
    }

    pub fn verify_symlinks(&self) -> Result<Vec<SymlinkHealth>> {
        let mut results = Vec::new();
        if !self.bin_dir.exists() {
            return Ok(results);
        }

        for entry in fs::read_dir(&self.bin_dir)? {
            let entry = entry?;
            let path = entry.path();
            let metadata = match fs::symlink_metadata(&path) {
                Ok(m) => m,
                Err(_) => continue,
            };

            if metadata.file_type().is_symlink() {
                let target = fs::read_link(&path).unwrap_or_default();
                let valid = target.exists();
                let binary_name = path.file_stem()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();
                results.push(SymlinkHealth {
                    symlink_path: path,
                    target_path: target,
                    is_valid: valid,
                    binary_name,
                });
            }
        }

        Ok(results)
    }

    pub fn list_active_binaries(&self) -> Vec<(String, PathBuf, PathBuf)> {
        let mut result = Vec::new();
        if !self.bin_dir.exists() {
            return result;
        }

        for entry in fs::read_dir(&self.bin_dir).into_iter().flatten() {
            let path = entry.path();
            if let Ok(target) = fs::read_link(&path) {
                if let Some(name) = path.file_stem().and_then(|n| n.to_str()) {
                    result.push((name.to_string(), path, target));
                }
            }
        }
        result
    }

    // ── Cleanup ─────────────────────────────────────────────────

    pub fn cleanup_runtime(&self, language: &str, version: &str) -> Result<()> {
        let binaries = discover_runtime_binaries(&self.runtime_path(language, version))
            .unwrap_or_default();
        for b in &binaries {
            let _ = self.remove_symlink(b);
        }

        let runtime_dir = self.runtime_path(language, version);
        if runtime_dir.exists() {
            fs::remove_dir_all(&runtime_dir)
                .context(format!("Failed to remove: {}", runtime_dir.display()))?;
            log::info!("Removed runtime: {}", runtime_dir.display());
        }

        let lang_dir = self.runtimes_dir.join(language);
        if lang_dir.exists() && is_dir_empty(&lang_dir) {
            fs::remove_dir(&lang_dir)?;
        }
        Ok(())
    }

    pub fn disk_usage(&self) -> u64 {
        if !self.runtimes_dir.exists() {
            return 0;
        }
        walkdir::WalkDir::new(&self.runtimes_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter_map(|e| e.metadata().ok())
            .map(|m| m.len())
            .sum()
    }
}

// ── Platform Helpers ────────────────────────────────────────────

fn create_platform_symlink(source: &Path, target: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(source, target)
            .context(format!("symlink {} -> {}", target.display(), source.display()))?;
    }
    #[cfg(windows)]
    {
        create_windows_symlink(source, target)?;
    }
    Ok(())
}

#[cfg(windows)]
fn create_windows_symlink(source: &Path, target: &Path) -> Result<()> {
    use std::process::Command;
    let output = Command::new("cmd")
        .args(["/c", "mklink", target.to_str().unwrap_or_default(), source.to_str().unwrap_or_default()])
        .output()
        .context("Failed to execute mklink")?;
    if !output.status.success() {
        bail!("mklink failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    Ok(())
}

fn find_binary_in_dir(dir: &Path, binary_name: &str) -> Result<PathBuf> {
    let candidates = if cfg!(windows) {
        vec![format!("{}.exe", binary_name), binary_name.to_string(), format!("{}.cmd", binary_name), format!("{}.bat", binary_name)]
    } else {
        vec![binary_name.to_string()]
    };

    for candidate in &candidates {
        let path = dir.join(candidate);
        if path.exists() { return Ok(path); }
    }

    for entry in walkdir::WalkDir::new(dir).max_depth(3).into_iter().filter_map(|e| e.ok()) {
        if let Some(name) = entry.path().file_name().and_then(|n| n.to_str()) {
            let stripped = name.trim_end_matches(".exe");
            if stripped == binary_name {
                return Ok(entry.path().to_path_buf());
            }
        }
    }
    bail!("Binary '{}' not found in {}", binary_name, dir.display())
}

pub fn discover_runtime_binaries(dir: &Path) -> Result<Vec<String>> {
    let mut binaries = Vec::new();
    let bin_subdir = dir.join("bin");
    let search_dirs = if bin_subdir.exists() { vec![&bin_subdir, dir] } else { vec![dir] };

    for search_dir in &search_dirs {
        if !search_dir.exists() { continue; }
        for entry in fs::read_dir(search_dir)? {
            let entry = entry?;
            if !entry.metadata()?.is_file() { continue; }
            if !is_executable(&entry.path()) { continue; }
            if let Some(name) = entry.path().file_stem().and_then(|n| n.to_str()) {
                if !binaries.contains(&name.to_string()) {
                    binaries.push(name.to_string());
                }
            }
        }
    }
    Ok(binaries)
}

fn is_executable(path: &Path) -> bool {
    if cfg!(windows) {
        path.extension().and_then(|e| e.to_str())
            .map(|e| matches!(e, "exe" | "cmd" | "bat" | "ps1"))
            .unwrap_or(false)
    } else {
        use std::os::unix::fs::PermissionsExt;
        fs::metadata(path).map(|m| m.permissions().mode() & 0o111 != 0).unwrap_or(false)
    }
}

fn remove_symlink_or_file(path: &Path) -> Result<()> {
    if path.symlink_metadata().map(|m| m.file_type().is_symlink()).unwrap_or(false) {
        fs::remove_file(path)?;
    } else if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

fn is_dir_empty(dir: &Path) -> bool {
    fs::read_dir(dir).map(|mut e| e.next().is_none()).unwrap_or(true)
}
