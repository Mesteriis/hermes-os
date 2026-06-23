use std::path::{Path, PathBuf};

use chrono::Utc;
use sha2::{Digest, Sha256};

use super::constants::{LOCAL_FS_STORAGE_KIND, SHA256_PREFIX};
use super::errors::CommunicationStorageError;
use super::validation::validate_storage_path;

#[derive(Clone, Debug)]
pub struct LocalCommunicationBlobStore {
    root: PathBuf,
}

impl LocalCommunicationBlobStore {
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub async fn put_blob(
        &self,
        bytes: &[u8],
    ) -> Result<LocalCommunicationBlob, CommunicationStorageError> {
        let size_bytes =
            i64::try_from(bytes.len()).map_err(|_| CommunicationStorageError::BlobTooLarge)?;
        let digest_hex = sha256_hex(bytes);
        let storage_path = relative_blob_path(&digest_hex);
        let absolute_path = self.root.join(&storage_path);

        if let Some(parent) = absolute_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        if !path_exists(&absolute_path).await? {
            let temp_path = absolute_path.with_extension(format!(
                "tmp-{}-{}",
                std::process::id(),
                Utc::now().timestamp_nanos_opt().unwrap_or_default()
            ));
            tokio::fs::write(&temp_path, bytes).await?;
            tokio::fs::rename(&temp_path, &absolute_path).await?;
        }

        let metadata = tokio::fs::metadata(&absolute_path).await?;
        let actual_size =
            i64::try_from(metadata.len()).map_err(|_| CommunicationStorageError::BlobTooLarge)?;
        if actual_size != size_bytes {
            return Err(CommunicationStorageError::BlobSizeMismatch {
                path: absolute_path,
                expected: size_bytes,
                actual: actual_size,
            });
        }

        Ok(LocalCommunicationBlob {
            storage_kind: LOCAL_FS_STORAGE_KIND.to_owned(),
            storage_path,
            sha256: format!("{SHA256_PREFIX}{digest_hex}"),
            size_bytes,
        })
    }

    pub async fn read_blob(
        &self,
        storage_path: &str,
    ) -> Result<Vec<u8>, CommunicationStorageError> {
        let storage_path = validate_storage_path(storage_path)?;
        Ok(tokio::fs::read(self.root.join(storage_path)).await?)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LocalCommunicationBlob {
    pub storage_kind: String,
    pub storage_path: String,
    pub sha256: String,
    pub size_bytes: i64,
}

async fn path_exists(path: &Path) -> Result<bool, std::io::Error> {
    match tokio::fs::metadata(path).await {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

fn relative_blob_path(digest_hex: &str) -> String {
    format!("sha256/{}/{}.blob", &digest_hex[..2], digest_hex)
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut encoded = String::with_capacity(digest.len() * 2);
    for byte in digest {
        encoded.push(hex_char(byte >> 4));
        encoded.push(hex_char(byte & 0x0f));
    }
    encoded
}

fn hex_char(value: u8) -> char {
    match value {
        0..=9 => char::from(b'0' + value),
        10..=15 => char::from(b'a' + (value - 10)),
        _ => unreachable!("hex nibble must fit in 0..=15"),
    }
}
