use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use aes_gcm::aead::rand_core::RngCore;
use aes_gcm::aead::{Aead, KeyInit, OsRng, Payload};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use argon2::Argon2;
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use thiserror::Error;

use crate::secrets::{
    ResolvedSecret, SecretReference, SecretResolutionError, SecretResolutionFuture, SecretResolver,
    SecretStoreKind,
};

const VAULT_VERSION: u8 = 1;
const VAULT_KDF: &str = "argon2id:v1";
const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;

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
        validate_non_empty("secret_ref", secret_ref)?;
        validate_non_empty("secret value", value)?;

        let mut file = self.load_or_create_file()?;
        let cipher = cipher(&self.master_key, &file.salt)?;
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

        let cipher = cipher(&self.master_key, &file.salt).map_err(secret_store_failure)?;
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

#[derive(Clone)]
pub struct DatabaseEncryptedSecretVault {
    pool: PgPool,
    master_key: ResolvedSecret,
}

impl DatabaseEncryptedSecretVault {
    pub fn new(pool: PgPool, master_key: ResolvedSecret) -> Self {
        Self { pool, master_key }
    }

    pub async fn store_secret(
        &self,
        secret_ref: &str,
        value: &str,
    ) -> Result<(), DatabaseEncryptedVaultError> {
        validate_database_non_empty("secret_ref", secret_ref)?;
        validate_database_non_empty("secret value", value)?;

        let secret_ref = secret_ref.trim();
        let salt = random_bytes::<SALT_LEN>();
        let encoded_salt = BASE64_STANDARD.encode(salt);
        let cipher = database_cipher(&self.master_key, &encoded_salt)?;
        let nonce = random_bytes::<NONCE_LEN>();
        let ciphertext = cipher
            .encrypt(
                Nonce::from_slice(&nonce),
                Payload {
                    msg: value.as_bytes(),
                    aad: secret_ref.as_bytes(),
                },
            )
            .map_err(|_| DatabaseEncryptedVaultError::Crypto)?;

        sqlx::query(
            r#"
            INSERT INTO encrypted_secret_vault_entries (
                secret_ref,
                kdf,
                salt,
                nonce,
                ciphertext,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, now())
            ON CONFLICT (secret_ref)
            DO UPDATE SET
                kdf = EXCLUDED.kdf,
                salt = EXCLUDED.salt,
                nonce = EXCLUDED.nonce,
                ciphertext = EXCLUDED.ciphertext,
                updated_at = now()
            "#,
        )
        .bind(secret_ref)
        .bind(VAULT_KDF)
        .bind(encoded_salt)
        .bind(BASE64_STANDARD.encode(nonce))
        .bind(BASE64_STANDARD.encode(ciphertext))
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn resolve_secret(
        &self,
        reference: &SecretReference,
    ) -> Result<ResolvedSecret, SecretResolutionError> {
        if reference.store_kind != SecretStoreKind::DatabaseEncryptedVault {
            return Err(SecretResolutionError::UnsupportedStoreKind(
                reference.store_kind.as_str().to_owned(),
            ));
        }

        let secret_ref = reference.secret_ref.trim();
        if secret_ref.is_empty() {
            return Err(SecretResolutionError::EmptySecretRef);
        }

        let Some(entry) = self
            .vault_entry(secret_ref)
            .await
            .map_err(database_secret_store_failure)?
        else {
            return Err(SecretResolutionError::MissingSecret {
                secret_ref: secret_ref.to_owned(),
            });
        };
        if entry.kdf != VAULT_KDF {
            return Err(database_secret_store_failure(
                DatabaseEncryptedVaultError::UnsupportedVaultFormat,
            ));
        }

        let cipher = database_cipher(&self.master_key, &entry.salt)
            .map_err(database_secret_store_failure)?;
        let nonce = BASE64_STANDARD.decode(&entry.nonce).map_err(|_| {
            database_secret_store_failure(DatabaseEncryptedVaultError::InvalidEncoding)
        })?;
        let ciphertext = BASE64_STANDARD.decode(&entry.ciphertext).map_err(|_| {
            database_secret_store_failure(DatabaseEncryptedVaultError::InvalidEncoding)
        })?;
        let plaintext = cipher
            .decrypt(
                Nonce::from_slice(&nonce),
                Payload {
                    msg: &ciphertext,
                    aad: secret_ref.as_bytes(),
                },
            )
            .map_err(|_| database_secret_store_failure(DatabaseEncryptedVaultError::Crypto))?;
        let value = String::from_utf8(plaintext).map_err(|_| {
            database_secret_store_failure(DatabaseEncryptedVaultError::InvalidEncoding)
        })?;

        ResolvedSecret::new(value)
    }

    async fn vault_entry(
        &self,
        secret_ref: &str,
    ) -> Result<Option<DatabaseEncryptedVaultEntry>, DatabaseEncryptedVaultError> {
        let row = sqlx::query(
            r#"
            SELECT
                kdf,
                salt,
                nonce,
                ciphertext
            FROM encrypted_secret_vault_entries
            WHERE secret_ref = $1
            "#,
        )
        .bind(secret_ref)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_database_entry).transpose()
    }
}

impl SecretResolver for DatabaseEncryptedSecretVault {
    fn resolve<'a>(&'a self, reference: &'a SecretReference) -> SecretResolutionFuture<'a> {
        Box::pin(self.resolve_secret(reference))
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

#[derive(Debug)]
struct DatabaseEncryptedVaultEntry {
    kdf: String,
    salt: String,
    nonce: String,
    ciphertext: String,
}

fn row_to_database_entry(
    row: PgRow,
) -> Result<DatabaseEncryptedVaultEntry, DatabaseEncryptedVaultError> {
    Ok(DatabaseEncryptedVaultEntry {
        kdf: row.try_get("kdf")?,
        salt: row.try_get("salt")?,
        nonce: row.try_get("nonce")?,
        ciphertext: row.try_get("ciphertext")?,
    })
}

fn cipher(
    master_key: &ResolvedSecret,
    encoded_salt: &str,
) -> Result<Aes256Gcm, EncryptedVaultError> {
    let salt = BASE64_STANDARD
        .decode(encoded_salt)
        .map_err(|_| EncryptedVaultError::InvalidEncoding)?;
    let mut key = [0_u8; 32];
    Argon2::default()
        .hash_password_into(master_key.expose_for_runtime().as_bytes(), &salt, &mut key)
        .map_err(|_| EncryptedVaultError::Crypto)?;

    Ok(Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key)))
}

fn database_cipher(
    master_key: &ResolvedSecret,
    encoded_salt: &str,
) -> Result<Aes256Gcm, DatabaseEncryptedVaultError> {
    let salt = BASE64_STANDARD
        .decode(encoded_salt)
        .map_err(|_| DatabaseEncryptedVaultError::InvalidEncoding)?;
    let mut key = [0_u8; 32];
    Argon2::default()
        .hash_password_into(master_key.expose_for_runtime().as_bytes(), &salt, &mut key)
        .map_err(|_| DatabaseEncryptedVaultError::Crypto)?;

    Ok(Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key)))
}

fn random_bytes<const N: usize>() -> [u8; N] {
    let mut bytes = [0_u8; N];
    OsRng.fill_bytes(&mut bytes);
    bytes
}

fn validate_non_empty(field: &'static str, value: &str) -> Result<(), EncryptedVaultError> {
    if value.trim().is_empty() {
        return Err(EncryptedVaultError::EmptyField(field));
    }

    Ok(())
}

fn validate_database_non_empty(
    field: &'static str,
    value: &str,
) -> Result<(), DatabaseEncryptedVaultError> {
    if value.trim().is_empty() {
        return Err(DatabaseEncryptedVaultError::EmptyField(field));
    }

    Ok(())
}

fn secret_store_failure(error: EncryptedVaultError) -> SecretResolutionError {
    SecretResolutionError::StoreFailure {
        message: error.public_message(),
    }
}

fn database_secret_store_failure(error: DatabaseEncryptedVaultError) -> SecretResolutionError {
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
    fn public_message(&self) -> String {
        match self {
            Self::Crypto => "invalid vault key or corrupted encrypted vault".to_owned(),
            Self::InvalidEncoding => "invalid encrypted vault encoding".to_owned(),
            Self::UnsupportedVaultFormat => "unsupported encrypted vault format".to_owned(),
            Self::EmptyField(field) => format!("{field} must not be empty"),
            Self::Io(_) | Self::Json(_) => "encrypted vault read/write failed".to_owned(),
        }
    }
}

#[derive(Debug, Error)]
pub enum DatabaseEncryptedVaultError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error("unsupported database encrypted vault format")]
    UnsupportedVaultFormat,

    #[error("invalid database encrypted vault encoding")]
    InvalidEncoding,

    #[error("database encrypted vault cryptographic operation failed")]
    Crypto,

    #[error("{0} must not be empty")]
    EmptyField(&'static str),
}

impl DatabaseEncryptedVaultError {
    fn public_message(&self) -> String {
        match self {
            Self::Crypto => "invalid vault key or corrupted encrypted database vault".to_owned(),
            Self::InvalidEncoding => "invalid encrypted database vault encoding".to_owned(),
            Self::UnsupportedVaultFormat => {
                "unsupported encrypted database vault format".to_owned()
            }
            Self::EmptyField(field) => format!("{field} must not be empty"),
            Self::Sqlx(_) => "encrypted database vault operation failed".to_owned(),
        }
    }
}

pub fn default_vault_path(home_dir: &Path) -> PathBuf {
    home_dir
        .join(".config")
        .join("hermes-hub")
        .join("secrets.vault.json")
}
