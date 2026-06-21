use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use thiserror::Error;

use crate::platform::secrets::{ResolvedSecret, SecretKind, SecretReference};

mod channel_messages;
mod email_sync;
pub mod rfc822;

pub use channel_messages::{
    ProviderChannelMessage, ProviderChannelMessageStore, ProviderCommunicationMessagePortError,
    ProviderHeuristicMember, ProviderMessageAttachmentAnchor,
    ProviderMessageProjectionObservationContext, ProviderMessageReferenceSummary,
};
pub use email_sync::{EmailSyncPlanError, imap_mailbox_stream_id, plan_email_sync};

pub const DEFAULT_MAIL_SYNC_BLOB_ROOT: &str = "docker/data/mail";

#[derive(Debug, Error)]
pub enum CommunicationContractError {
    #[error("unsupported communication provider kind: {0}")]
    UnsupportedProviderKind(String),

    #[error("unsupported provider account secret purpose: {0}")]
    UnsupportedSecretPurpose(String),

    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error("{0} must be a JSON object")]
    NonObjectJson(&'static str),
}

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
    type Error = CommunicationContractError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.trim() {
            "gmail" => Ok(Self::Gmail),
            "icloud" => Ok(Self::Icloud),
            "imap" => Ok(Self::Imap),
            "telegram_user" => Ok(Self::TelegramUser),
            "telegram_bot" => Ok(Self::TelegramBot),
            "whatsapp_web" => Ok(Self::WhatsappWeb),
            other => Err(CommunicationContractError::UnsupportedProviderKind(
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

    pub(crate) fn validate(&self) -> Result<(), CommunicationContractError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("display_name", &self.display_name)?;
        validate_non_empty("external_account_id", &self.external_account_id)?;
        validate_object("config", &self.config)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct StoredRawCommunicationRecord {
    pub raw_record_id: String,
    pub observation_id: String,
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

    pub(crate) fn validate(&self) -> Result<(), CommunicationContractError> {
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmailSyncPlan {
    pub account_id: String,
    pub provider_kind: EmailProviderKind,
    pub credential_purpose: ProviderAccountSecretPurpose,
    pub stream_id: String,
    pub adapter_config: EmailSyncAdapterConfig,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EmailSyncAdapterConfig {
    Gmail {
        history_stream_id: String,
    },
    Imap {
        host: String,
        port: u16,
        tls: bool,
        mailbox: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FetchedCommunicationSourceMessage {
    pub provider_record_id: String,
    pub source_fingerprint: String,
    pub occurred_at: Option<DateTime<Utc>>,
    pub payload: Value,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmailSyncBatch {
    pub provider_kind: EmailProviderKind,
    pub stream_id: String,
    pub checkpoint: Option<Value>,
    pub messages: Vec<FetchedCommunicationSourceMessage>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmailSyncImportReport {
    pub inserted_or_existing_records: usize,
    pub checkpoint_saved: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmailSyncBlobImportReport {
    pub inserted_or_existing_records: usize,
    pub checkpoint_saved: bool,
    pub blobs_upserted: usize,
    pub raw_records: Vec<StoredRawCommunicationRecord>,
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
            Self::TelegramSessionKey | Self::WhatsappWebSessionKey => {
                matches!(secret_kind, SecretKind::PrivateKey | SecretKind::Other)
            }
        }
    }
}

impl TryFrom<&str> for ProviderAccountSecretPurpose {
    type Error = CommunicationContractError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.trim() {
            "oauth_token" => Ok(Self::OauthToken),
            "imap_password" => Ok(Self::ImapPassword),
            "smtp_password" => Ok(Self::SmtpPassword),
            "telegram_api_hash" => Ok(Self::TelegramApiHash),
            "telegram_session_key" => Ok(Self::TelegramSessionKey),
            "telegram_bot_token" => Ok(Self::TelegramBotToken),
            "whatsapp_web_session_key" => Ok(Self::WhatsappWebSessionKey),
            other => Err(CommunicationContractError::UnsupportedSecretPurpose(
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

    pub(crate) fn validate(&self) -> Result<(), CommunicationContractError> {
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

#[derive(Clone, Debug)]
pub struct OutgoingEmail {
    pub from: String,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub bcc: Vec<String>,
    pub subject: String,
    pub body_text: String,
    pub body_html: Option<String>,
    pub in_reply_to: Option<String>,
    pub references: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct SendResult {
    pub message_id: String,
    pub accepted_recipients: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub tls: bool,
    pub starttls: bool,
    pub username: String,
}

impl SmtpConfig {
    pub fn new(host: impl Into<String>, port: u16, tls: bool, username: impl Into<String>) -> Self {
        Self {
            host: host.into(),
            port,
            tls,
            starttls: false,
            username: username.into(),
        }
    }

    pub fn starttls(mut self, starttls: bool) -> Self {
        self.starttls = starttls;
        self
    }
}

#[derive(Debug, Error)]
pub enum EmailSendError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Tls(#[from] async_native_tls::Error),

    #[error("SMTP protocol error: {0}")]
    Protocol(String),

    #[error("provider send error: {0}")]
    Provider(String),
}

pub trait SmtpTransport: Clone + Send + Sync {
    fn send<'a>(
        &'a self,
        config: &'a SmtpConfig,
        password: &'a ResolvedSecret,
        email: &'a OutgoingEmail,
    ) -> Pin<Box<dyn Future<Output = Result<SendResult, EmailSendError>> + Send + 'a>>;
}

pub struct GmailOutboxSendRequest<'a> {
    pub account_id: &'a str,
    pub oauth_secret_ref: &'a str,
    pub api_base_url: &'a str,
    pub email: &'a OutgoingEmail,
}

pub trait GmailOutboxTransport: Clone + Send + Sync {
    fn send<'a>(
        &'a self,
        request: GmailOutboxSendRequest<'a>,
    ) -> Pin<Box<dyn Future<Output = Result<SendResult, EmailSendError>> + Send + 'a>>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GmailMessageListFetchRequest {
    pub account_id: String,
    pub max_results: u16,
    pub page_token: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GmailHistoryFetchRequest {
    pub account_id: String,
    pub start_history_id: String,
    pub max_results: u16,
    pub page_token: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImapMessageFetchRequest {
    pub account_id: String,
    pub provider_kind: EmailProviderKind,
    pub host: String,
    pub port: u16,
    pub tls: bool,
    pub mailbox: String,
    pub username: String,
    pub max_messages: usize,
    pub last_seen_uid: Option<u32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EmailProviderSyncErrorKind {
    MissingCredential,
    Credential,
    AccountSetup,
    ProviderNetwork,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
#[error("{kind:?}: {message}")]
pub struct EmailProviderSyncError {
    pub kind: EmailProviderSyncErrorKind,
    pub message: String,
    pub history_expired: bool,
}

impl EmailProviderSyncError {
    pub fn missing_credential() -> Self {
        Self {
            kind: EmailProviderSyncErrorKind::MissingCredential,
            message: "missing provider credential binding".to_owned(),
            history_expired: false,
        }
    }

    pub fn credential(error: impl std::fmt::Display) -> Self {
        Self {
            kind: EmailProviderSyncErrorKind::Credential,
            message: error.to_string(),
            history_expired: false,
        }
    }

    pub fn account_setup(error: impl std::fmt::Display) -> Self {
        Self {
            kind: EmailProviderSyncErrorKind::AccountSetup,
            message: error.to_string(),
            history_expired: false,
        }
    }

    pub fn provider_network(error: impl std::fmt::Display, history_expired: bool) -> Self {
        Self {
            kind: EmailProviderSyncErrorKind::ProviderNetwork,
            message: error.to_string(),
            history_expired,
        }
    }
}

pub type SharedEmailProviderSyncPort = Arc<dyn EmailProviderSyncPort>;

pub trait EmailProviderSyncPort: Send + Sync {
    fn fetch_gmail_message_list<'a>(
        &'a self,
        request: GmailMessageListFetchRequest,
    ) -> Pin<Box<dyn Future<Output = Result<EmailSyncBatch, EmailProviderSyncError>> + Send + 'a>>;

    fn fetch_gmail_history<'a>(
        &'a self,
        request: GmailHistoryFetchRequest,
    ) -> Pin<Box<dyn Future<Output = Result<EmailSyncBatch, EmailProviderSyncError>> + Send + 'a>>;

    fn fetch_imap_messages<'a>(
        &'a self,
        request: ImapMessageFetchRequest,
    ) -> Pin<Box<dyn Future<Output = Result<EmailSyncBatch, EmailProviderSyncError>> + Send + 'a>>;
}

fn validate_non_empty(field: &'static str, value: &str) -> Result<(), CommunicationContractError> {
    if value.trim().is_empty() {
        Err(CommunicationContractError::EmptyField(field))
    } else {
        Ok(())
    }
}

fn validate_object(field: &'static str, value: &Value) -> Result<(), CommunicationContractError> {
    if value.is_object() {
        Ok(())
    } else {
        Err(CommunicationContractError::NonObjectJson(field))
    }
}
