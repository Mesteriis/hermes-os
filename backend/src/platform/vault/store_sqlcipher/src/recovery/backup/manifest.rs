use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::Path;

use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use getrandom::fill;
use sha2::{Digest, Sha256};

use crate::database::store::VaultStoreError;

const MAGIC: &[u8; 8] = b"HVBKMAN1";
const NONCE_BYTES: usize = 24;
const MAX_INSTANCE_ID_BYTES: usize = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VaultBackupClassV1 {
    EncryptedVaultState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VaultBackupManifestV1 {
    class: VaultBackupClassV1,
    instance_id: String,
    database_sha256: [u8; 32],
    anchor_sha256: [u8; 32],
}

impl VaultBackupManifestV1 {
    #[must_use]
    pub fn class(&self) -> VaultBackupClassV1 {
        self.class
    }
    #[must_use]
    pub fn instance_id(&self) -> &str {
        &self.instance_id
    }
    #[must_use]
    pub fn database_sha256(&self) -> &[u8; 32] {
        &self.database_sha256
    }
    #[must_use]
    pub fn anchor_sha256(&self) -> &[u8; 32] {
        &self.anchor_sha256
    }
}

pub(crate) fn create_manifest(
    path: &Path,
    instance_id: &str,
    database: &Path,
    anchor: &Path,
    key: &[u8; 32],
) -> Result<VaultBackupManifestV1, VaultStoreError> {
    let manifest = VaultBackupManifestV1 {
        class: VaultBackupClassV1::EncryptedVaultState,
        instance_id: instance_id.to_owned(),
        database_sha256: digest(database)?,
        anchor_sha256: digest(anchor)?,
    };
    let plaintext = encode_plaintext(&manifest)?;
    let bytes = encrypt(&plaintext, key)?;
    write_new_private_file(path, &bytes)?;
    Ok(manifest)
}

pub(crate) fn verify_manifest(
    path: &Path,
    database: &Path,
    anchor: &Path,
    key: &[u8; 32],
) -> Result<VaultBackupManifestV1, VaultStoreError> {
    let plaintext = decrypt(&read_private_regular_file(path)?, key)?;
    let manifest = decode_plaintext(&plaintext)?;
    if digest(database)? != manifest.database_sha256 || digest(anchor)? != manifest.anchor_sha256 {
        return Err(VaultStoreError::Backup);
    }
    Ok(manifest)
}

fn encode_plaintext(manifest: &VaultBackupManifestV1) -> Result<Vec<u8>, VaultStoreError> {
    let instance = manifest.instance_id.as_bytes();
    if instance.is_empty() || instance.len() > MAX_INSTANCE_ID_BYTES {
        return Err(VaultStoreError::Backup);
    }
    let length = u16::try_from(instance.len()).map_err(|_| VaultStoreError::Backup)?;
    let mut bytes = Vec::with_capacity(1 + 2 + instance.len() + 64);
    bytes.push(1);
    bytes.extend_from_slice(&length.to_be_bytes());
    bytes.extend_from_slice(instance);
    bytes.extend_from_slice(&manifest.database_sha256);
    bytes.extend_from_slice(&manifest.anchor_sha256);
    Ok(bytes)
}

fn decode_plaintext(bytes: &[u8]) -> Result<VaultBackupManifestV1, VaultStoreError> {
    let version = *bytes.first().ok_or(VaultStoreError::Backup)?;
    let length: [u8; 2] = bytes
        .get(1..3)
        .ok_or(VaultStoreError::Backup)?
        .try_into()
        .map_err(|_| VaultStoreError::Backup)?;
    let length = usize::from(u16::from_be_bytes(length));
    let instance_end = 3 + length;
    let digest_end = instance_end + 64;
    if version != 1 || length == 0 || length > MAX_INSTANCE_ID_BYTES || digest_end != bytes.len() {
        return Err(VaultStoreError::Backup);
    }
    let instance = std::str::from_utf8(&bytes[3..instance_end])
        .map_err(|_| VaultStoreError::Backup)?
        .to_owned();
    let database_sha256 = bytes[instance_end..instance_end + 32]
        .try_into()
        .map_err(|_| VaultStoreError::Backup)?;
    let anchor_sha256 = bytes[instance_end + 32..digest_end]
        .try_into()
        .map_err(|_| VaultStoreError::Backup)?;
    Ok(VaultBackupManifestV1 {
        class: VaultBackupClassV1::EncryptedVaultState,
        instance_id: instance,
        database_sha256,
        anchor_sha256,
    })
}

fn encrypt(plaintext: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, VaultStoreError> {
    let mut nonce = [0; NONCE_BYTES];
    fill(&mut nonce).map_err(|_| VaultStoreError::Backup)?;
    let cipher = XChaCha20Poly1305::new_from_slice(key).map_err(|_| VaultStoreError::Backup)?;
    let nonce = XNonce::try_from(nonce.as_slice()).map_err(|_| VaultStoreError::Backup)?;
    let ciphertext = cipher
        .encrypt(
            &nonce,
            Payload {
                msg: plaintext,
                aad: MAGIC,
            },
        )
        .map_err(|_| VaultStoreError::Backup)?;
    Ok([MAGIC.as_slice(), nonce.as_slice(), ciphertext.as_slice()].concat())
}

fn decrypt(bytes: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, VaultStoreError> {
    if bytes.len() <= MAGIC.len() + NONCE_BYTES || bytes[..MAGIC.len()] != *MAGIC {
        return Err(VaultStoreError::Backup);
    }
    let cipher = XChaCha20Poly1305::new_from_slice(key).map_err(|_| VaultStoreError::Backup)?;
    let nonce = XNonce::try_from(&bytes[MAGIC.len()..MAGIC.len() + NONCE_BYTES])
        .map_err(|_| VaultStoreError::Backup)?;
    cipher
        .decrypt(
            &nonce,
            Payload {
                msg: &bytes[MAGIC.len() + NONCE_BYTES..],
                aad: MAGIC,
            },
        )
        .map_err(|_| VaultStoreError::Backup)
}

fn digest(path: &Path) -> Result<[u8; 32], VaultStoreError> {
    Ok(Sha256::digest(read_private_regular_file(path)?).into())
}

fn write_new_private_file(path: &Path, bytes: &[u8]) -> Result<(), VaultStoreError> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
        .map_err(|_| VaultStoreError::Backup)?;
    file.write_all(bytes)
        .and_then(|_| file.sync_all())
        .map_err(|_| VaultStoreError::Backup)
}

fn read_private_regular_file(path: &Path) -> Result<Vec<u8>, VaultStoreError> {
    let before = std::fs::symlink_metadata(path).map_err(|_| VaultStoreError::Backup)?;
    if !before.is_file()
        || before.file_type().is_symlink()
        || before.uid() != unsafe { libc::geteuid() }
        || before.mode() & 0o077 != 0
    {
        return Err(VaultStoreError::Backup);
    }
    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
        .map_err(|_| VaultStoreError::Backup)?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .map_err(|_| VaultStoreError::Backup)?;
    let after = file.metadata().map_err(|_| VaultStoreError::Backup)?;
    if before.dev() != after.dev() || before.ino() != after.ino() || before.len() != after.len() {
        return Err(VaultStoreError::Backup);
    }
    Ok(bytes)
}
