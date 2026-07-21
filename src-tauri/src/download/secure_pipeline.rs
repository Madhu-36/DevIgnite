use anyhow::{bail, Context, Result};
use flate2::read::GzDecoder;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tokio::fs as async_fs;
use tokio::io::AsyncWriteExt;

pub struct SecureDownloader {
    client: reqwest::Client,
    cache_dir: PathBuf,
    temp_dir: PathBuf,
    max_retries: u32,
}

impl SecureDownloader {
    pub fn new() -> Result<Self> {
        let home = dirs::home_dir().context("Failed to resolve home directory")?;
        let cache_dir = home.join(".devignite").join("cache");
        let temp_dir = home.join(".devignite").join("temp");
        fs::create_dir_all(&cache_dir)?;
        fs::create_dir_all(&temp_dir)?;

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(600))
            .user_agent("DevIgnite/0.2.0")
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self { client, cache_dir, temp_dir, max_retries: 3 })
    }

    pub async fn download_and_install(
        &self,
        url: &str,
        expected_sha256: &str,
        language: &str,
        version: &str,
        binary_name: &str,
        progress_callback: Option<Box<dyn Fn(u8, &str) + Send + Sync>>,
    ) -> Result<PathBuf> {
        let temp_file = self.temp_dir.join(format!(
            "{}-{}-{}.tmp", language, version, uuid::Uuid::new_v4()
        ));

        let mut last_error = None;
        for attempt in 0..self.max_retries {
            if attempt > 0 {
                if let Some(ref cb) = progress_callback {
                    cb(0, &format!("Retry {}/{}...", attempt + 1, self.max_retries));
                }
                tokio::time::sleep(std::time::Duration::from_secs(2u64.pow(attempt))).await;
            }

            match self.do_download(url, &temp_file, expected_sha256, &progress_callback).await {
                Ok(()) => { last_error = None; break; }
                Err(e) => {
                    log::warn!("Download attempt {} failed: {}", attempt + 1, e);
                    last_error = Some(e);
                }
            }
        }

        if let Some(e) = last_error {
            let _ = async_fs::remove_file(&temp_file).await;
            bail!("Download failed after {} retries: {}", self.max_retries, e);
        }

        if let Some(ref cb) = progress_callback {
            cb(90, "Extracting runtime...");
        }

        let runtime_dir = self.extract_archive(&temp_file, language, version, binary_name).await?;
        let _ = async_fs::remove_file(&temp_file).await;

        if let Some(ref cb) = progress_callback {
            cb(100, "Installation complete!");
        }

        log::info!("Installed {} {} -> {}", language, version, runtime_dir.display());
        Ok(runtime_dir)
    }

    async fn do_download(
        &self,
        url: &str,
        dest: &Path,
        expected_sha256: &str,
        progress_callback: &Option<Box<dyn Fn(u8, &str) + Send + Sync>>,
    ) -> Result<()> {
        if let Some(ref cb) = progress_callback {
            cb(5, "Connecting...");
        }

        let response = self.client.get(url).send().await.context("Failed to initiate download")?;
        if !response.status().is_success() {
            bail!("HTTP {}", response.status());
        }

        let total_size = response.content_length().unwrap_or(0);
        let mut downloaded: u64 = 0;

        let mut file = async_fs::File::create(dest).await.context("Failed to create temp file")?;
        let mut stream = response.bytes_stream();
        let mut hasher = Sha256::new();

        use futures_util::StreamExt;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.context("Failed to read chunk")?;
            file.write_all(&chunk).await.context("Failed to write chunk")?;
            hasher.update(&chunk);
            downloaded += chunk.len() as u64;

            if total_size > 0 {
                let pct = 5 + ((downloaded as f64 / total_size as f64) * 75.0) as u8;
                if let Some(ref cb) = progress_callback {
                    cb(pct.min(80), &format!("Downloading... {}%", (downloaded as f64 / total_size as f64 * 100.0) as u8));
                }
            }
        }

        file.flush().await?;
        drop(file);

        if let Some(ref cb) = progress_callback {
            cb(85, "Verifying checksum...");
        }

        let computed = format!("{:x}", hasher.finalize());
        if !hex_eq(&computed, expected_sha256) {
            bail!("SHA-256 mismatch!\n  Expected: {}\n  Computed: {}", expected_sha256, computed);
        }

        log::info!("SHA-256 verified: {}", computed);
        Ok(())
    }

    async fn extract_archive(
        &self,
        archive_path: &Path,
        language: &str,
        version: &str,
        _binary_name: &str,
    ) -> Result<PathBuf> {
        let home = dirs::home_dir().context("Failed to resolve home directory")?;
        let runtime_dir = home.join(".devignite").join("runtimes").join(language).join(version);
        fs::create_dir_all(&runtime_dir)?;

        let file_bytes = fs::read(archive_path)?;
        let name = archive_path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
            extract_tar_gz(&file_bytes, &runtime_dir)?;
        } else if name.ends_with(".zip") {
            extract_zip(&file_bytes, &runtime_dir)?;
        } else {
            extract_by_magic(&file_bytes, &runtime_dir)?;
        }

        #[cfg(unix)]
        set_executable_permissions(&runtime_dir, _binary_name)?;

        Ok(runtime_dir)
    }

    pub fn verify_cached_file(&self, file_path: &Path, expected: &str) -> Result<bool> {
        let mut file = fs::File::open(file_path)?;
        let mut hasher = Sha256::new();
        let mut buf = [0u8; 8192];
        loop {
            let n = file.read(&mut buf)?;
            if n == 0 { break; }
            hasher.update(&buf[..n]);
        }
        Ok(hex_eq(&format!("{:x}", hasher.finalize()), expected))
    }

    pub async fn fetch_checksum(&self, checksum_url: &str) -> Result<String> {
        let text = self.client.get(checksum_url).send().await?.text().await?;
        parse_sha256_from_text(&text)
    }

    pub fn cleanup_temp(&self) -> Result<()> {
        if self.temp_dir.exists() {
            fs::remove_dir_all(&self.temp_dir)?;
            fs::create_dir_all(&self.temp_dir)?;
        }
        Ok(())
    }

    pub fn cache_dir(&self) -> &Path { &self.cache_dir }
    pub fn temp_dir(&self) -> &Path { &self.temp_dir }
}

fn hex_eq(a: &str, b: &str) -> bool {
    let a: String = a.chars().filter(|c| !c.is_whitespace()).collect();
    let b: String = b.chars().filter(|c| !c.is_whitespace()).collect();
    a.to_lowercase() == b.to_lowercase()
}

fn parse_sha256_from_text(text: &str) -> Result<String> {
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() { continue; }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 1 && parts[0].len() == 64 && parts[0].chars().all(|c| c.is_ascii_hexdigit()) {
            return Ok(parts[0].to_string());
        }
    }
    bail!("No valid SHA-256 hash found in checksum text")
}

fn extract_tar_gz(data: &[u8], dest: &Path) -> Result<()> {
    let decoder = GzDecoder::new(data);
    let mut archive = tar::Archive::new(decoder);
    archive.unpack(dest)?;
    Ok(())
}

fn extract_zip(data: &[u8], dest: &Path) -> Result<()> {
    let cursor = std::io::Cursor::new(data);
    let mut archive = zip::ZipArchive::new(cursor)?;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let outpath = dest.join(entry.mangled_name());
        if entry.is_dir() {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() { fs::create_dir_all(parent)?; }
            fs::copy(&mut entry, &outpath)?;
        }
    }
    Ok(())
}

fn extract_by_magic(data: &[u8], dest: &Path) -> Result<()> {
    if data.len() >= 2 && data[0] == 0x1f && data[1] == 0x8b { return extract_tar_gz(data, dest); }
    if data.len() >= 4 && data[0] == b'P' && data[1] == b'K' && data[2] == 0x03 && data[3] == 0x04 { return extract_zip(data, dest); }
    bail!("Unrecognized archive format")
}

#[cfg(unix)]
fn set_executable_permissions(dir: &Path, binary_name: &str) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    for entry in walkdir::WalkDir::new(dir).max_depth(3).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            let stem = name.trim_end_matches(".exe");
            if stem == binary_name && path.is_file() {
                let mut perms = fs::metadata(path)?.permissions();
                perms.set_mode(perms.mode() | 0o755);
                fs::set_permissions(path, perms)?;
            }
        }
    }
    Ok(())
}
