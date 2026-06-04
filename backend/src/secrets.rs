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
    ExternalVault,
    TestDouble,
}

impl SecretStoreKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::OsKeychain => "os_keychain",
            Self::EncryptedVault => "encrypted_vault",
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
