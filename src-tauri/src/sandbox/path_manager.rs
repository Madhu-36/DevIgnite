use anyhow::{bail, Context, Result};
use std::fs;
use std::path::PathBuf;

pub struct PathManager {
    sandbox_bin_path: PathBuf,
}

impl PathManager {
    pub fn new() -> Result<Self> {
        let home = dirs::home_dir().context("Failed to resolve home directory")?;
        let sandbox_bin_path = home.join(".devignite").join("bin");
        Ok(Self { sandbox_bin_path })
    }

    pub fn sandbox_bin_path(&self) -> &PathBuf {
        &self.sandbox_bin_path
    }

    pub fn is_in_user_path(&self) -> Result<bool> {
        let current_path = std::env::var("PATH").unwrap_or_default();
        let sandbox_str = self.sandbox_bin_path.to_str().unwrap_or_default();
        Ok(current_path
            .split(';')
            .any(|p| p.trim().eq_ignore_ascii_case(sandbox_str)))
    }

    pub fn inject_sandbox_into_path(&self) -> Result<bool> {
        if cfg!(target_os = "windows") {
            self.inject_windows_path()
        } else {
            self.inject_unix_path()
        }
    }

    #[cfg(target_os = "windows")]
    fn inject_windows_path(&self) -> Result<bool> {
        use windows::Win32::Foundation::HWND;
        use windows::Win32::System::Registry::*;
        use windows::Win32::UI::WindowsAndMessaging::SendMessageTimeoutW;
        use windows::Win32::UI::WindowsAndMessaging::{
            HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_SETTINGCHANGE,
        };

        let sandbox_str = self.sandbox_bin_path.to_str()
            .context("Sandbox path contains invalid Unicode")?;

        unsafe {
            let result = RegOpenKeyExW(
                HKEY_CURRENT_USER,
                w!("Environment"),
                0,
                KEY_READ | KEY_WRITE,
            );

            let hkey = match result {
                Ok((hkey, _)) => hkey,
                Err(e) => bail!("Failed to open registry key: {}", e),
            };

            let mut current_value_size: u32 = 0;
            let _ = RegQueryValueExW(
                hkey,
                w!("Path"),
                None,
                None,
                None,
                Some(&mut current_value_size),
            );

            let mut current_value_bytes = vec![0u16; (current_value_size / 2) as usize + 1];
            let mut value_type = REG_NONE;

            let query_result = RegQueryValueExW(
                hkey,
                w!("Path"),
                None,
                Some(&mut value_type),
                Some(current_value_bytes.as_mut_ptr() as *mut u8),
                Some(&mut current_value_size),
            );

            if let Err(e) = query_result {
                RegCloseKey(hkey)?;
                bail!("Failed to read Path registry value: {}", e);
            }

            let current_path = String::from_utf16_lossy(&current_value_bytes)
                .trim_end_matches('\0')
                .to_string();

            let path_entries: Vec<&str> =
                current_path.split(';').map(|s| s.trim()).collect();

            for entry in &path_entries {
                if entry.eq_ignore_ascii_case(sandbox_str) {
                    RegCloseKey(hkey)?;
                    log::info!("Sandbox bin path already present in User PATH");
                    return Ok(false);
                }
            }

            let new_path = if current_path.is_empty() {
                sandbox_str.to_string()
            } else {
                format!("{};{}", current_path, sandbox_str)
            };

            let mut new_path_utf16: Vec<u16> = new_path.encode_utf16().collect();
            new_path_utf16.push(0);

            let set_result = RegSetValueExW(
                hkey,
                w!("Path"),
                0,
                REG_EXPAND_SZ,
                Some(new_path_utf16.as_ptr() as *const u8),
                (new_path_utf16.len() * 2) as u32,
            );

            if let Err(e) = set_result {
                RegCloseKey(hkey)?;
                bail!("Failed to write Path registry value: {}", e);
            }

            RegCloseKey(hkey)?;

            let broadcast_msg = format!("Environment\0Path\0");
            let mut msg_utf16: Vec<u16> = broadcast_msg.encode_utf16().collect();
            msg_utf16.push(0);

            SendMessageTimeoutW(
                HWND_BROADCAST,
                WM_SETTINGCHANGE,
                HWND(0),
                windows::core::PCWSTR::from_raw(msg_utf16.as_ptr()),
                SMTO_ABORTIFHUNG,
                5000,
                None,
            );

            log::info!(
                "Injected sandbox path into User PATH and broadcast WM_SETTINGCHANGE: {}",
                sandbox_str
            );

            Ok(true)
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn inject_unix_path(&self) -> Result<bool> {
        let sandbox_str = self.sandbox_bin_path.to_str()
            .context("Sandbox path contains invalid Unicode")?;

        let export_line = format!("export PATH=\"{}:$PATH\"", sandbox_str);

        let shell_profiles = self.get_shell_profiles()?;
        let mut injected = false;

        for profile in &shell_profiles {
            if !profile.exists() {
                fs::write(profile, format!("{}\n", export_line))?;
                injected = true;
                log::info!("Created shell profile and injected path: {}", profile.display());
                continue;
            }

            let contents = fs::read_to_string(profile)?;
            if contents.contains(sandbox_str) {
                log::info!(
                    "Sandbox path already present in {}",
                    profile.display()
                );
                continue;
            }

            let mut modified = contents;
            if !modified.ends_with('\n') {
                modified.push('\n');
            }
            modified.push_str(&format!("\n# DevIgnite sandbox path\n{}\n", export_line));
            fs::write(profile, modified)?;
            injected = true;
            log::info!("Injected sandbox path into {}", profile.display());
        }

        Ok(injected)
    }

    #[cfg(not(target_os = "windows"))]
    fn get_shell_profiles(&self) -> Result<Vec<PathBuf>> {
        let home = dirs::home_dir().context("Failed to resolve home directory")?;
        let mut profiles = Vec::new();

        if let Ok(shell) = std::env::var("SHELL") {
            if shell.contains("zsh") {
                profiles.push(home.join(".zshrc"));
            } else if shell.contains("fish") {
                profiles.push(home.join(".config").join("fish").join("config.fish"));
            } else {
                profiles.push(home.join(".bashrc"));
            }
        } else {
            profiles.push(home.join(".bashrc"));
            profiles.push(home.join(".zshrc"));
        }

        Ok(profiles)
    }

    pub fn remove_sandbox_from_path(&self) -> Result<()> {
        if cfg!(target_os = "windows") {
            self.remove_windows_path()
        } else {
            self.remove_unix_path()
        }
    }

    #[cfg(target_os = "windows")]
    fn remove_windows_path(&self) -> Result<()> {
        use windows::Win32::System::Registry::*;
        use windows::Win32::UI::WindowsAndMessaging::{
            SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_SETTINGCHANGE,
        };

        let sandbox_str = self.sandbox_bin_path.to_str()
            .context("Sandbox path contains invalid Unicode")?;

        unsafe {
            let result = RegOpenKeyExW(
                HKEY_CURRENT_USER,
                w!("Environment"),
                0,
                KEY_READ | KEY_WRITE,
            );

            let hkey = match result {
                Ok((hkey, _)) => hkey,
                Err(e) => bail!("Failed to open registry key: {}", e),
            };

            let mut current_value_size: u32 = 0;
            let _ = RegQueryValueExW(
                hkey,
                w!("Path"),
                None,
                None,
                None,
                Some(&mut current_value_size),
            );

            let mut current_value_bytes = vec![0u16; (current_value_size / 2) as usize + 1];
            let mut value_type = REG_NONE;

            let query_result = RegQueryValueExW(
                hkey,
                w!("Path"),
                None,
                Some(&mut value_type),
                Some(current_value_bytes.as_mut_ptr() as *mut u8),
                Some(&mut current_value_size),
            );

            if let Err(e) = query_result {
                RegCloseKey(hkey)?;
                bail!("Failed to read Path registry value: {}", e);
            }

            let current_path = String::from_utf16_lossy(&current_value_bytes)
                .trim_end_matches('\0')
                .to_string();

            let new_path: String = current_path
                .split(';')
                .map(|s| s.trim())
                .filter(|s| !s.eq_ignore_ascii_case(sandbox_str))
                .collect::<Vec<_>>()
                .join(";");

            let mut new_path_utf16: Vec<u16> = new_path.encode_utf16().collect();
            new_path_utf16.push(0);

            RegSetValueExW(
                hkey,
                w!("Path"),
                0,
                REG_EXPAND_SZ,
                Some(new_path_utf16.as_ptr() as *const u8),
                (new_path_utf16.len() * 2) as u32,
            )?;

            RegCloseKey(hkey)?;

            let broadcast_msg = format!("Environment\0Path\0");
            let mut msg_utf16: Vec<u16> = broadcast_msg.encode_utf16().collect();
            msg_utf16.push(0);

            SendMessageTimeoutW(
                HWND_BROADCAST,
                WM_SETTINGCHANGE,
                windows::core::HWND(0),
                windows::core::PCWSTR::from_raw(msg_utf16.as_ptr()),
                SMTO_ABORTIFHUNG,
                5000,
                None,
            );

            log::info!("Removed sandbox path from User PATH and broadcast change");
            Ok(())
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn remove_unix_path(&self) -> Result<()> {
        let sandbox_str = self.sandbox_bin_path.to_str()
            .context("Sandbox path contains invalid Unicode")?;

        let profiles = self.get_shell_profiles()?;
        for profile in &profiles {
            if !profile.exists() {
                continue;
            }
            let contents = fs::read_to_string(profile)?;
            let filtered: String = contents
                .lines()
                .filter(|line| !line.contains(sandbox_str))
                .filter(|line| !line.trim().eq("# DevIgnite sandbox path"))
                .collect::<Vec<_>>()
                .join("\n");

            fs::write(profile, format!("{}\n", filtered.trim()))?;
        }

        log::info!("Removed sandbox path from shell profiles");
        Ok(())
    }

    pub fn verify_path_broadcast(&self) -> Result<PathBroadcastStatus> {
        let current_path = std::env::var("PATH").unwrap_or_default();
        let sandbox_str = self.sandbox_bin_path.to_str().unwrap_or_default();

        let in_env_path = current_path
            .split(';')
            .any(|p| p.trim().eq_ignore_ascii_case(sandbox_str));

        let bin_dir_exists = self.sandbox_bin_path.exists();

        let symlink_count = if bin_dir_exists {
            fs::read_dir(&self.sandbox_bin_path)
                .map(|entries| {
                    entries
                        .filter_map(|e| e.ok())
                        .filter(|e| {
                            e.metadata()
                                .map(|m| m.file_type().is_symlink() || m.file_type().is_file())
                                .unwrap_or(false)
                        })
                        .count()
                })
                .unwrap_or(0)
        } else {
            0
        };

        Ok(PathBroadcastStatus {
            in_user_path: in_env_path,
            bin_dir_exists,
            symlink_count,
            sandbox_path: self.sandbox_bin_path.display().to_string(),
        })
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PathBroadcastStatus {
    pub in_user_path: bool,
    pub bin_dir_exists: bool,
    pub symlink_count: usize,
    pub sandbox_path: String,
}
