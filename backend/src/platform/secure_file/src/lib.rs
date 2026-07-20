//! Bounded Unix file reads that never follow the terminal path symlink.

use std::fs::{File, OpenOptions};
use std::io::Read;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::Path;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SecureReadPolicy {
    max_bytes: u64,
    require_owner_private: bool,
}

impl SecureReadPolicy {
    #[must_use]
    pub const fn regular(max_bytes: u64) -> Self {
        Self {
            max_bytes,
            require_owner_private: false,
        }
    }

    #[must_use]
    pub const fn owner_private(max_bytes: u64) -> Self {
        Self {
            max_bytes,
            require_owner_private: true,
        }
    }
}

#[derive(Debug)]
pub enum SecureFileError {
    Io(std::io::Error),
    InvalidFile,
    TooLarge,
}

impl std::fmt::Display for SecureFileError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Io(_) => "secure file operation failed",
            Self::InvalidFile => "secure file is not an allowed regular file",
            Self::TooLarge => "secure file exceeds the bounded read limit",
        })
    }
}

impl std::error::Error for SecureFileError {}

pub fn read(path: &Path, policy: SecureReadPolicy) -> Result<Vec<u8>, SecureFileError> {
    let mut file = open(path)?;
    let metadata = file.metadata().map_err(SecureFileError::Io)?;
    validate(&metadata, policy)?;
    let capacity = usize::try_from(metadata.len()).map_err(|_| SecureFileError::TooLarge)?;
    let mut bytes = Vec::with_capacity(capacity);
    file.read_to_end(&mut bytes).map_err(SecureFileError::Io)?;
    if bytes.len() as u64 != metadata.len() {
        return Err(SecureFileError::InvalidFile);
    }
    Ok(bytes)
}

fn open(path: &Path) -> Result<File, SecureFileError> {
    OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path)
        .map_err(SecureFileError::Io)
}

fn validate(metadata: &std::fs::Metadata, policy: SecureReadPolicy) -> Result<(), SecureFileError> {
    if !metadata.is_file() || metadata.len() == 0 {
        return Err(SecureFileError::InvalidFile);
    }
    if metadata.len() > policy.max_bytes {
        return Err(SecureFileError::TooLarge);
    }
    if policy.require_owner_private
        && (metadata.uid() != current_uid() || metadata.mode() & 0o077 != 0)
    {
        return Err(SecureFileError::InvalidFile);
    }
    Ok(())
}

fn current_uid() -> u32 {
    // SAFETY: `geteuid` has no preconditions and only reads process credentials.
    unsafe { libc::geteuid() }
}
