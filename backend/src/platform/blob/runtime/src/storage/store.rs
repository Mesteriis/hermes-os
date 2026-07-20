//! Owner-fenced encrypted Blob reads and atomic writes.

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

use hermes_blob_protocol::{BlobAccessFenceV1, BlobRangeV1, BlobRefV1};

use crate::lease::{BlobKeyLeaseV1, BlobLeaseError};

use super::{format, root};

const MAX_BLOB_BYTES: u64 = 64 * 1024 * 1024;

pub struct EncryptedBlobStore {
    content_root: PathBuf,
    maximum_blob_bytes: u64,
}

impl EncryptedBlobStore {
    pub fn open(data_dir: &Path, maximum_blob_bytes: u64) -> Result<Self, BlobStorageError> {
        if maximum_blob_bytes == 0 || maximum_blob_bytes > MAX_BLOB_BYTES {
            return Err(BlobStorageError::InvalidQuota);
        }
        Ok(Self {
            content_root: root::prepare_content_root(data_dir)?,
            maximum_blob_bytes,
        })
    }

    pub fn write_new(
        &self,
        reference: &BlobRefV1,
        fence: &BlobAccessFenceV1,
        lease: &BlobKeyLeaseV1,
        plaintext: &[u8],
        now_unix_ms: u64,
    ) -> Result<(), BlobStorageError> {
        self.validate_write(reference, fence, plaintext, now_unix_ms)?;
        let key = lease
            .key_for(reference, fence, now_unix_ms)
            .map_err(BlobStorageError::Lease)?;
        let target = root::blob_path(&self.content_root, reference);
        reject_existing_path(&target)?;
        let encrypted = format::encrypt(reference, fence, key, plaintext)?;
        self.write_staged(&target, &encrypted)
    }

    pub fn read_range(
        &self,
        reference: &BlobRefV1,
        fence: &BlobAccessFenceV1,
        lease: &BlobKeyLeaseV1,
        range: BlobRangeV1,
        now_unix_ms: u64,
    ) -> Result<Vec<u8>, BlobStorageError> {
        if reference.is_expired_at(now_unix_ms) {
            return Err(BlobStorageError::Expired);
        }
        if fence.owner_id() != reference.owner_id() {
            return Err(BlobStorageError::FenceMismatch);
        }
        if range.end_exclusive() > reference.declared_size() {
            return Err(BlobStorageError::InvalidRange);
        }
        let key = lease
            .key_for(reference, fence, now_unix_ms)
            .map_err(BlobStorageError::Lease)?;
        let target = root::blob_path(&self.content_root, reference);
        root::validate_private_regular_file(&target)?;
        let encrypted = fs::read(target).map_err(|_| BlobStorageError::Filesystem)?;
        let plaintext = format::decrypt(reference, fence, key, &encrypted)?;
        if u64::try_from(plaintext.len()) != Ok(reference.declared_size()) {
            return Err(BlobStorageError::MalformedCiphertext);
        }
        let start = usize::try_from(range.start()).map_err(|_| BlobStorageError::InvalidRange)?;
        let end =
            usize::try_from(range.end_exclusive()).map_err(|_| BlobStorageError::InvalidRange)?;
        plaintext
            .get(start..end)
            .map(ToOwned::to_owned)
            .ok_or(BlobStorageError::InvalidRange)
    }

    /// Removes one owner-authorized Blob and syncs the containing private directory.
    pub fn delete(
        &self,
        reference: &BlobRefV1,
        fence: &BlobAccessFenceV1,
        lease: &BlobKeyLeaseV1,
        now_unix_ms: u64,
    ) -> Result<(), BlobStorageError> {
        if fence.owner_id() != reference.owner_id() {
            return Err(BlobStorageError::FenceMismatch);
        }
        lease
            .key_for(reference, fence, now_unix_ms)
            .map_err(BlobStorageError::Lease)?;
        let target = root::blob_path(&self.content_root, reference);
        root::validate_private_regular_file(&target)?;
        fs::remove_file(target).map_err(|_| BlobStorageError::Filesystem)?;
        self.sync_content_root()
    }

    pub(crate) fn exists(&self, reference: &BlobRefV1) -> Result<bool, BlobStorageError> {
        let target = root::blob_path(&self.content_root, reference);
        match fs::symlink_metadata(&target) {
            Ok(_) => {
                root::validate_private_regular_file(&target)?;
                Ok(true)
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(_) => Err(BlobStorageError::Filesystem),
        }
    }

    pub(crate) fn discard_uncommitted(
        &self,
        reference: &BlobRefV1,
    ) -> Result<(), BlobStorageError> {
        let target = root::blob_path(&self.content_root, reference);
        match fs::symlink_metadata(&target) {
            Ok(_) => {
                root::validate_private_regular_file(&target)?;
                fs::remove_file(target).map_err(|_| BlobStorageError::Filesystem)?;
                self.sync_content_root()
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(_) => Err(BlobStorageError::Filesystem),
        }
    }

    fn validate_write(
        &self,
        reference: &BlobRefV1,
        fence: &BlobAccessFenceV1,
        plaintext: &[u8],
        now_unix_ms: u64,
    ) -> Result<(), BlobStorageError> {
        if reference.is_expired_at(now_unix_ms) {
            return Err(BlobStorageError::Expired);
        }
        if fence.owner_id() != reference.owner_id()
            || reference.declared_size() > self.maximum_blob_bytes
            || u64::try_from(plaintext.len()) != Ok(reference.declared_size())
        {
            return Err(BlobStorageError::InvalidWrite);
        }
        Ok(())
    }

    fn write_staged(&self, target: &Path, encrypted: &[u8]) -> Result<(), BlobStorageError> {
        let staged = target.with_extension("staged");
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW)
            .open(&staged)
            .map_err(|_| BlobStorageError::AlreadyExists)?;
        let result = (|| {
            file.write_all(encrypted)
                .map_err(|_| BlobStorageError::Filesystem)?;
            file.sync_all().map_err(|_| BlobStorageError::Filesystem)?;
            root::validate_private_regular_file(&staged)?;
            reject_existing_path(target)?;
            fs::rename(&staged, target).map_err(|_| BlobStorageError::Filesystem)?;
            self.sync_content_root()
        })();
        if result.is_err() {
            let _ = fs::remove_file(&staged);
        }
        result
    }

    fn sync_content_root(&self) -> Result<(), BlobStorageError> {
        File::open(&self.content_root)
            .and_then(|directory| directory.sync_all())
            .map_err(|_| BlobStorageError::Filesystem)
    }
}

fn reject_existing_path(path: &Path) -> Result<(), BlobStorageError> {
    match fs::symlink_metadata(path) {
        Ok(_) => Err(BlobStorageError::AlreadyExists),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err(BlobStorageError::Filesystem),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlobStorageError {
    AlreadyExists,
    AuthenticationFailed,
    Crypto,
    Expired,
    FenceMismatch,
    Filesystem,
    InvalidQuota,
    InvalidRange,
    InvalidWrite,
    Lease(BlobLeaseError),
    MalformedCiphertext,
    Randomness,
    UnsafePath,
}
