use std::collections::HashMap;
use std::fmt;
use std::future::Future;
use std::pin::Pin;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use thiserror::Error;

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
    ExternalVault,
    TestDouble,
}

impl SecretStoreKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::OsKeychain => "os_keychain",
            Self::EncryptedVault => "encrypted_vault",
            Self::DatabaseEncryptedVault => "database_encrypted_vault",
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
