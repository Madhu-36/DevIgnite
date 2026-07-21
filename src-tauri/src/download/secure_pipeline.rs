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
}

impl SecureDownloader {
    pub fn new() -> Result<Self> {
        let home = dirs::home_dir().context("Failed to resolve home directory")?;
        let cache_dir = home.join(".devignite").join("cache");
        let temp_dir = home.join(".devignite").join("temp");

        fs::create_dir_all(&cache_dir)?;
        fs::create_dir_all(&temp_dir)?;

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .user_agent("DevIgnite/0.1.0")
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self {
            client,
            cache_dir,
            temp_dir,
        })
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
            "{}-{}-{}.tmp",
            language,
            version,
            uuid::Uuid::new_v4()
        ));

        if let Some(ref cb) = progress_callback {
            cb(5, "Starting download...");
        }

        let response = self
            .client
            .get(url)
            .send()
            .await
            .context("Failed to initiate download")?;

        let status = response.status();
        if !status.is_success() {
            bail!("Download failed with HTTP status: {}", status);
        }

        let total_size = response.content_length().unwrap_or(0);
        let mut downloaded: u64 = 0;

        let mut file = async_fs::File::create(&temp_file)
            .await
            .context("Failed to create temp file")?;

        let mut stream = response.bytes_stream();
        let mut hasher = Sha256::new();

        use futures_util::StreamExt;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.context("Failed to read download chunk")?;
            file.write_all(&chunk)
                .await
                .context("Failed to write chunk to file")?;
            hasher.update(&chunk);

            downloaded += chunk.len() as u64;
            if total_size > 0 {
                let pct = ((downloaded as f64 / total_size as f64) * 80.0) as u8 + 5;
                if let Some(ref cb) = progress_callback {
                    cb(pct.min(85), &format!("Downloading... {}%", (downloaded as f64 / total_size as f64 * 100.0) as u8));
                }
            }
        }

        file.flush().await?;
        drop(file);

        if let Some(ref cb) = progress_callback {
            cb(85, "Verifying checksum...");
        }

        let computed_hash = format!("{:x}", hasher.finalize());

        if !hex_str_eq(&computed_hash, expected_sha256) {
            let _ = async_fs::remove_file(&temp_file).await;
            bail!(
                "SHA-256 checksum mismatch!\n  Expected: {}\n  Computed: {}",
                expected_sha256,
                computed_hash
            );
        }

        log::info!("SHA-256 checksum verified: {}", computed_hash);

        if let Some(ref cb) = progress_callback {
            cb(90, "Extracting runtime...");
        }

        let runtime_dir = self.extract_archive(
            &temp_file,
            language,
            version,
            binary_name,
        )
        .await?;

        let _ = async_fs::remove_file(&temp_file).await;

        if let Some(ref cb) = progress_callback {
            cb(100, "Installation complete!");
        }

        log::info!(
            "Runtime installed: {} {} -> {}",
            language,
            version,
            runtime_dir.display()
        );

        Ok(runtime_dir)
    }

    async fn extract_archive(
        &self,
        archive_path: &Path,
        language: &str,
        version: &str,
        binary_name: &str,
    ) -> Result<PathBuf> {
        let home = dirs::home_dir().context("Failed to resolve home directory")?;
        let runtime_dir = home
            .join(".devignite")
            .join("runtimes")
            .join(language)
            .join(version);

        fs::create_dir_all(&runtime_dir)?;

        let file_bytes = fs::read(archive_path)
            .context("Failed to read archive file")?;

        let archive_name = archive_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        if archive_name.ends_with(".tar.gz") || archive_name.ends_with(".tgz") {
            extract_tar_gz(&file_bytes, &runtime_dir)
                .context("Failed to extract .tar.gz archive")?;
        } else if archive_name.ends_with(".zip") {
            extract_zip(&file_bytes, &runtime_dir)
                .context("Failed to extract .zip archive")?;
        } else {
            extract_by_magic(&file_bytes, &runtime_dir)
                .context("Failed to extract archive (unknown format)")?;
        }

        ensure_binary_executable(&runtime_dir, binary_name)?;

        Ok(runtime_dir)
    }

    pub fn verify_cached_file(
        &self,
        file_path: &Path,
        expected_sha256: &str,
    ) -> Result<bool> {
        let mut file = fs::File::open(file_path)
            .context("Failed to open cached file for verification")?;
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 8192];

        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        let computed = format!("{:x}", hasher.finalize());
        Ok(hex_str_eq(&computed, expected_sha256))
    }

    pub async fn fetch_checksum(&self, checksum_url: &str) -> Result<String> {
        let response = self
            .client
            .get(checksum_url)
            .send()
            .await
            .context("Failed to fetch checksum file")?;

        let text = response
            .text()
            .await
            .context("Failed to read checksum response")?;

        parse_sha256_from_text(&text)
    }

    pub fn cleanup_temp(&self) -> Result<()> {
        if self.temp_dir.exists() {
            fs::remove_dir_all(&self.temp_dir)?;
            fs::create_dir_all(&self.temp_dir)?;
        }
        Ok(())
    }
}

fn hex_str_eq(a: &str, b: &str) -> bool {
    let a_lower: String = a.chars().filter(|c| *c != ' ' && *c != '\n').collect();
    let b_lower: String = b.chars().filter(|c| *c != ' ' && *c != '\n').collect();
    a_lower.to_lowercase() == b_lower.to_lowercase()
}

fn parse_sha256_from_text(text: &str) -> Result<String> {
    let lines: Vec<&str> = text.lines().collect();
    for line in &lines {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 1 {
            let hash = parts[0];
            if hash.len() == 64 && hash.chars().all(|c| c.is_ascii_hexdigit()) {
                return Ok(hash.to_string());
            }
        }
    }
    bail!("No valid SHA-256 hash found in checksum text")
}

fn extract_tar_gz(data: &[u8], dest: &Path) -> Result<()> {
    let decoder = GzDecoder::new(data);
    let mut archive = tar::Archive::new(decoder);
    archive.unpack(dest).context("Failed to unpack tar.gz")?;
    Ok(())
}

fn extract_zip(data: &[u8], dest: &Path) -> Result<()> {
    let cursor = std::io::Cursor::new(data);
    let mut archive = zip::ZipArchive::new(cursor)
        .context("Failed to open zip archive")?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)
            .context("Failed to read zip entry")?;

        let outpath = dest.join(entry.mangled_name());

        if entry.is_dir() {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut outfile = fs::File::create(&outpath)?;
            std::io::copy(&mut entry, &mut outfile)?;
        }
    }

    Ok(())
}

fn extract_by_magic(data: &[u8], dest: &Path) -> Result<()> {
    if data.len() >= 2 && data[0] == 0x1f && data[1] == 0x8b {
        return extract_tar_gz(data, dest);
    }
    if data.len() >= 4 && data[0] == b'P' && data[1] == b'K' && data[2] == 0x03 && data[3] == 0x04 {
        return extract_zip(data, dest);
    }
    bail!("Unrecognized archive format")
}

fn ensure_binary_executable(runtime_dir: &Path, binary_name: &str) -> Result<()> {
    if cfg!(unix) {
        use std::os::unix::fs::PermissionsExt;

        for entry in walkdir::WalkDir::new(runtime_dir)
            .max_depth(3)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                let stem = name.trim_end_matches(".exe");
                if stem == binary_name && path.is_file() {
                    let mut perms = fs::metadata(path)?.permissions();
                    perms.set_mode(perms.mode() | 0o755);
                    fs::set_permissions(path, perms)?;
                    log::info!("Set executable permissions: {}", path.display());
                    return Ok(());
                }
            }
        }
    }

    log::warn!(
        "Binary '{}' not found in {} for permission setting",
        binary_name,
        runtime_dir.display()
    );
    Ok(())
}
