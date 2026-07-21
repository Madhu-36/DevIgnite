use anyhow::{bail, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub struct SymlinkEngine {
    sandbox_root: PathBuf,
    bin_dir: PathBuf,
    runtimes_dir: PathBuf,
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

    fn ensure_directories(&self) -> Result<()> {
        fs::create_dir_all(&self.bin_dir)
            .context("Failed to create sandbox bin directory")?;
        fs::create_dir_all(&self.runtimes_dir)
            .context("Failed to create runtimes directory")?;
        Ok(())
    }

    pub fn sandbox_root(&self) -> &Path {
        &self.sandbox_root
    }

    pub fn bin_dir(&self) -> &Path {
        &self.bin_dir
    }

    pub fn runtimes_dir(&self) -> &Path {
        &self.runtimes_dir
    }

    pub fn runtime_path(&self, language: &str, version: &str) -> PathBuf {
        self.runtimes_dir.join(language).join(version)
    }

    pub fn create_symlink(
        &self,
        language: &str,
        version: &str,
        binary_name: &str,
    ) -> Result<PathBuf> {
        let runtime_dir = self.runtime_path(language, version);
        if !runtime_dir.exists() {
            bail!(
                "Runtime directory does not exist: {}",
                runtime_dir.display()
            );
        }

        let source_binary = find_binary_in_dir(&runtime_dir, binary_name)
            .context(format!(
                "Binary '{}' not found in {}",
                binary_name,
                runtime_dir.display()
            ))?;

        let symlink_target = self.bin_dir.join(binary_name);

        if symlink_target.exists() || symlink_target.symlink_metadata().is_ok() {
            remove_symlink_or_file(&symlink_target).context(format!(
                "Failed to remove existing symlink: {}",
                symlink_target.display()
            ))?;
        }

        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&source_binary, &symlink_target).context(format!(
                "Failed to create symlink: {} -> {}",
                symlink_target.display(),
                source_binary.display()
            ))?;
        }

        #[cfg(windows)]
        {
            create_windows_symlink(&source_binary, &symlink_target)?;
        }

        log::info!(
            "Created symlink: {} -> {}",
            symlink_target.display(),
            source_binary.display()
        );

        Ok(symlink_target)
    }

    pub fn remove_symlink(&self, binary_name: &str) -> Result<()> {
        let symlink_path = self.bin_dir.join(binary_name);
        if symlink_path.exists() || symlink_path.symlink_metadata().is_ok() {
            remove_symlink_or_file(&symlink_path)?;
            log::info!("Removed symlink: {}", symlink_path.display());
        }
        Ok(())
    }

    pub fn remove_all_symlinks_for_runtime(
        &self,
        language: &str,
        version: &str,
    ) -> Result<()> {
        let runtime_dir = self.runtime_path(language, version);
        if !runtime_dir.exists() {
            return Ok(());
        }

        let binaries = discover_runtime_binaries(&runtime_dir)?;
        for binary_name in binaries {
            let _ = self.remove_symlink(&binary_name);
        }
        Ok(())
    }

    pub fn switch_version(
        &self,
        language: &str,
        new_version: &str,
    ) -> Result<Vec<PathBuf>> {
        let new_runtime_dir = self.runtime_path(language, new_version);
        if !new_runtime_dir.exists() {
            bail!(
                "Target runtime version directory does not exist: {}",
                new_runtime_dir.display()
            );
        }

        let binaries = discover_runtime_binaries(&new_runtime_dir)?;
        let mut created_symlinks = Vec::new();

        for binary_name in &binaries {
            let symlink = self.create_symlink(language, new_version, binary_name)?;
            created_symlinks.push(symlink);
        }

        log::info!(
            "Switched {} to version {} ({} symlinks updated)",
            language,
            new_version,
            created_symlinks.len()
        );

        Ok(created_symlinks)
    }

    pub fn cleanup_runtime(&self, language: &str, version: &str) -> Result<()> {
        self.remove_all_symlinks_for_runtime(language, version)?;

        let runtime_dir = self.runtime_path(language, version);
        if runtime_dir.exists() {
            fs::remove_dir_all(&runtime_dir).context(format!(
                "Failed to remove runtime directory: {}",
                runtime_dir.display()
            ))?;
            log::info!("Cleaned up runtime directory: {}", runtime_dir.display());
        }

        let lang_dir = self.runtimes_dir.join(language);
        if lang_dir.exists() && is_dir_empty(&lang_dir) {
            fs::remove_dir(&lang_dir)?;
        }

        Ok(())
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
                results.push(SymlinkHealth {
                    symlink_path: path,
                    target_path: target,
                    is_valid: valid,
                });
            } else if metadata.file_type().is_file() {
                results.push(SymlinkHealth {
                    symlink_path: path,
                    target_path: PathBuf::new(),
                    is_valid: true,
                });
            }
        }

        Ok(results)
    }

    pub fn restore_all_symlinks(
        &self,
        active_versions: &[(String, String, String)],
    ) -> Result<()> {
        for (language, version, binary_name) in active_versions {
            if let Err(e) = self.create_symlink(language, version, binary_name) {
                log::warn!(
                    "Failed to restore symlink for {} {}: {}",
                    language,
                    version,
                    e
                );
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct SymlinkHealth {
    pub symlink_path: PathBuf,
    pub target_path: PathBuf,
    pub is_valid: bool,
}

fn find_binary_in_dir(dir: &Path, binary_name: &str) -> Result<PathBuf> {
    let candidates = if cfg!(windows) {
        vec![
            format!("{}.exe", binary_name),
            binary_name.to_string(),
            format!("{}.cmd", binary_name),
            format!("{}.bat", binary_name),
        ]
    } else {
        vec![binary_name.to_string()]
    };

    for candidate in &candidates {
        let path = dir.join(candidate);
        if path.exists() {
            return Ok(path);
        }
    }

    for entry in walkdir::WalkDir::new(dir)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            let name_stripped = name.trim_end_matches(".exe");
            if name_stripped == binary_name {
                return Ok(path.to_path_buf());
            }
        }
    }

    bail!("Binary '{}' not found in {}", binary_name, dir.display())
}

fn discover_runtime_binaries(dir: &Path) -> Result<Vec<String>> {
    let mut binaries = Vec::new();
    let bin_subdir = dir.join("bin");

    let search_dirs = if bin_subdir.exists() {
        vec![bin_subdir, dir.to_path_buf()]
    } else {
        vec![dir.to_path_buf()]
    };

    for search_dir in &search_dirs {
        if !search_dir.exists() {
            continue;
        }
        for entry in fs::read_dir(search_dir)? {
            let entry = entry?;
            let path = entry.path();
            let metadata = entry.metadata()?;
            if !metadata.is_file() {
                continue;
            }
            if !is_executable(&path) {
                continue;
            }
            if let Some(name) = path.file_stem().and_then(|n| n.to_str()) {
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
        path.extension()
            .and_then(|e| e.to_str())
            .map(|e| matches!(e, "exe" | "cmd" | "bat" | "ps1"))
            .unwrap_or(false)
    } else {
        use std::os::unix::fs::PermissionsExt;
        fs::metadata(path)
            .map(|m| m.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    }
}

fn remove_symlink_or_file(path: &Path) -> Result<()> {
    if path.symlink_metadata()?.file_type().is_symlink() {
        fs::remove_file(path)?;
    } else if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

fn is_dir_empty(dir: &Path) -> bool {
    fs::read_dir(dir)
        .map(|mut entries| entries.next().is_none())
        .unwrap_or(true)
}

#[cfg(windows)]
fn create_windows_symlink(source: &Path, target: &Path) -> Result<()> {
    use std::process::Command;

    let output = Command::new("cmd")
        .args([
            "/c",
            "mklink",
            target.to_str().unwrap_or_default(),
            source.to_str().unwrap_or_default(),
        ])
        .output()
        .context("Failed to execute mklink command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("mklink failed: {}", stderr);
    }

    Ok(())
}
