use anyhow::{bail, Context, Result};
use std::fs;
use std::path::PathBuf;

pub struct PathManager {
    sandbox_bin_path: PathBuf,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PathBroadcastStatus {
    pub in_user_path: bool,
    pub bin_dir_exists: bool,
    pub symlink_count: usize,
    pub sandbox_path: String,
    pub shell_profiles_checked: Vec<ShellProfileStatus>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ShellProfileStatus {
    pub path: String,
    pub exists: bool,
    pub contains_sandbox: bool,
}

impl PathManager {
    pub fn new() -> Result<Self> {
        let home = dirs::home_dir().context("Failed to resolve home directory")?;
        Ok(Self { sandbox_bin_path: home.join(".devignite").join("bin") })
    }

    pub fn sandbox_bin_path(&self) -> &PathBuf { &self.sandbox_bin_path }

    pub fn is_in_user_path(&self) -> Result<bool> {
        let current_path = std::env::var("PATH").unwrap_or_default();
        let sandbox_str = self.sandbox_bin_path.to_str().unwrap_or_default();
        Ok(current_path.split(';').any(|p| p.trim().eq_ignore_ascii_case(sandbox_str)))
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
        use windows::Win32::System::Registry::*;
        use windows::Win32::UI::WindowsAndMessaging::*;
        use windows::Win32::Foundation::HWND;

        let sandbox_str = self.sandbox_bin_path.to_str()
            .context("Sandbox path contains invalid Unicode")?;

        unsafe {
            let hkey = RegOpenKeyExW(
                HKEY_CURRENT_USER,
                windows::core::PCWSTR::from_raw(
                    "Environment\0".encode_utf16().collect::<Vec<u16>>().as_ptr()
                ),
                0,
                KEY_READ | KEY_WRITE,
            ).map(|(h, _)| h).map_err(|e| anyhow::anyhow!("RegOpenKeyExW failed: {}", e))?;

            let mut buf_size: u32 = 0;
            let _ = RegQueryValueExW(hkey, windows::core::PCWSTR::from_raw("Path\0".encode_utf16().collect::<Vec<u16>>().as_ptr()), None, None, None, Some(&mut buf_size));

            let mut buf = vec![0u16; (buf_size / 2) as usize + 1];
            let mut vtype = REG_NONE;
            let path_name_utf16: Vec<u16> = "Path\0".encode_utf16().collect();

            RegQueryValueExW(
                hkey,
                windows::core::PCWSTR::from_raw(path_name_utf16.as_ptr()),
                None,
                Some(&mut vtype),
                Some(buf.as_mut_ptr() as *mut u8),
                Some(&mut buf_size),
            ).map_err(|e| anyhow::anyhow!("RegQueryValueExW failed: {}", e))?;

            let current_path = String::from_utf16_lossy(&buf).trim_end_matches('\0').to_string();

            if current_path.split(';').any(|p| p.trim().eq_ignore_ascii_case(sandbox_str)) {
                RegCloseKey(hkey)?;
                return Ok(false);
            }

            let new_path = if current_path.is_empty() {
                sandbox_str.to_string()
            } else {
                format!("{};{}", current_path, sandbox_str)
            };

            let mut new_path_utf16: Vec<u16> = new_path.encode_utf16().collect();
            new_path_utf16.push(0);

            let path_value_name_utf16: Vec<u16> = "Path\0".encode_utf16().collect();

            RegSetValueExW(
                hkey,
                windows::core::PCWSTR::from_raw(path_value_name_utf16.as_ptr()),
                0,
                REG_EXPAND_SZ,
                Some(new_path_utf16.as_ptr() as *const u8),
                (new_path_utf16.len() * 2) as u32,
            ).map_err(|e| anyhow::anyhow!("RegSetValueExW failed: {}", e))?;

            RegCloseKey(hkey)?;

            let broadcast: Vec<u16> = "Environment\0Path\0".encode_utf16().collect();
            SendMessageTimeoutW(
                HWND_BROADCAST,
                WM_SETTINGCHANGE,
                HWND(0),
                windows::core::PCWSTR::from_raw(broadcast.as_ptr()),
                SMTO_ABORTIFHUNG,
                5000,
                None,
            );

            log::info!("PATH injected + WM_SETTINGCHANGE broadcast: {}", sandbox_str);
            Ok(true)
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn inject_windows_path(&self) -> Result<bool> {
        self.inject_unix_path()
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
                continue;
            }
            let contents = fs::read_to_string(profile)?;
            if contents.contains(sandbox_str) { continue; }
            let mut modified = contents;
            if !modified.ends_with('\n') { modified.push('\n'); }
            modified.push_str(&format!("\n# DevIgnite sandbox path\n{}\n", export_line));
            fs::write(profile, modified)?;
            injected = true;
        }
        Ok(injected)
    }

    #[cfg(target_os = "windows")]
    fn inject_unix_path(&self) -> Result<bool> {
        bail!("Unix path injection not supported on Windows")
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

    #[cfg(target_os = "windows")]
    fn get_shell_profiles(&self) -> Result<Vec<PathBuf>> {
        Ok(Vec::new())
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
        use windows::Win32::UI::WindowsAndMessaging::*;

        let sandbox_str = self.sandbox_bin_path.to_str()
            .context("Sandbox path contains invalid Unicode")?;

        unsafe {
            let hkey = RegOpenKeyExW(
                HKEY_CURRENT_USER,
                windows::core::PCWSTR::from_raw("Environment\0".encode_utf16().collect::<Vec<u16>>().as_ptr()),
                0, KEY_READ | KEY_WRITE,
            ).map(|(h, _)| h).map_err(|e| anyhow::anyhow!("RegOpenKeyExW failed: {}", e))?;

            let mut buf_size: u32 = 0;
            let _ = RegQueryValueExW(hkey, windows::core::PCWSTR::from_raw("Path\0".encode_utf16().collect::<Vec<u16>>().as_ptr()), None, None, None, Some(&mut buf_size));

            let mut buf = vec![0u16; (buf_size / 2) as usize + 1];
            let mut vtype = REG_NONE;
            let path_name_utf16: Vec<u16> = "Path\0".encode_utf16().collect();

            RegQueryValueExW(hkey, windows::core::PCWSTR::from_raw(path_name_utf16.as_ptr()), None, Some(&mut vtype), Some(buf.as_mut_ptr() as *mut u8), Some(&mut buf_size))
                .map_err(|e| anyhow::anyhow!("RegQueryValueExW failed: {}", e))?;

            let current = String::from_utf16_lossy(&buf).trim_end_matches('\0').to_string();
            let new_path: String = current.split(';').map(|s| s.trim())
                .filter(|s| !s.eq_ignore_ascii_case(sandbox_str)).collect::<Vec<_>>().join(";");

            let mut new_utf16: Vec<u16> = new_path.encode_utf16().collect();
            new_utf16.push(0);
            let path_value_name_utf16: Vec<u16> = "Path\0".encode_utf16().collect();

            RegSetValueExW(hkey, windows::core::PCWSTR::from_raw(path_value_name_utf16.as_ptr()), 0, REG_EXPAND_SZ, Some(new_utf16.as_ptr() as *const u8), (new_utf16.len() * 2) as u32)?;
            RegCloseKey(hkey)?;

            let broadcast: Vec<u16> = "Environment\0Path\0".encode_utf16().collect();
            SendMessageTimeoutW(HWND_BROADCAST, WM_SETTINGCHANGE, windows::core::HWND(0), windows::core::PCWSTR::from_raw(broadcast.as_ptr()), SMTO_ABORTIFHUNG, 5000, None);
            log::info!("PATH cleaned + WM_SETTINGCHANGE broadcast");
            Ok(())
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn remove_windows_path(&self) -> Result<()> { Ok(()) }

    #[cfg(not(target_os = "windows"))]
    fn remove_unix_path(&self) -> Result<()> {
        let sandbox_str = self.sandbox_bin_path.to_str().unwrap_or("");
        for profile in self.get_shell_profiles()? {
            if !profile.exists() { continue; }
            let contents = fs::read_to_string(&profile)?;
            let filtered: String = contents.lines()
                .filter(|l| !l.contains(sandbox_str) && !l.trim().eq("# DevIgnite sandbox path"))
                .collect::<Vec<_>>().join("\n");
            fs::write(&profile, format!("{}\n", filtered.trim()))?;
        }
        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn remove_unix_path(&self) -> Result<()> { Ok(()) }

    pub fn verify_path_broadcast(&self) -> Result<PathBroadcastStatus> {
        let current_path = std::env::var("PATH").unwrap_or_default();
        let sandbox_str = self.sandbox_bin_path.to_str().unwrap_or_default();
        let in_env = current_path.split(';').any(|p| p.trim().eq_ignore_ascii_case(sandbox_str));
        let bin_dir_exists = self.sandbox_bin_path.exists();
        let symlink_count = if bin_dir_exists {
            fs::read_dir(&self.sandbox_bin_path).map(|e| e.filter_map(|e| e.ok()).filter(|e| e.metadata().map(|m| m.file_type().is_symlink()).unwrap_or(false)).count()).unwrap_or(0)
        } else { 0 };

        let mut profiles = Vec::new();
        if cfg!(not(target_os = "windows")) {
            if let Ok(sp) = self.get_shell_profiles() {
                for p in sp {
                    let exists = p.exists();
                    let contains = if exists { fs::read_to_string(&p).unwrap_or_default().contains(sandbox_str) } else { false };
                    profiles.push(ShellProfileStatus { path: p.display().to_string(), exists, contains_sandbox: contains });
                }
            }
        }

        Ok(PathBroadcastStatus {
            in_user_path: in_env,
            bin_dir_exists,
            symlink_count,
            sandbox_path: self.sandbox_bin_path.display().to_string(),
            shell_profiles_checked: profiles,
        })
    }
}
