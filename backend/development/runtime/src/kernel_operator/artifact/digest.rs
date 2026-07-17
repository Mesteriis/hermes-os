use std::fs::{File, Metadata};
use std::io::Read;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

use sha2::{Digest, Sha256};

pub struct ArtifactDigest {
    canonical_path: String,
    sha256: [u8; 32],
    size: u64,
    device: u64,
    inode: u64,
}

impl ArtifactDigest {
    #[must_use]
    pub fn canonical_path(&self) -> &str {
        &self.canonical_path
    }

    #[must_use]
    pub fn sha256(&self) -> &[u8; 32] {
        &self.sha256
    }

    #[must_use]
    pub fn size(&self) -> u64 {
        self.size
    }

    #[must_use]
    pub fn device(&self) -> u64 {
        self.device
    }

    #[must_use]
    pub fn inode(&self) -> u64 {
        self.inode
    }
}

pub fn read_stable_regular_file(path: &Path) -> Result<ArtifactDigest, String> {
    if !path.is_absolute() {
        return Err("distribution artifact must be an absolute path".to_owned());
    }
    let input_metadata = std::fs::symlink_metadata(path).map_err(|error| error.to_string())?;
    if input_metadata.file_type().is_symlink() {
        return Err("distribution artifact must not be a symlink".to_owned());
    }
    let canonical_path = std::fs::canonicalize(path).map_err(|error| error.to_string())?;
    let path_before = regular_file_metadata(&canonical_path)?;
    let mut file = File::open(&canonical_path).map_err(|error| error.to_string())?;
    let opened = file.metadata().map_err(|error| error.to_string())?;
    if !same_file(&path_before, &opened) {
        return Err("distribution artifact changed while it was opened".to_owned());
    }
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    let mut bytes_read = 0_u64;
    loop {
        let count = file.read(&mut buffer).map_err(|error| error.to_string())?;
        if count == 0 {
            break;
        }
        bytes_read = bytes_read
            .checked_add(u64::try_from(count).expect("buffer length fits u64"))
            .ok_or_else(|| "distribution artifact is too large".to_owned())?;
        digest.update(&buffer[..count]);
    }
    let opened_after = file.metadata().map_err(|error| error.to_string())?;
    let path_after = regular_file_metadata(&canonical_path)?;
    if bytes_read != opened.len()
        || !same_file(&opened, &opened_after)
        || !same_file(&opened, &path_after)
    {
        return Err("distribution artifact changed while it was read".to_owned());
    }
    Ok(ArtifactDigest {
        canonical_path: canonical_path
            .into_os_string()
            .into_string()
            .map_err(|_| "distribution artifact path must be valid UTF-8".to_owned())?,
        sha256: digest.finalize().into(),
        size: opened.len(),
        device: opened.dev(),
        inode: opened.ino(),
    })
}

fn regular_file_metadata(path: &Path) -> Result<Metadata, String> {
    let metadata = std::fs::symlink_metadata(path).map_err(|error| error.to_string())?;
    if metadata.file_type().is_symlink() {
        return Err("distribution artifact must not be a symlink".to_owned());
    }
    if !metadata.is_file() {
        return Err("distribution artifact must be a regular file".to_owned());
    }
    Ok(metadata)
}

fn same_file(left: &Metadata, right: &Metadata) -> bool {
    left.dev() == right.dev()
        && left.ino() == right.ino()
        && left.len() == right.len()
        && left.mtime() == right.mtime()
        && left.mtime_nsec() == right.mtime_nsec()
        && left.ctime() == right.ctime()
        && left.ctime_nsec() == right.ctime_nsec()
}
