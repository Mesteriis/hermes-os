use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::platform::observations::{ObservationOriginKind, ObservationStoreError};
use crate::platform::secrets::{ResolvedSecret, SecretKind, SecretReference};

mod email_sync;
mod raw_signals;
pub mod rfc822;

pub use email_sync::{EmailSyncPlanError, imap_mailbox_stream_id, plan_email_sync};
pub use raw_signals::{CommunicationRawSignalSource, build_communication_raw_signal_event};

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
    WhatsappBusinessCloud,
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
            Self::WhatsappBusinessCloud => "whatsapp_business_cloud",
        }
    }

    pub fn is_email(self) -> bool {
        matches!(self, Self::Gmail | Self::Icloud | Self::Imap)
    }

    pub fn is_telegram(self) -> bool {
        matches!(self, Self::TelegramUser | Self::TelegramBot)
    }

    pub fn is_whatsapp(self) -> bool {
        matches!(self, Self::WhatsappWeb | Self::WhatsappBusinessCloud)
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
            "whatsapp_business_cloud" => Ok(Self::WhatsappBusinessCloud),
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderChannelMessage {
    pub message_id: String,
    pub raw_record_id: String,
    pub account_id: String,
    pub provider_record_id: String,
    pub subject: String,
    pub sender: String,
    pub body_text: String,
    pub occurred_at: Option<DateTime<Utc>>,
    pub projected_at: DateTime<Utc>,
    pub channel_kind: String,
    pub conversation_id: String,
    pub sender_display_name: Option<String>,
    pub delivery_state: String,
    pub message_metadata: Value,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderMessageAttachmentAnchor {
    pub message_id: String,
    pub raw_record_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderMessageReferenceSummary {
    pub message_id: String,
    pub provider_record_id: String,
    pub conversation_id: Option<String>,
    pub subject: String,
    pub sender: String,
    pub sender_display_name: Option<String>,
    pub body_text: String,
    pub occurred_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderHeuristicMember {
    pub sender_id: String,
    pub sender_display_name: Option<String>,
    pub message_count: i64,
    pub last_message_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Copy)]
pub struct ProviderMessageProjectionObservationContext<'a> {
    pub channel_kinds: &'a [&'a str],
    pub relationship_kind: &'a str,
    pub actor: &'a str,
}

pub type ProviderChannelMessagePortFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, ProviderCommunicationMessagePortError>> + Send + 'a>>;

pub struct ProviderAttachmentDownloadStateUpdate<'a> {
    pub message_id: &'a str,
    pub provider_attachment_id: &'a str,
    pub provider_file_id: i64,
    pub download_state: &'a str,
    pub local_path: Option<&'a str>,
    pub size_bytes: Option<i64>,
    pub content_type: &'a str,
    pub filename: Option<&'a str>,
    pub observed_at: DateTime<Utc>,
    pub context: ProviderMessageProjectionObservationContext<'a>,
}

pub struct ProviderMessageObservationEvent<'a> {
    pub provider: &'a str,
    pub account_id: &'a str,
    pub channel_kind: &'a str,
    pub message_id: &'a str,
    pub external_message_id: &'a str,
    pub event_kind: &'a str,
    pub observed_at: DateTime<Utc>,
    pub external_event_id: Option<&'a str>,
    pub payload: &'a Value,
    pub causation_id: Option<&'a str>,
    pub correlation_id: Option<&'a str>,
}

pub type ProviderMessageObservationEventFuture<'a> = Pin<
    Box<
        dyn Future<Output = Result<Option<i64>, ProviderCommunicationMessagePortError>> + Send + 'a,
    >,
>;

pub type CommunicationRawRecordPortFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, ProviderCommunicationMessagePortError>> + Send + 'a>>;

pub trait CommunicationRawRecordCommandPort: Send + Sync {
    fn record_raw_source<'a>(
        &'a self,
        record: &'a NewRawCommunicationRecord,
    ) -> CommunicationRawRecordPortFuture<'a, StoredRawCommunicationRecord>;
}

pub trait ProviderMessageObservationEventPort: Send + Sync {
    fn append_provider_message_observation<'a>(
        &'a self,
        observation: ProviderMessageObservationEvent<'a>,
    ) -> ProviderMessageObservationEventFuture<'a>;
}

#[derive(Clone)]
pub struct EventStoreProviderMessageObservationEventPort {
    event_store: crate::platform::events::EventStore,
}

impl EventStoreProviderMessageObservationEventPort {
    pub fn new(pool: sqlx::postgres::PgPool) -> Self {
        Self {
            event_store: crate::platform::events::EventStore::new(pool),
        }
    }
}

impl ProviderMessageObservationEventPort for EventStoreProviderMessageObservationEventPort {
    fn append_provider_message_observation<'a>(
        &'a self,
        observation: ProviderMessageObservationEvent<'a>,
    ) -> ProviderMessageObservationEventFuture<'a> {
        Box::pin(async move {
            validate_provider_observation_event(&observation).map_err(|error| {
                ProviderCommunicationMessagePortError::InvalidRequest(error.to_string())
            })?;
            let payload_hash = sha256_json(observation.payload)?;
            let idempotency_key = provider_observation_idempotency_key(
                observation.provider,
                observation.account_id,
                observation.event_kind,
                observation.external_message_id,
                observation.external_event_id,
                &payload_hash,
            );
            let event_type =
                provider_observation_event_type(observation.provider, observation.event_kind);
            let builder = crate::platform::events::NewEventEnvelope::builder(
                format!(
                    "evt_provider_observation_{}",
                    stable_event_id_fragment(&idempotency_key)
                ),
                event_type,
                observation.observed_at,
                json!({
                    "kind": "provider_observation",
                    "provider": observation.provider,
                    "account_id": observation.account_id,
                    "source_id": idempotency_key,
                }),
                json!({
                    "kind": "provider_message",
                    "provider": observation.provider,
                    "id": observation.external_message_id,
                    "message_id": observation.message_id,
                }),
            )
            .payload(json!({
                "provider_kind": observation.channel_kind,
                "account_id": observation.account_id,
                "external_event_id": observation.external_event_id,
                "external_message_id": observation.external_message_id,
                "message_id": observation.message_id,
                "event_kind": observation.event_kind,
                "observed_at": observation.observed_at,
                "payload_hash": payload_hash,
                "payload": observation.payload,
            }))
            .provenance(json!({
                "provider": observation.provider,
                "ownership": "provider_observation_fact",
            }));
            let correlation_id = observation
                .correlation_id
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .unwrap_or(&idempotency_key);
            let mut builder = builder.correlation_id(correlation_id);
            if let Some(causation_id) = observation.causation_id {
                builder = builder.causation_id(causation_id);
            }
            let event = builder.build()?;

            self.event_store
                .append_for_dispatch_idempotent(&event)
                .await
                .map_err(Into::into)
        })
    }
}

#[derive(Debug, Error)]
pub enum ProviderCommunicationMessagePortError {
    #[error("invalid provider communication message request: {0}")]
    InvalidRequest(String),

    #[error(transparent)]
    ObservationStore(#[from] ObservationStoreError),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    EventStore(#[from] crate::platform::events::EventStoreError),

    #[error(transparent)]
    EventEnvelope(#[from] crate::platform::events::EventEnvelopeError),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
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
    WhatsappBusinessCloudAccessToken,
    WhatsappBusinessCloudAppSecret,
    WhatsappBusinessCloudWebhookVerifyToken,
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
            Self::WhatsappBusinessCloudAccessToken => "whatsapp_business_cloud_access_token",
            Self::WhatsappBusinessCloudAppSecret => "whatsapp_business_cloud_app_secret",
            Self::WhatsappBusinessCloudWebhookVerifyToken => {
                "whatsapp_business_cloud_webhook_verify_token"
            }
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
            Self::WhatsappBusinessCloudAccessToken
            | Self::WhatsappBusinessCloudAppSecret
            | Self::WhatsappBusinessCloudWebhookVerifyToken => secret_kind == SecretKind::ApiToken,
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
            "whatsapp_business_cloud_access_token" => Ok(Self::WhatsappBusinessCloudAccessToken),
            "whatsapp_business_cloud_app_secret" => Ok(Self::WhatsappBusinessCloudAppSecret),
            "whatsapp_business_cloud_webhook_verify_token" => {
                Ok(Self::WhatsappBusinessCloudWebhookVerifyToken)
            }
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

#[derive(Debug, Error)]
#[error("communication provider account port error: {0}")]
pub struct ProviderAccountPortError(pub String);

impl ProviderAccountPortError {
    pub fn new(error: impl std::fmt::Display) -> Self {
        Self(error.to_string())
    }
}

#[derive(Debug, Error)]
#[error("communication provider secret binding port error: {0}")]
pub struct ProviderSecretBindingPortError(pub String);

impl ProviderSecretBindingPortError {
    pub fn new(error: impl std::fmt::Display) -> Self {
        Self(error.to_string())
    }
}

pub trait ProviderAccountLookupPort: Send + Sync {
    fn get<'a>(
        &'a self,
        account_id: &'a str,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<Option<ProviderAccount>, ProviderAccountPortError>>
                + Send
                + 'a,
        >,
    >;

    fn list<'a>(
        &'a self,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<Vec<ProviderAccount>, ProviderAccountPortError>> + Send + 'a,
        >,
    >;
}

pub trait ProviderAccountCommandPort: ProviderAccountLookupPort {
    fn upsert<'a>(
        &'a self,
        account: &'a NewProviderAccount,
    ) -> Pin<Box<dyn Future<Output = Result<ProviderAccount, ProviderAccountPortError>> + Send + 'a>>;

    fn upsert_runtime_account<'a>(
        &'a self,
        account_id: String,
        provider_kind: String,
        display_name: String,
        external_account_id: String,
        config: Value,
    ) -> Pin<Box<dyn Future<Output = Result<ProviderAccount, ProviderAccountPortError>> + Send + 'a>>;

    fn update_config<'a>(
        &'a self,
        account_id: &'a str,
        config: &'a Value,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<Option<ProviderAccount>, ProviderAccountPortError>>
                + Send
                + 'a,
        >,
    >;

    fn update_config_with_origin<'a>(
        &'a self,
        account_id: &'a str,
        config: &'a Value,
        origin_kind: ObservationOriginKind,
        actor: &'a str,
        action: &'a str,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<Option<ProviderAccount>, ProviderAccountPortError>>
                + Send
                + 'a,
        >,
    >;

    fn mark_logged_out<'a>(
        &'a self,
        account_id: &'a str,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<Option<ProviderAccount>, ProviderAccountPortError>>
                + Send
                + 'a,
        >,
    >;

    fn delete_metadata<'a>(
        &'a self,
        account_id: &'a str,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<DeletedProviderAccount, ProviderAccountPortError>>
                + Send
                + 'a,
        >,
    >;
}

pub trait ProviderSecretBindingLookupPort: Send + Sync {
    fn list_for_account<'a>(
        &'a self,
        account_id: &'a str,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<
                        Vec<ProviderAccountSecretBinding>,
                        ProviderSecretBindingPortError,
                    >,
                > + Send
                + 'a,
        >,
    >;

    fn get_for_account<'a>(
        &'a self,
        account_id: &'a str,
        secret_purpose: ProviderAccountSecretPurpose,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<
                        Option<ProviderAccountSecretBinding>,
                        ProviderSecretBindingPortError,
                    >,
                > + Send
                + 'a,
        >,
    >;
}

pub trait ProviderSecretBindingCommandPort: ProviderSecretBindingLookupPort {
    fn bind<'a>(
        &'a self,
        binding: &'a NewProviderAccountSecretBinding,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<ProviderAccountSecretBinding, ProviderSecretBindingPortError>,
                > + Send
                + 'a,
        >,
    >;

    fn unbind_for_account<'a>(
        &'a self,
        account_id: &'a str,
        secret_purpose: ProviderAccountSecretPurpose,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<
                        Option<ProviderAccountSecretBinding>,
                        ProviderSecretBindingPortError,
                    >,
                > + Send
                + 'a,
        >,
    >;
}

pub trait ProviderChannelMessageLookupPort: Send + Sync {
    fn message_by_id<'a>(
        &'a self,
        message_id: &'a str,
        channel_kinds: &'a [&'a str],
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<
                        Option<ProviderChannelMessage>,
                        ProviderCommunicationMessagePortError,
                    >,
                > + Send
                + 'a,
        >,
    >;

    fn message_by_provider_record_id<'a>(
        &'a self,
        account_id: &'a str,
        provider_record_id: &'a str,
        channel_kinds: &'a [&'a str],
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<
                        Option<ProviderChannelMessage>,
                        ProviderCommunicationMessagePortError,
                    >,
                > + Send
                + 'a,
        >,
    >;

    fn recent_messages<'a>(
        &'a self,
        account_id: Option<&'a str>,
        conversation_id: Option<&'a str>,
        channel_kinds: &'a [&'a str],
        limit: i64,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<
                        Vec<ProviderChannelMessage>,
                        ProviderCommunicationMessagePortError,
                    >,
                > + Send
                + 'a,
        >,
    >;

    fn messages_by_ids<'a>(
        &'a self,
        message_ids: &'a [String],
        channel_kinds: &'a [&'a str],
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<
                        Vec<ProviderChannelMessage>,
                        ProviderCommunicationMessagePortError,
                    >,
                > + Send
                + 'a,
        >,
    >;

    fn search_messages<'a>(
        &'a self,
        account_id: Option<&'a str>,
        conversation_id: Option<&'a str>,
        query: &'a str,
        channel_kinds: &'a [&'a str],
        limit: i64,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<
                        Vec<ProviderChannelMessage>,
                        ProviderCommunicationMessagePortError,
                    >,
                > + Send
                + 'a,
        >,
    >;

    fn pinned_messages<'a>(
        &'a self,
        account_id: &'a str,
        conversation_id: &'a str,
        channel_kinds: &'a [&'a str],
        limit: i64,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<
                        Vec<ProviderChannelMessage>,
                        ProviderCommunicationMessagePortError,
                    >,
                > + Send
                + 'a,
        >,
    >;

    fn body_text<'a>(
        &'a self,
        message_id: &'a str,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<Option<String>, ProviderCommunicationMessagePortError>>
                + Send
                + 'a,
        >,
    >;

    fn message_ids_by_metadata_string<'a>(
        &'a self,
        metadata_key: &'a str,
        metadata_value: &'a str,
        channel_kinds: &'a [&'a str],
        limit: i64,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<Vec<String>, ProviderCommunicationMessagePortError>>
                + Send
                + 'a,
        >,
    >;

    fn message_id_by_provider_record_id<'a>(
        &'a self,
        account_id: &'a str,
        provider_record_id: &'a str,
        channel_kinds: &'a [&'a str],
    ) -> Pin<
        Box<
            dyn Future<Output = Result<Option<String>, ProviderCommunicationMessagePortError>>
                + Send
                + 'a,
        >,
    >;

    fn reference_summaries<'a>(
        &'a self,
        message_ids: &'a [String],
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<
                        Vec<ProviderMessageReferenceSummary>,
                        ProviderCommunicationMessagePortError,
                    >,
                > + Send
                + 'a,
        >,
    >;

    fn heuristic_members<'a>(
        &'a self,
        account_id: &'a str,
        conversation_id: &'a str,
        query: Option<&'a str>,
        channel_kinds: &'a [&'a str],
        limit: i64,
        offset: i64,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<
                        Vec<ProviderHeuristicMember>,
                        ProviderCommunicationMessagePortError,
                    >,
                > + Send
                + 'a,
        >,
    >;

    fn attachment_anchor<'a>(
        &'a self,
        account_id: &'a str,
        conversation_id: &'a str,
        provider_record_id: &'a str,
        channel_kinds: &'a [&'a str],
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<
                        Option<ProviderMessageAttachmentAnchor>,
                        ProviderCommunicationMessagePortError,
                    >,
                > + Send
                + 'a,
        >,
    >;

    fn unread_counts<'a>(
        &'a self,
        account_id: &'a str,
        conversation_id: &'a str,
        channel_kinds: &'a [&'a str],
        last_read_at: Option<DateTime<Utc>>,
    ) -> ProviderChannelMessagePortFuture<'a, (i64, i64)>;
}

pub trait ProviderChannelMessageCommandPort: ProviderChannelMessageLookupPort {
    fn apply_metadata<'a>(
        &'a self,
        message_id: &'a str,
        metadata: &'a Value,
        context: ProviderMessageProjectionObservationContext<'a>,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<
                        Option<ProviderChannelMessage>,
                        ProviderCommunicationMessagePortError,
                    >,
                > + Send
                + 'a,
        >,
    >;

    fn set_delivery_state<'a>(
        &'a self,
        message_id: &'a str,
        delivery_state: &'a str,
        observed_at: DateTime<Utc>,
        context: ProviderMessageProjectionObservationContext<'a>,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<
                        Option<ProviderChannelMessage>,
                        ProviderCommunicationMessagePortError,
                    >,
                > + Send
                + 'a,
        >,
    >;

    fn apply_content_update<'a>(
        &'a self,
        message_id: &'a str,
        body_text: &'a str,
        metadata: &'a Value,
        observed_at: DateTime<Utc>,
        context: ProviderMessageProjectionObservationContext<'a>,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<
                        Option<ProviderChannelMessage>,
                        ProviderCommunicationMessagePortError,
                    >,
                > + Send
                + 'a,
        >,
    >;

    fn apply_pinned_state<'a>(
        &'a self,
        message_id: &'a str,
        is_pinned: bool,
        observed_at: DateTime<Utc>,
        context: ProviderMessageProjectionObservationContext<'a>,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<
                        Option<ProviderChannelMessage>,
                        ProviderCommunicationMessagePortError,
                    >,
                > + Send
                + 'a,
        >,
    >;

    fn update_attachment_download_state<'a>(
        &'a self,
        update: ProviderAttachmentDownloadStateUpdate<'a>,
    ) -> ProviderChannelMessagePortFuture<'a, Option<ProviderChannelMessage>>;
}

fn validate_provider_observation_event(
    observation: &ProviderMessageObservationEvent<'_>,
) -> Result<(), CommunicationContractError> {
    validate_non_empty("provider", observation.provider)?;
    validate_non_empty("account_id", observation.account_id)?;
    validate_non_empty("channel_kind", observation.channel_kind)?;
    validate_non_empty("message_id", observation.message_id)?;
    validate_non_empty("external_message_id", observation.external_message_id)?;
    validate_non_empty("event_kind", observation.event_kind)?;
    validate_object("payload", observation.payload)
}

fn provider_observation_event_type(provider: &str, event_kind: &str) -> String {
    if provider == "telegram" {
        return match event_kind {
            "content_observed" => "signal.raw.telegram.message.content.observed".to_owned(),
            "metadata_observed" => "signal.raw.telegram.message.metadata.observed".to_owned(),
            "delivery_state_observed" => {
                "signal.raw.telegram.message.delivery_state.observed".to_owned()
            }
            "pinned_state_observed" => {
                "signal.raw.telegram.message.pinned_state.observed".to_owned()
            }
            "attachment_download_state_observed" => {
                "signal.raw.telegram.attachment.download_state.observed".to_owned()
            }
            other => format!("signal.raw.telegram.message.{other}.observed"),
        };
    }
    format!("integration.{provider}.message.{event_kind}")
}

fn provider_observation_idempotency_key(
    provider: &str,
    account_id: &str,
    event_kind: &str,
    external_message_id: &str,
    external_event_id: Option<&str>,
    payload_hash: &str,
) -> String {
    if let Some(external_event_id) = external_event_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return format!("{provider}:{account_id}:external_event:{external_event_id}");
    }
    format!("{provider}:{account_id}:{event_kind}:{external_message_id}:{payload_hash}")
}

fn stable_event_id_fragment(value: &str) -> String {
    value
        .chars()
        .map(|char| {
            if char.is_ascii_alphanumeric() {
                char
            } else {
                '_'
            }
        })
        .collect()
}

fn sha256_json(value: &Value) -> Result<String, ProviderCommunicationMessagePortError> {
    let encoded = serde_json::to_vec(value)?;
    let mut hasher = Sha256::new();
    hasher.update(encoded);
    Ok(format!("sha256:{:x}", hasher.finalize()))
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
