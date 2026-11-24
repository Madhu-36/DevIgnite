use crate::error::{AppError, Result};
use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub tool_name: String,
    pub version: String,
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
    pub percentage: f32,
}

impl DownloadProgress {
    pub fn new(tool_name: &str, version: &str, downloaded_bytes: u64, total_bytes: Option<u64>) -> Self {
        let percentage = match total_bytes {
            Some(total) if total > 0 => ((downloaded_bytes as f64 / total as f64) * 100.0) as f32,
            _ => 0.0,
        };
        Self {
            tool_name: tool_name.to_string(),
            version: version.to_string(),
            downloaded_bytes,
            total_bytes,
            percentage,
        }
    }
}

pub fn get_known_checksum(tool: &str, version: &str, os: &str, arch: &str) -> Option<&'static str> {
    match (tool, version, os, arch) {
        ("node", "20.11.0", "windows", "x64") => Some("e76bcfdbe5062a74c43ba7db3bc61b2e65c92cda19c0b115456c6f78f9f68882"),
        ("node", "18.19.0", "windows", "x64") => Some("8272ea87a7164ff50d7e8286a073f15c48bbfb17781b490f237ef1b423f03b5f"),
        ("go", "1.22.0", "windows", "amd64") => Some("f61dc7824147076a582fae2e28a6fbe857ab349f4c3983286f91ecba3a2e3797"),
        _ => None,
    }
}

pub async fn download_and_verify(
    url: &str,
    output_path: &PathBuf,
    expected_sha256: Option<&str>,
) -> Result<()> {
    let client = Client::new();
    let res = client.get(url).send().await?.error_for_status()?;

    let mut file = File::create(output_path).await?;
    let mut hasher = Sha256::new();
    let mut stream = res.bytes_stream();

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result?;
        file.write_all(&chunk).await?;
        hasher.update(&chunk);
    }

    file.flush().await?;

    let hash = format!("{:x}", hasher.finalize());

    if let Some(expected) = expected_sha256 {
        if expected != hash {
            let _ = tokio::fs::remove_file(output_path).await; // clean up on mismatch
            return Err(AppError::ChecksumMismatch {
                expected: expected.to_string(),
                actual: hash,
            });
        }
    }

    Ok(())
}

pub fn unpack_archive(archive_path: &PathBuf, dest_dir: &PathBuf) -> Result<()> {
    if !dest_dir.exists() {
        std::fs::create_dir_all(dest_dir)?;
    }

    let ext = archive_path.extension().unwrap_or_default().to_string_lossy();
    let file = std::fs::File::open(archive_path)?;

    if ext.eq_ignore_ascii_case("zip") {
        let mut archive = zip::ZipArchive::new(file).map_err(|e| AppError::Installation(e.to_string()))?;
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| AppError::Installation(e.to_string()))?;
            let outpath = match file.enclosed_name() {
                Some(path) => dest_dir.join(path),
                None => continue, // Prevents Zip-Slip traversal by dropping invalid paths
            };

            if (*file.name()).ends_with('/') {
                std::fs::create_dir_all(&outpath)?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        std::fs::create_dir_all(p)?;
                    }
                }
                let mut outfile = std::fs::File::create(&outpath)?;
                std::io::copy(&mut file, &mut outfile)?;
            }
            
            // On Unix, extract permissions if available
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Some(mode) = file.unix_mode() {
                    std::fs::set_permissions(&outpath, std::fs::Permissions::from_mode(mode))?;
                }
            }
        }
    } else if ext.eq_ignore_ascii_case("gz") || ext.eq_ignore_ascii_case("tgz") {
        use flate2::read::GzDecoder;
        use tar::Archive;
        let tar = GzDecoder::new(file);
        let mut archive = Archive::new(tar);
        
        for entry in archive.entries().map_err(|e| AppError::Installation(e.to_string()))? {
            let mut entry = entry.map_err(|e| AppError::Installation(e.to_string()))?;
            let path = entry.path().map_err(|e| AppError::Installation(e.to_string()))?;
            
            // Manual Zip-Slip prevention for tar
            if path.components().any(|c| matches!(c, std::path::Component::ParentDir | std::path::Component::RootDir)) {
                continue;
            }
            
            entry.unpack_in(dest_dir).map_err(|e| AppError::Installation(e.to_string()))?;
        }
    } else {
        return Err(AppError::Installation(format!("Unsupported archive format: {}", ext)));
    }

    Ok(())
}
