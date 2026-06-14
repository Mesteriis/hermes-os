use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use aes_gcm::Nonce;
use aes_gcm::aead::{Aead, Payload};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::crypto::{
    NONCE_LEN, SALT_LEN, VAULT_KDF, VAULT_VERSION, encrypted_vault_cipher, random_bytes,
};
use super::errors::SecretResolutionError;
use super::models::{ResolvedSecret, SecretReference, SecretStoreKind};
use super::resolver::{SecretResolutionFuture, SecretResolver};
use super::validation::validate_vault_field;

#[derive(Clone)]
pub struct EncryptedSecretVault {
    path: PathBuf,
    master_key: ResolvedSecret,
}

impl EncryptedSecretVault {
    pub fn new(path: impl Into<PathBuf>, master_key: ResolvedSecret) -> Self {
        Self {
            path: path.into(),
            master_key,
        }
    }

    pub fn store_secret(&self, secret_ref: &str, value: &str) -> Result<(), EncryptedVaultError> {
        validate_vault_field("secret_ref", secret_ref)?;
        validate_vault_field("secret value", value)?;

        let mut file = self.load_or_create_file()?;
        let cipher = encrypted_vault_cipher(&self.master_key, &file.salt)?;
        let nonce = random_bytes::<NONCE_LEN>();
        let ciphertext = cipher
            .encrypt(
                Nonce::from_slice(&nonce),
                Payload {
                    msg: value.as_bytes(),
                    aad: secret_ref.trim().as_bytes(),
                },
            )
            .map_err(|_| EncryptedVaultError::Crypto)?;

        file.entries.insert(
            secret_ref.trim().to_owned(),
            EncryptedVaultEntry {
                nonce: BASE64_STANDARD.encode(nonce),
                ciphertext: BASE64_STANDARD.encode(ciphertext),
            },
        );
        self.save_file(&file)
    }

    fn load_or_create_file(&self) -> Result<EncryptedVaultFile, EncryptedVaultError> {
        if !self.path.exists() {
            return Ok(EncryptedVaultFile {
                version: VAULT_VERSION,
                kdf: VAULT_KDF.to_owned(),
                salt: BASE64_STANDARD.encode(random_bytes::<SALT_LEN>()),
                entries: BTreeMap::new(),
            });
        }

        let raw = fs::read_to_string(&self.path)?;
        let file: EncryptedVaultFile = serde_json::from_str(&raw)?;
        if file.version != VAULT_VERSION || file.kdf != VAULT_KDF {
            return Err(EncryptedVaultError::UnsupportedVaultFormat);
        }

        Ok(file)
    }

    fn load_file(&self) -> Result<Option<EncryptedVaultFile>, EncryptedVaultError> {
        if !self.path.exists() {
            return Ok(None);
        }

        self.load_or_create_file().map(Some)
    }

    fn save_file(&self, file: &EncryptedVaultFile) -> Result<(), EncryptedVaultError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        let temp_path = self.path.with_extension("tmp");
        let serialized = serde_json::to_vec_pretty(file)?;
        fs::write(&temp_path, serialized)?;
        fs::rename(temp_path, &self.path)?;

        Ok(())
    }

    fn resolve_secret(
        &self,
        reference: &SecretReference,
    ) -> Result<ResolvedSecret, SecretResolutionError> {
        if reference.store_kind != SecretStoreKind::EncryptedVault {
            return Err(SecretResolutionError::UnsupportedStoreKind(
                reference.store_kind.as_str().to_owned(),
            ));
        }

        let secret_ref = reference.secret_ref.trim();
        if secret_ref.is_empty() {
            return Err(SecretResolutionError::EmptySecretRef);
        }

        let Some(file) = self.load_file().map_err(secret_store_failure)? else {
            return Err(SecretResolutionError::MissingSecret {
                secret_ref: secret_ref.to_owned(),
            });
        };
        let Some(entry) = file.entries.get(secret_ref) else {
            return Err(SecretResolutionError::MissingSecret {
                secret_ref: secret_ref.to_owned(),
            });
        };

        let cipher =
            encrypted_vault_cipher(&self.master_key, &file.salt).map_err(secret_store_failure)?;
        let nonce = BASE64_STANDARD
            .decode(&entry.nonce)
            .map_err(|_| secret_store_failure(EncryptedVaultError::InvalidEncoding))?;
        let ciphertext = BASE64_STANDARD
            .decode(&entry.ciphertext)
            .map_err(|_| secret_store_failure(EncryptedVaultError::InvalidEncoding))?;
        let plaintext = cipher
            .decrypt(
                Nonce::from_slice(&nonce),
                Payload {
                    msg: &ciphertext,
                    aad: secret_ref.as_bytes(),
                },
            )
            .map_err(|_| secret_store_failure(EncryptedVaultError::Crypto))?;
        let value = String::from_utf8(plaintext)
            .map_err(|_| secret_store_failure(EncryptedVaultError::InvalidEncoding))?;

        ResolvedSecret::new(value)
    }
}

impl SecretResolver for EncryptedSecretVault {
    fn resolve<'a>(&'a self, reference: &'a SecretReference) -> SecretResolutionFuture<'a> {
        Box::pin(std::future::ready(self.resolve_secret(reference)))
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct EncryptedVaultFile {
    version: u8,
    kdf: String,
    salt: String,
    entries: BTreeMap<String, EncryptedVaultEntry>,
}

#[derive(Debug, Deserialize, Serialize)]
struct EncryptedVaultEntry {
    nonce: String,
    ciphertext: String,
}

fn secret_store_failure(error: EncryptedVaultError) -> SecretResolutionError {
    SecretResolutionError::StoreFailure {
        message: error.public_message(),
    }
}

#[derive(Debug, Error)]
pub enum EncryptedVaultError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error("unsupported encrypted vault format")]
    UnsupportedVaultFormat,

    #[error("invalid encrypted vault encoding")]
    InvalidEncoding,

    #[error("encrypted vault cryptographic operation failed")]
    Crypto,

    #[error("{0} must not be empty")]
    EmptyField(&'static str),
}

impl EncryptedVaultError {
    pub(super) fn public_message(&self) -> String {
        match self {
            Self::Crypto => "invalid vault key or corrupted encrypted vault".to_owned(),
            Self::InvalidEncoding => "invalid encrypted vault encoding".to_owned(),
            Self::UnsupportedVaultFormat => "unsupported encrypted vault format".to_owned(),
            Self::EmptyField(field) => format!("{field} must not be empty"),
            Self::Io(_) | Self::Json(_) => "encrypted vault read/write failed".to_owned(),
        }
    }
}
