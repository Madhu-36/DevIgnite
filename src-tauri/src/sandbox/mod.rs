use crate::error::{AppError, Result};
use std::path::{Path, PathBuf};

#[cfg(windows)]
use winapi::um::winuser::{SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_SETTINGCHANGE};
#[cfg(windows)]
use std::ptr;
#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;
#[cfg(windows)]
use std::ffi::OsStr;

pub fn create_symlink(target: &Path, link: &Path) -> Result<()> {
    if link.exists() || link.is_symlink() {
        std::fs::remove_file(link).or_else(|_| std::fs::remove_dir_all(link))?;
    }

    if let Some(parent) = link.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }

    #[cfg(windows)]
    {
        // On Windows, use junction for directories, or hardlink/softlink for files
        // Here we can use the junction crate for directories
        if target.is_dir() {
            junction::create(target, link).map_err(|e| AppError::Symlink(e.to_string()))?;
        } else {
            std::os::windows::fs::symlink_file(target, link).map_err(|e| AppError::Symlink(e.to_string()))?;
        }
    }

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(target, link)?;
    }

    Ok(())
}

pub fn broadcast_path_change(bin_dir: &Path) -> Result<()> {
    #[cfg(windows)]
    {
        use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_WRITE};
        use winreg::RegKey;

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let env = hkcu.open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE).map_err(|e| AppError::Symlink(e.to_string()))?;
        
        let current_path: String = env.get_value("Path").unwrap_or_default();
        let bin_dir_str = bin_dir.to_string_lossy().to_string();
        
        if !current_path.contains(&bin_dir_str) {
            let new_path = if current_path.is_empty() || current_path.ends_with(';') {
                format!("{}{};", current_path, bin_dir_str)
            } else {
                format!("{};{};", current_path, bin_dir_str)
            };
            
            env.set_value("Path", &new_path).map_err(|e| AppError::Symlink(e.to_string()))?;
        }

        let env_str: Vec<u16> = OsStr::new("Environment")
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
            
        unsafe {
            SendMessageTimeoutW(
                HWND_BROADCAST,
                WM_SETTINGCHANGE,
                0,
                env_str.as_ptr() as _,
                SMTO_ABORTIFHUNG,
                5000,
                ptr::null_mut(),
            );
        }
    }

    #[cfg(unix)]
    {
        // Path changes typically require restarting shell or sourcing, 
        // but we can log or instruct users.
        println!("Unix path broadcast: Please ensure {} is in your PATH.", bin_dir.display());
    }

    Ok(())
}

pub fn detect_shell_profiles(home_dir: &Path) -> Vec<PathBuf> {
    let candidates = [
        ".zshrc",
        ".bashrc",
        ".bash_profile",
        ".profile",
        ".config/fish/config.fish",
    ];

    candidates
        .iter()
        .map(|name| home_dir.join(name))
        .filter(|p| p.exists())
        .collect()
}

pub fn generate_shell_export_snippet(bin_dir: &Path) -> String {
    format!("\n# DevIgnite Sandboxed Bin Directory\nexport PATH=\"{}:$PATH\"\n", bin_dir.display())
}
