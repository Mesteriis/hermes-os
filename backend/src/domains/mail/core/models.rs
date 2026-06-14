use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::platform::secrets::{ResolvedSecret, SecretKind, SecretReference};

use super::errors::CommunicationIngestionError;
use super::validation::{validate_non_empty, validate_object};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CommunicationProviderKind {
    Gmail,
    Icloud,
    Imap,
    TelegramUser,
    TelegramBot,
    WhatsappWeb,
}

pub type EmailProviderKind = CommunicationProviderKind;

impl CommunicationProviderKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Gmail => "gmail",
            Self::Icloud => "icloud",
            Self::Imap => "imap",
            Self::TelegramUser => "telegram_user",
            Self::TelegramBot => "telegram_bot",
            Self::WhatsappWeb => "whatsapp_web",
        }
    }

    pub fn is_email(self) -> bool {
        matches!(self, Self::Gmail | Self::Icloud | Self::Imap)
    }

    pub fn is_telegram(self) -> bool {
        matches!(self, Self::TelegramUser | Self::TelegramBot)
    }

    pub fn is_whatsapp(self) -> bool {
        matches!(self, Self::WhatsappWeb)
    }
}

impl TryFrom<&str> for CommunicationProviderKind {
    type Error = CommunicationIngestionError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.trim() {
            "gmail" => Ok(Self::Gmail),
            "icloud" => Ok(Self::Icloud),
            "imap" => Ok(Self::Imap),
            "telegram_user" => Ok(Self::TelegramUser),
            "telegram_bot" => Ok(Self::TelegramBot),
            "whatsapp_web" => Ok(Self::WhatsappWeb),
            other => Err(CommunicationIngestionError::UnsupportedProviderKind(
                other.to_owned(),
            )),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProviderAccount {
    pub account_id: String,
    pub provider_kind: CommunicationProviderKind,
    pub display_name: String,
    pub external_account_id: String,
    pub config: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProviderAccountUsage {
    pub raw_record_count: i64,
    pub message_count: i64,
    pub checkpoint_count: i64,
}

impl ProviderAccountUsage {
    pub fn has_retained_evidence(&self) -> bool {
        self.raw_record_count > 0 || self.message_count > 0
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DeletedProviderAccount {
    pub account: Option<ProviderAccount>,
    pub unbound_secret_refs: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewProviderAccount {
    pub account_id: String,
    pub provider_kind: CommunicationProviderKind,
    pub display_name: String,
    pub external_account_id: String,
    pub config: Value,
}

impl NewProviderAccount {
    pub fn new(
        account_id: impl Into<String>,
        provider_kind: CommunicationProviderKind,
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

    pub(super) fn validate(&self) -> Result<(), CommunicationIngestionError> {
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

    pub(super) fn validate(&self) -> Result<(), CommunicationIngestionError> {
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

    pub(super) fn validate(&self) -> Result<(), CommunicationIngestionError> {
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
    TelegramApiHash,
    TelegramSessionKey,
    TelegramBotToken,
    WhatsappWebSessionKey,
}

impl ProviderAccountSecretPurpose {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::OauthToken => "oauth_token",
            Self::ImapPassword => "imap_password",
            Self::SmtpPassword => "smtp_password",
            Self::TelegramApiHash => "telegram_api_hash",
            Self::TelegramSessionKey => "telegram_session_key",
            Self::TelegramBotToken => "telegram_bot_token",
            Self::WhatsappWebSessionKey => "whatsapp_web_session_key",
        }
    }

    pub fn accepts_secret_kind(self, secret_kind: SecretKind) -> bool {
        match self {
            Self::OauthToken => secret_kind == SecretKind::OauthToken,
            Self::ImapPassword | Self::SmtpPassword => {
                matches!(secret_kind, SecretKind::AppPassword | SecretKind::Password)
            }
            Self::TelegramApiHash | Self::TelegramBotToken => secret_kind == SecretKind::ApiToken,
            Self::TelegramSessionKey => {
                matches!(secret_kind, SecretKind::PrivateKey | SecretKind::Other)
            }
            Self::WhatsappWebSessionKey => {
                matches!(secret_kind, SecretKind::PrivateKey | SecretKind::Other)
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
            "telegram_api_hash" => Ok(Self::TelegramApiHash),
            "telegram_session_key" => Ok(Self::TelegramSessionKey),
            "telegram_bot_token" => Ok(Self::TelegramBotToken),
            "whatsapp_web_session_key" => Ok(Self::WhatsappWebSessionKey),
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

    pub(super) fn validate(&self) -> Result<(), CommunicationIngestionError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("secret_ref", &self.secret_ref)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderCredential {
    pub binding: ProviderAccountSecretBinding,
    pub reference: SecretReference,
    pub secret: ResolvedSecret,
}
