use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use thiserror::Error;

use crate::secrets::{
    ResolvedSecret, SecretKind, SecretReference, SecretReferenceError, SecretReferenceStore,
    SecretResolutionError, SecretResolver,
};

#[derive(Clone)]
pub struct CommunicationIngestionStore {
    pool: PgPool,
}

impl CommunicationIngestionStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert_provider_account(
        &self,
        account: &NewProviderAccount,
    ) -> Result<ProviderAccount, CommunicationIngestionError> {
        account.validate()?;

        let row = sqlx::query(
            r#"
            INSERT INTO communication_provider_accounts (
                account_id,
                provider_kind,
                display_name,
                external_account_id,
                config,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, now())
            ON CONFLICT (account_id)
            DO UPDATE SET
                provider_kind = EXCLUDED.provider_kind,
                display_name = EXCLUDED.display_name,
                external_account_id = EXCLUDED.external_account_id,
                config = EXCLUDED.config,
                updated_at = now()
            RETURNING
                account_id,
                provider_kind,
                display_name,
                external_account_id,
                config,
                created_at,
                updated_at
            "#,
        )
        .bind(account.account_id.trim())
        .bind(account.provider_kind.as_str())
        .bind(account.display_name.trim())
        .bind(account.external_account_id.trim())
        .bind(&account.config)
        .fetch_one(&self.pool)
        .await?;

        row_to_provider_account(row)
    }

    pub async fn record_raw_source(
        &self,
        record: &NewRawCommunicationRecord,
    ) -> Result<StoredRawCommunicationRecord, CommunicationIngestionError> {
        record.validate()?;

        let inserted = sqlx::query(
            r#"
            INSERT INTO communication_raw_records (
                raw_record_id,
                account_id,
                record_kind,
                provider_record_id,
                source_fingerprint,
                import_batch_id,
                occurred_at,
                payload,
                provenance
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (account_id, record_kind, provider_record_id)
            DO NOTHING
            RETURNING
                raw_record_id,
                account_id,
                record_kind,
                provider_record_id,
                source_fingerprint,
                import_batch_id,
                occurred_at,
                captured_at,
                payload,
                provenance
            "#,
        )
        .bind(record.raw_record_id.trim())
        .bind(record.account_id.trim())
        .bind(record.record_kind.trim())
        .bind(record.provider_record_id.trim())
        .bind(record.source_fingerprint.trim())
        .bind(record.import_batch_id.trim())
        .bind(record.occurred_at)
        .bind(&record.payload)
        .bind(&record.provenance)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = inserted {
            return row_to_raw_record(row);
        }

        let row = sqlx::query(
            r#"
            SELECT
                raw_record_id,
                account_id,
                record_kind,
                provider_record_id,
                source_fingerprint,
                import_batch_id,
                occurred_at,
                captured_at,
                payload,
                provenance
            FROM communication_raw_records
            WHERE account_id = $1
              AND record_kind = $2
              AND provider_record_id = $3
            "#,
        )
        .bind(record.account_id.trim())
        .bind(record.record_kind.trim())
        .bind(record.provider_record_id.trim())
        .fetch_one(&self.pool)
        .await?;

        row_to_raw_record(row)
    }

    pub async fn save_checkpoint(
        &self,
        checkpoint: &NewIngestionCheckpoint,
    ) -> Result<IngestionCheckpoint, CommunicationIngestionError> {
        checkpoint.validate()?;

        let row = sqlx::query(
            r#"
            INSERT INTO communication_ingestion_checkpoints (
                account_id,
                stream_id,
                checkpoint,
                updated_at
            )
            VALUES ($1, $2, $3, now())
            ON CONFLICT (account_id, stream_id)
            DO UPDATE SET
                checkpoint = EXCLUDED.checkpoint,
                updated_at = now()
            RETURNING
                account_id,
                stream_id,
                checkpoint,
                updated_at
            "#,
        )
        .bind(checkpoint.account_id.trim())
        .bind(checkpoint.stream_id.trim())
        .bind(&checkpoint.checkpoint)
        .fetch_one(&self.pool)
        .await?;

        row_to_checkpoint(row)
    }

    pub async fn checkpoint(
        &self,
        account_id: &str,
        stream_id: &str,
    ) -> Result<Option<IngestionCheckpoint>, CommunicationIngestionError> {
        validate_non_empty("account_id", account_id)?;
        validate_non_empty("stream_id", stream_id)?;

        let row = sqlx::query(
            r#"
            SELECT
                account_id,
                stream_id,
                checkpoint,
                updated_at
            FROM communication_ingestion_checkpoints
            WHERE account_id = $1
              AND stream_id = $2
            "#,
        )
        .bind(account_id.trim())
        .bind(stream_id.trim())
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_checkpoint).transpose()
    }

    pub async fn bind_provider_account_secret(
        &self,
        binding: &NewProviderAccountSecretBinding,
    ) -> Result<ProviderAccountSecretBinding, CommunicationIngestionError> {
        binding.validate()?;

        let row = sqlx::query(
            r#"
            INSERT INTO communication_provider_account_secret_refs (
                account_id,
                secret_purpose,
                secret_ref,
                updated_at
            )
            VALUES ($1, $2, $3, now())
            ON CONFLICT (account_id, secret_purpose)
            DO UPDATE SET
                secret_ref = EXCLUDED.secret_ref,
                updated_at = now()
            RETURNING
                account_id,
                secret_purpose,
                secret_ref,
                created_at,
                updated_at
            "#,
        )
        .bind(binding.account_id.trim())
        .bind(binding.secret_purpose.as_str())
        .bind(binding.secret_ref.trim())
        .fetch_one(&self.pool)
        .await?;

        row_to_secret_binding(row)
    }

    pub async fn provider_account_secret_bindings(
        &self,
        account_id: &str,
    ) -> Result<Vec<ProviderAccountSecretBinding>, CommunicationIngestionError> {
        validate_non_empty("account_id", account_id)?;

        let rows = sqlx::query(
            r#"
            SELECT
                account_id,
                secret_purpose,
                secret_ref,
                created_at,
                updated_at
            FROM communication_provider_account_secret_refs
            WHERE account_id = $1
            ORDER BY secret_purpose ASC
            "#,
        )
        .bind(account_id.trim())
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_secret_binding).collect()
    }

    pub async fn provider_account_secret_binding(
        &self,
        account_id: &str,
        secret_purpose: ProviderAccountSecretPurpose,
    ) -> Result<Option<ProviderAccountSecretBinding>, CommunicationIngestionError> {
        validate_non_empty("account_id", account_id)?;

        let row = sqlx::query(
            r#"
            SELECT
                account_id,
                secret_purpose,
                secret_ref,
                created_at,
                updated_at
            FROM communication_provider_account_secret_refs
            WHERE account_id = $1
              AND secret_purpose = $2
            "#,
        )
        .bind(account_id.trim())
        .bind(secret_purpose.as_str())
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_secret_binding).transpose()
    }
}

pub struct ProviderCredentialReader<'a, R: SecretResolver + ?Sized> {
    communication_store: CommunicationIngestionStore,
    secret_store: SecretReferenceStore,
    resolver: &'a R,
}

impl<'a, R: SecretResolver + ?Sized> ProviderCredentialReader<'a, R> {
    pub fn new(
        communication_store: CommunicationIngestionStore,
        secret_store: SecretReferenceStore,
        resolver: &'a R,
    ) -> Self {
        Self {
            communication_store,
            secret_store,
            resolver,
        }
    }

    pub async fn read(
        &self,
        account_id: &str,
        secret_purpose: ProviderAccountSecretPurpose,
    ) -> Result<ProviderCredential, ProviderCredentialError> {
        validate_non_empty("account_id", account_id)?;

        let binding = self
            .communication_store
            .provider_account_secret_binding(account_id, secret_purpose)
            .await?
            .ok_or_else(|| ProviderCredentialError::MissingBinding {
                account_id: account_id.trim().to_owned(),
                secret_purpose,
            })?;
        let reference = self
            .secret_store
            .secret_reference(&binding.secret_ref)
            .await?
            .ok_or_else(|| ProviderCredentialError::MissingSecretReference {
                secret_ref: binding.secret_ref.clone(),
            })?;
        if !binding
            .secret_purpose
            .accepts_secret_kind(reference.secret_kind)
        {
            return Err(ProviderCredentialError::IncompatibleSecretKind {
                secret_ref: reference.secret_ref.clone(),
                secret_purpose: binding.secret_purpose,
                secret_kind: reference.secret_kind,
            });
        }

        let secret = self.resolver.resolve(&reference)?;

        Ok(ProviderCredential {
            binding,
            reference,
            secret,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderCredential {
    pub binding: ProviderAccountSecretBinding,
    pub reference: SecretReference,
    pub secret: ResolvedSecret,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EmailProviderKind {
    Gmail,
    Icloud,
    Imap,
}

impl EmailProviderKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Gmail => "gmail",
            Self::Icloud => "icloud",
            Self::Imap => "imap",
        }
    }
}

impl TryFrom<&str> for EmailProviderKind {
    type Error = CommunicationIngestionError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.trim() {
            "gmail" => Ok(Self::Gmail),
            "icloud" => Ok(Self::Icloud),
            "imap" => Ok(Self::Imap),
            other => Err(CommunicationIngestionError::UnsupportedProviderKind(
                other.to_owned(),
            )),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProviderAccount {
    pub account_id: String,
    pub provider_kind: EmailProviderKind,
    pub display_name: String,
    pub external_account_id: String,
    pub config: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewProviderAccount {
    pub account_id: String,
    pub provider_kind: EmailProviderKind,
    pub display_name: String,
    pub external_account_id: String,
    pub config: Value,
}

impl NewProviderAccount {
    pub fn new(
        account_id: impl Into<String>,
        provider_kind: EmailProviderKind,
        display_name: impl Into<String>,
        external_account_id: impl Into<String>,
    ) -> Self {
        Self {
            account_id: account_id.into(),
            provider_kind,
            display_name: display_name.into(),
            external_account_id: external_account_id.into(),
            config: json!({}),
        }
    }

    pub fn config(mut self, config: Value) -> Self {
        self.config = config;
        self
    }

    fn validate(&self) -> Result<(), CommunicationIngestionError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("display_name", &self.display_name)?;
        validate_non_empty("external_account_id", &self.external_account_id)?;
        validate_object("config", &self.config)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct StoredRawCommunicationRecord {
    pub raw_record_id: String,
    pub account_id: String,
    pub record_kind: String,
    pub provider_record_id: String,
    pub source_fingerprint: String,
    pub import_batch_id: String,
    pub occurred_at: Option<DateTime<Utc>>,
    pub captured_at: DateTime<Utc>,
    pub payload: Value,
    pub provenance: Value,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewRawCommunicationRecord {
    pub raw_record_id: String,
    pub account_id: String,
    pub record_kind: String,
    pub provider_record_id: String,
    pub source_fingerprint: String,
    pub import_batch_id: String,
    pub occurred_at: Option<DateTime<Utc>>,
    pub payload: Value,
    pub provenance: Value,
}

impl NewRawCommunicationRecord {
    pub fn new(
        raw_record_id: impl Into<String>,
        account_id: impl Into<String>,
        record_kind: impl Into<String>,
        provider_record_id: impl Into<String>,
        source_fingerprint: impl Into<String>,
        import_batch_id: impl Into<String>,
        payload: Value,
    ) -> Self {
        Self {
            raw_record_id: raw_record_id.into(),
            account_id: account_id.into(),
            record_kind: record_kind.into(),
            provider_record_id: provider_record_id.into(),
            source_fingerprint: source_fingerprint.into(),
            import_batch_id: import_batch_id.into(),
            occurred_at: None,
            payload,
            provenance: json!({}),
        }
    }

    pub fn occurred_at(mut self, occurred_at: DateTime<Utc>) -> Self {
        self.occurred_at = Some(occurred_at);
        self
    }

    pub fn provenance(mut self, provenance: Value) -> Self {
        self.provenance = provenance;
        self
    }

    fn validate(&self) -> Result<(), CommunicationIngestionError> {
        validate_non_empty("raw_record_id", &self.raw_record_id)?;
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("record_kind", &self.record_kind)?;
        validate_non_empty("provider_record_id", &self.provider_record_id)?;
        validate_non_empty("source_fingerprint", &self.source_fingerprint)?;
        validate_non_empty("import_batch_id", &self.import_batch_id)?;
        validate_object("payload", &self.payload)?;
        validate_object("provenance", &self.provenance)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IngestionCheckpoint {
    pub account_id: String,
    pub stream_id: String,
    pub checkpoint: Value,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewIngestionCheckpoint {
    pub account_id: String,
    pub stream_id: String,
    pub checkpoint: Value,
}

impl NewIngestionCheckpoint {
    pub fn new(
        account_id: impl Into<String>,
        stream_id: impl Into<String>,
        checkpoint: Value,
    ) -> Self {
        Self {
            account_id: account_id.into(),
            stream_id: stream_id.into(),
            checkpoint,
        }
    }

    fn validate(&self) -> Result<(), CommunicationIngestionError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("stream_id", &self.stream_id)?;
        validate_object("checkpoint", &self.checkpoint)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderAccountSecretPurpose {
    OauthToken,
    ImapPassword,
    SmtpPassword,
}

impl ProviderAccountSecretPurpose {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::OauthToken => "oauth_token",
            Self::ImapPassword => "imap_password",
            Self::SmtpPassword => "smtp_password",
        }
    }

    pub fn accepts_secret_kind(self, secret_kind: SecretKind) -> bool {
        match self {
            Self::OauthToken => secret_kind == SecretKind::OauthToken,
            Self::ImapPassword | Self::SmtpPassword => {
                matches!(secret_kind, SecretKind::AppPassword | SecretKind::Password)
            }
        }
    }
}

impl TryFrom<&str> for ProviderAccountSecretPurpose {
    type Error = CommunicationIngestionError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.trim() {
            "oauth_token" => Ok(Self::OauthToken),
            "imap_password" => Ok(Self::ImapPassword),
            "smtp_password" => Ok(Self::SmtpPassword),
            other => Err(CommunicationIngestionError::UnsupportedSecretPurpose(
                other.to_owned(),
            )),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProviderAccountSecretBinding {
    pub account_id: String,
    pub secret_purpose: ProviderAccountSecretPurpose,
    pub secret_ref: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewProviderAccountSecretBinding {
    pub account_id: String,
    pub secret_purpose: ProviderAccountSecretPurpose,
    pub secret_ref: String,
}

impl NewProviderAccountSecretBinding {
    pub fn new(
        account_id: impl Into<String>,
        secret_purpose: ProviderAccountSecretPurpose,
        secret_ref: impl Into<String>,
    ) -> Self {
        Self {
            account_id: account_id.into(),
            secret_purpose,
            secret_ref: secret_ref.into(),
        }
    }

    fn validate(&self) -> Result<(), CommunicationIngestionError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("secret_ref", &self.secret_ref)
    }
}

fn row_to_provider_account(row: PgRow) -> Result<ProviderAccount, CommunicationIngestionError> {
    let provider_kind =
        EmailProviderKind::try_from(row.try_get::<String, _>("provider_kind")?.as_str())?;

    Ok(ProviderAccount {
        account_id: row.try_get("account_id")?,
        provider_kind,
        display_name: row.try_get("display_name")?,
        external_account_id: row.try_get("external_account_id")?,
        config: row.try_get("config")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn row_to_raw_record(
    row: PgRow,
) -> Result<StoredRawCommunicationRecord, CommunicationIngestionError> {
    Ok(StoredRawCommunicationRecord {
        raw_record_id: row.try_get("raw_record_id")?,
        account_id: row.try_get("account_id")?,
        record_kind: row.try_get("record_kind")?,
        provider_record_id: row.try_get("provider_record_id")?,
        source_fingerprint: row.try_get("source_fingerprint")?,
        import_batch_id: row.try_get("import_batch_id")?,
        occurred_at: row.try_get("occurred_at")?,
        captured_at: row.try_get("captured_at")?,
        payload: row.try_get("payload")?,
        provenance: row.try_get("provenance")?,
    })
}

fn row_to_checkpoint(row: PgRow) -> Result<IngestionCheckpoint, CommunicationIngestionError> {
    Ok(IngestionCheckpoint {
        account_id: row.try_get("account_id")?,
        stream_id: row.try_get("stream_id")?,
        checkpoint: row.try_get("checkpoint")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn row_to_secret_binding(
    row: PgRow,
) -> Result<ProviderAccountSecretBinding, CommunicationIngestionError> {
    let secret_purpose = ProviderAccountSecretPurpose::try_from(
        row.try_get::<String, _>("secret_purpose")?.as_str(),
    )?;

    Ok(ProviderAccountSecretBinding {
        account_id: row.try_get("account_id")?,
        secret_purpose,
        secret_ref: row.try_get("secret_ref")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn validate_non_empty(
    field_name: &'static str,
    value: &str,
) -> Result<(), CommunicationIngestionError> {
    if value.trim().is_empty() {
        return Err(CommunicationIngestionError::EmptyField(field_name));
    }

    Ok(())
}

fn validate_object(
    field_name: &'static str,
    value: &Value,
) -> Result<(), CommunicationIngestionError> {
    if !value.is_object() {
        return Err(CommunicationIngestionError::NonObjectJson(field_name));
    }

    Ok(())
}

#[derive(Debug, Error)]
pub enum CommunicationIngestionError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error("unsupported email provider kind: {0}")]
    UnsupportedProviderKind(String),

    #[error("unsupported provider account secret purpose: {0}")]
    UnsupportedSecretPurpose(String),

    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error("{0} must be a JSON object")]
    NonObjectJson(&'static str),
}

#[derive(Debug, Error)]
pub enum ProviderCredentialError {
    #[error(transparent)]
    Communication(#[from] CommunicationIngestionError),

    #[error(transparent)]
    SecretReference(#[from] SecretReferenceError),

    #[error(transparent)]
    SecretResolution(#[from] SecretResolutionError),

    #[error(
        "provider account secret binding not found: account_id={account_id}, secret_purpose={secret_purpose:?}"
    )]
    MissingBinding {
        account_id: String,
        secret_purpose: ProviderAccountSecretPurpose,
    },

    #[error("provider account secret reference metadata was not found: {secret_ref}")]
    MissingSecretReference { secret_ref: String },

    #[error(
        "provider account secret kind is incompatible: secret_ref={secret_ref}, secret_purpose={secret_purpose:?}, secret_kind={secret_kind:?}"
    )]
    IncompatibleSecretKind {
        secret_ref: String,
        secret_purpose: ProviderAccountSecretPurpose,
        secret_kind: SecretKind,
    },
}
