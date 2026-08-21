use serde::{Serialize, Serializer};

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Database Error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("Network Error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("Download Error: {0}")]
    Download(String),
    #[error("Installation Error: {0}")]
    Installation(String),
    #[error("Checksum mismatch. Expected {expected}, got {actual}")]
    ChecksumMismatch { expected: String, actual: String },
    #[error("Symlink Error: {0}")]
    Symlink(String),
    #[error("Unknown Error: {0}")]
    Unknown(#[from] anyhow::Error),
}

// Implement Serialize so we can return AppError to the Tauri frontend
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
