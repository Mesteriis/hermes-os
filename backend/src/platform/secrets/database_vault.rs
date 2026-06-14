use aes_gcm::Nonce;
use aes_gcm::aead::{Aead, Payload};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use thiserror::Error;

use super::crypto::{NONCE_LEN, SALT_LEN, VAULT_KDF, database_vault_cipher, random_bytes};
use super::errors::SecretResolutionError;
use super::models::{ResolvedSecret, SecretReference, SecretStoreKind};
use super::resolver::{SecretResolutionFuture, SecretResolver};
use super::validation::validate_database_non_empty;

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
        let cipher = database_vault_cipher(&self.master_key, &encoded_salt)?;
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

        let cipher = database_vault_cipher(&self.master_key, &entry.salt)
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

fn database_secret_store_failure(error: DatabaseEncryptedVaultError) -> SecretResolutionError {
    SecretResolutionError::StoreFailure {
        message: error.public_message(),
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
    pub(super) fn public_message(&self) -> String {
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
