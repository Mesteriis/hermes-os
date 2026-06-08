// This file exceeds 700 lines because it groups the secret reference store,
// the database-backed encrypted secret vault, and their shared error types
// into a single security boundary. The vault depends on the reference store
// for lookup, and both share the same cryptographic primitives. Splitting
// would require either duplicating validation logic or introducing an
// indirection layer that adds complexity without clarifying the domain.

use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::future::Future;
use std::pin::Pin;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use thiserror::Error;

use std::path::{Path, PathBuf};

use aes_gcm::aead::rand_core::RngCore;
use aes_gcm::aead::{Aead, KeyInit, OsRng, Payload};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use argon2::Argon2;
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;

#[derive(Clone)]
pub struct SecretReferenceStore {
    pool: PgPool,
}

impl SecretReferenceStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert_secret_reference(
        &self,
        reference: &NewSecretReference,
    ) -> Result<SecretReference, SecretReferenceError> {
        reference.validate()?;

        let row = sqlx::query(
            r#"
            INSERT INTO secret_references (
                secret_ref,
                secret_kind,
                store_kind,
                label,
                metadata,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, now())
            ON CONFLICT (secret_ref)
            DO UPDATE SET
                secret_kind = EXCLUDED.secret_kind,
                store_kind = EXCLUDED.store_kind,
                label = EXCLUDED.label,
                metadata = EXCLUDED.metadata,
                updated_at = now()
            RETURNING
                secret_ref,
                secret_kind,
                store_kind,
                label,
                metadata,
                created_at,
                updated_at
            "#,
        )
        .bind(reference.secret_ref.trim())
        .bind(reference.secret_kind.as_str())
        .bind(reference.store_kind.as_str())
        .bind(reference.label.trim())
        .bind(&reference.metadata)
        .fetch_one(&self.pool)
        .await?;

        row_to_secret_reference(row)
    }

    pub async fn secret_reference(
        &self,
        secret_ref: &str,
    ) -> Result<Option<SecretReference>, SecretReferenceError> {
        validate_non_empty("secret_ref", secret_ref)?;

        let row = sqlx::query(
            r#"
            SELECT
                secret_ref,
                secret_kind,
                store_kind,
                label,
                metadata,
                created_at,
                updated_at
            FROM secret_references
            WHERE secret_ref = $1
            "#,
        )
        .bind(secret_ref.trim())
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_secret_reference).transpose()
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SecretKind {
    OauthToken,
    AppPassword,
    Password,
    ApiToken,
    PrivateKey,
    Other,
}

impl SecretKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::OauthToken => "oauth_token",
            Self::AppPassword => "app_password",
            Self::Password => "password",
            Self::ApiToken => "api_token",
            Self::PrivateKey => "private_key",
            Self::Other => "other",
        }
    }
}

impl TryFrom<&str> for SecretKind {
    type Error = SecretReferenceError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.trim() {
            "oauth_token" => Ok(Self::OauthToken),
            "app_password" => Ok(Self::AppPassword),
            "password" => Ok(Self::Password),
            "api_token" => Ok(Self::ApiToken),
            "private_key" => Ok(Self::PrivateKey),
            "other" => Ok(Self::Other),
            other => Err(SecretReferenceError::UnsupportedSecretKind(
                other.to_owned(),
            )),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SecretStoreKind {
    OsKeychain,
    EncryptedVault,
    DatabaseEncryptedVault,
    HostVault,
    ExternalVault,
    TestDouble,
}

impl SecretStoreKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::OsKeychain => "os_keychain",
            Self::EncryptedVault => "encrypted_vault",
            Self::DatabaseEncryptedVault => "database_encrypted_vault",
            Self::HostVault => "host_vault",
            Self::ExternalVault => "external_vault",
            Self::TestDouble => "test_double",
        }
    }
}

impl TryFrom<&str> for SecretStoreKind {
    type Error = SecretReferenceError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.trim() {
            "os_keychain" => Ok(Self::OsKeychain),
            "encrypted_vault" => Ok(Self::EncryptedVault),
            "database_encrypted_vault" => Ok(Self::DatabaseEncryptedVault),
            "host_vault" => Ok(Self::HostVault),
            "external_vault" => Ok(Self::ExternalVault),
            "test_double" => Ok(Self::TestDouble),
            other => Err(SecretReferenceError::UnsupportedStoreKind(other.to_owned())),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SecretReference {
    pub secret_ref: String,
    pub secret_kind: SecretKind,
    pub store_kind: SecretStoreKind,
    pub label: String,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub type SecretResolutionFuture<'a> =
    Pin<Box<dyn Future<Output = Result<ResolvedSecret, SecretResolutionError>> + Send + 'a>>;

pub trait SecretResolver {
    fn resolve<'a>(&'a self, reference: &'a SecretReference) -> SecretResolutionFuture<'a>;
}

#[derive(Clone, Eq, PartialEq)]
pub struct ResolvedSecret {
    value: String,
}

impl ResolvedSecret {
    pub fn new(value: impl Into<String>) -> Result<Self, SecretResolutionError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(SecretResolutionError::EmptySecretValue);
        }

        Ok(Self { value })
    }

    pub fn expose_for_runtime(&self) -> &str {
        &self.value
    }
}

impl fmt::Debug for ResolvedSecret {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ResolvedSecret")
            .field("value", &"<redacted>")
            .finish()
    }
}

#[derive(Clone, Debug, Default)]
pub struct InMemorySecretResolver {
    values: HashMap<String, ResolvedSecret>,
}

impl InMemorySecretResolver {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(
        &mut self,
        secret_ref: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<(), SecretResolutionError> {
        let secret_ref = secret_ref.into();
        validate_secret_resolution_ref(&secret_ref)?;
        let resolved_secret = ResolvedSecret::new(value)?;

        self.values
            .insert(secret_ref.trim().to_owned(), resolved_secret);

        Ok(())
    }

    fn resolve_reference(
        &self,
        reference: &SecretReference,
    ) -> Result<ResolvedSecret, SecretResolutionError> {
        if reference.store_kind != SecretStoreKind::TestDouble {
            return Err(SecretResolutionError::UnsupportedStoreKind(
                reference.store_kind.as_str().to_owned(),
            ));
        }

        validate_secret_resolution_ref(&reference.secret_ref)?;
        let secret_ref = reference.secret_ref.trim();

        self.values
            .get(secret_ref)
            .cloned()
            .ok_or_else(|| SecretResolutionError::MissingSecret {
                secret_ref: secret_ref.to_owned(),
            })
    }
}

impl SecretResolver for InMemorySecretResolver {
    fn resolve<'a>(&'a self, reference: &'a SecretReference) -> SecretResolutionFuture<'a> {
        Box::pin(std::future::ready(self.resolve_reference(reference)))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewSecretReference {
    pub secret_ref: String,
    pub secret_kind: SecretKind,
    pub store_kind: SecretStoreKind,
    pub label: String,
    pub metadata: Value,
}

impl NewSecretReference {
    pub fn new(
        secret_ref: impl Into<String>,
        secret_kind: SecretKind,
        store_kind: SecretStoreKind,
        label: impl Into<String>,
    ) -> Self {
        Self {
            secret_ref: secret_ref.into(),
            secret_kind,
            store_kind,
            label: label.into(),
            metadata: json!({}),
        }
    }

    pub fn metadata(mut self, metadata: Value) -> Self {
        self.metadata = metadata;
        self
    }

    fn validate(&self) -> Result<(), SecretReferenceError> {
        validate_non_empty("secret_ref", &self.secret_ref)?;
        validate_non_empty("label", &self.label)?;
        validate_object("metadata", &self.metadata)
    }
}

fn row_to_secret_reference(row: PgRow) -> Result<SecretReference, SecretReferenceError> {
    let secret_kind = SecretKind::try_from(row.try_get::<String, _>("secret_kind")?.as_str())?;
    let store_kind = SecretStoreKind::try_from(row.try_get::<String, _>("store_kind")?.as_str())?;

    Ok(SecretReference {
        secret_ref: row.try_get("secret_ref")?,
        secret_kind,
        store_kind,
        label: row.try_get("label")?,
        metadata: row.try_get("metadata")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn validate_non_empty(field_name: &'static str, value: &str) -> Result<(), SecretReferenceError> {
    if value.trim().is_empty() {
        return Err(SecretReferenceError::EmptyField(field_name));
    }

    Ok(())
}

fn validate_object(field_name: &'static str, value: &Value) -> Result<(), SecretReferenceError> {
    if !value.is_object() {
        return Err(SecretReferenceError::NonObjectJson(field_name));
    }

    Ok(())
}

fn validate_secret_resolution_ref(value: &str) -> Result<(), SecretResolutionError> {
    if value.trim().is_empty() {
        return Err(SecretResolutionError::EmptySecretRef);
    }

    Ok(())
}

#[derive(Debug, Error)]
pub enum SecretReferenceError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error("unsupported secret kind: {0}")]
    UnsupportedSecretKind(String),

    #[error("unsupported secret store kind: {0}")]
    UnsupportedStoreKind(String),

    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error("{0} must be a JSON object")]
    NonObjectJson(&'static str),
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum SecretResolutionError {
    #[error("secret_ref must not be empty")]
    EmptySecretRef,

    #[error("secret value must not be empty")]
    EmptySecretValue,

    #[error("secret reference was not found: {secret_ref}")]
    MissingSecret { secret_ref: String },

    #[error("secret store kind is not supported by in-memory resolver: {0}")]
    UnsupportedStoreKind(String),

    #[error("secret store operation failed: {message}")]
    StoreFailure { message: String },
}
use std::collections::BTreeMap;

fn validate_vault_field(field: &'static str, value: &str) -> Result<(), EncryptedVaultError> {
    if value.trim().is_empty() {
        return Err(EncryptedVaultError::EmptyField(field));
    }
    Ok(())
}

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
        validate_vault_field("secret_ref", secret_ref)?;
        validate_vault_field("secret value", value)?;

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
