use hermes_communications_api::accounts::{
    CommunicationProviderKind, DeletedProviderAccount, NewProviderAccount,
    NewProviderAccountSecretBinding, ProviderAccount, ProviderAccountPortError,
    ProviderAccountSecretBinding, ProviderAccountSecretPurpose, ProviderAccountUsage,
    ProviderSecretBindingPortError,
};
use hermes_communications_api::evidence::StoredRawCommunicationRecord;
use hermes_events_api::{EventEnvelopeError, NewEventEnvelope};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::platform::secrets::{ResolvedSecret, SecretKind, SecretReference};
use hermes_observations_api::models::ObservationOriginKind;
use hermes_observations_postgres::errors::ObservationStoreError;

mod attachment_text;
pub mod email_sync;
mod mbox;
mod raw_signals;
pub mod rfc822;

pub use attachment_text::{
    AttachmentTextExtractionError, MAX_ATTACHMENT_TEXT_EXTRACTION_BYTES,
    RichAttachmentContentDisarm, RichAttachmentExtractionKind, RichAttachmentExtractionResult,
    RichAttachmentSafePreview, disarm_rich_attachment, extract_local_attachment_text,
    extract_rich_attachment_text, is_locally_extractable_text_type,
    render_rich_attachment_safe_preview, rich_attachment_extraction_kind,
    rich_attachment_extractor_address,
};
pub use email_sync::{
    EmailSyncPlanError, IMAP_ALL_MAILBOXES, email_sync_plan_selects_all_imap_mailboxes,
    email_sync_plan_stream_ids, imap_mailbox_stream_id, imap_mailbox_stream_prefix,
    plan_email_sync,
};
pub use mbox::{MboxParseError, split_mbox_messages};
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
    pub communication_attachment_id: Option<&'a str>,
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

pub trait ProviderMessageObservationEventPort: Send + Sync {
    fn append_provider_message_observation<'a>(
        &'a self,
        observation: ProviderMessageObservationEvent<'a>,
    ) -> ProviderMessageObservationEventFuture<'a>;
}

#[derive(Clone)]
pub struct EventStoreProviderMessageObservationEventPort {
    event_store: hermes_events_postgres::store::EventStore,
}

impl EventStoreProviderMessageObservationEventPort {
    pub fn new(pool: sqlx::postgres::PgPool) -> Self {
        Self {
            event_store: hermes_events_postgres::store::EventStore::new(pool),
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
            let builder = hermes_events_api::NewEventEnvelope::builder(
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
    EventStore(#[from] hermes_events_postgres::errors::EventStoreError),

    #[error(transparent)]
    EventEnvelope(#[from] hermes_events_api::EventEnvelopeError),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmailSyncPlan {
    pub account_id: String,
    pub provider_kind: CommunicationProviderKind,
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
        mailboxes: Vec<String>,
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
    pub provider_kind: CommunicationProviderKind,
    pub stream_id: String,
    pub checkpoint: Option<Value>,
    pub messages: Vec<FetchedCommunicationSourceMessage>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AddressBookProviderFetchRequest {
    pub account_id: String,
    pub provider_kind: CommunicationProviderKind,
    pub provider_config: Value,
    pub page_token: Option<String>,
    pub page_size: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AddressBookProviderUpsertRequest {
    pub account_id: String,
    pub provider_kind: CommunicationProviderKind,
    pub provider_address_book_entry_id: Option<String>,
    pub provider_etag: Option<String>,
    pub display_name: String,
    pub email_address: Option<String>,
    pub phone_numbers: Vec<String>,
    pub remote_write_allowed: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AddressBookProviderEntry {
    pub provider_address_book_entry_id: String,
    pub display_name: Option<String>,
    pub email_addresses: Vec<String>,
    pub phone_numbers: Vec<String>,
    pub etag: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AddressBookProviderBatch {
    pub entries: Vec<AddressBookProviderEntry>,
    pub next_page_token: Option<String>,
}

pub type AddressBookProviderSyncFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, AddressBookProviderSyncError>> + Send + 'a>>;

pub type SharedAddressBookProviderSyncPort = Arc<dyn AddressBookProviderSyncPort>;

pub trait AddressBookProviderSyncPort: Send + Sync {
    fn fetch_entries<'a>(
        &'a self,
        request: AddressBookProviderFetchRequest,
    ) -> AddressBookProviderSyncFuture<'a, AddressBookProviderBatch>;

    fn upsert_entry<'a>(
        &'a self,
        request: AddressBookProviderUpsertRequest,
    ) -> AddressBookProviderSyncFuture<'a, AddressBookProviderEntry>;
}

#[derive(Debug, Error)]
pub enum AddressBookProviderSyncError {
    #[error("address book sync is not supported for provider: {0}")]
    UnsupportedProvider(String),

    #[error("provider credential error: {0}")]
    Credential(String),

    #[error("remote address book write is blocked: {0}")]
    RemoteWriteBlocked(&'static str),

    #[error("provider network error: {0}")]
    ProviderNetwork(String),
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderCredential {
    pub binding: ProviderAccountSecretBinding,
    pub reference: SecretReference,
    pub secret: ResolvedSecret,
}

#[derive(Clone, Debug)]
pub struct OutgoingEmail {
    pub from: String,
    pub message_id: Option<String>,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub bcc: Vec<String>,
    pub subject: String,
    pub body_text: String,
    pub body_html: Option<String>,
    pub in_reply_to: Option<String>,
    pub references: Vec<String>,
    pub attachments: Vec<OutgoingEmailAttachment>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutgoingEmailAttachment {
    pub filename: String,
    pub content_type: String,
    pub disposition: String,
    pub content_id: Option<String>,
    pub bytes: Vec<u8>,
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
    pub provider_kind: CommunicationProviderKind,
    pub host: String,
    pub port: u16,
    pub tls: bool,
    pub mailbox: String,
    pub username: String,
    pub max_messages: usize,
    pub last_seen_uid: Option<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImapMailboxListRequest {
    pub account_id: String,
    pub host: String,
    pub port: u16,
    pub tls: bool,
    pub username: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MailProviderResourceKind {
    Folder,
    Label,
}

impl MailProviderResourceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Folder => "folder",
            Self::Label => "label",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MailProviderSemanticRole {
    Inbox,
    Sent,
    Drafts,
    Archive,
    Trash,
    Junk,
    All,
    Flagged,
    Important,
    User,
}

impl MailProviderSemanticRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Inbox => "inbox",
            Self::Sent => "sent",
            Self::Drafts => "drafts",
            Self::Archive => "archive",
            Self::Trash => "trash",
            Self::Junk => "junk",
            Self::All => "all",
            Self::Flagged => "flagged",
            Self::Important => "important",
            Self::User => "user",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DiscoveredMailProviderResource {
    pub resource_kind: MailProviderResourceKind,
    pub provider_resource_id: String,
    pub display_name: String,
    pub semantic_role: Option<MailProviderSemanticRole>,
    pub selectable: bool,
    pub writable: bool,
    pub capabilities: Value,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GmailResourceDiscoveryRequest {
    pub account_id: String,
}

#[derive(Debug, Error)]
#[error("mail provider resource port error: {0}")]
pub struct MailProviderResourcePortError(pub String);

impl MailProviderResourcePortError {
    pub fn new(error: impl std::fmt::Display) -> Self {
        Self(error.to_string())
    }
}

pub trait MailProviderResourceCommandPort: Send + Sync {
    fn record_discovered_resources<'a>(
        &'a self,
        account_id: &'a str,
        resources: &'a [DiscoveredMailProviderResource],
    ) -> Pin<Box<dyn Future<Output = Result<(), MailProviderResourcePortError>> + Send + 'a>>;
}

pub type SharedMailProviderResourceCommandPort = Arc<dyn MailProviderResourceCommandPort>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImapIdleWaitRequest {
    pub account_id: String,
    pub host: String,
    pub port: u16,
    pub tls: bool,
    pub mailbox: String,
    pub username: String,
    pub timeout: std::time::Duration,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImapIdleWaitOutcome {
    Changed,
    TimedOut,
    Unsupported,
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

    fn list_imap_mailboxes<'a>(
        &'a self,
        request: ImapMailboxListRequest,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<String>, EmailProviderSyncError>> + Send + 'a>>;

    fn discover_gmail_resources<'a>(
        &'a self,
        request: GmailResourceDiscoveryRequest,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<Vec<DiscoveredMailProviderResource>, EmailProviderSyncError>>
                + Send
                + 'a,
        >,
    >;

    fn discover_imap_resources<'a>(
        &'a self,
        request: ImapMailboxListRequest,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<Vec<DiscoveredMailProviderResource>, EmailProviderSyncError>>
                + Send
                + 'a,
        >,
    >;

    fn wait_for_imap_change<'a>(
        &'a self,
        request: ImapIdleWaitRequest,
    ) -> Pin<
        Box<dyn Future<Output = Result<ImapIdleWaitOutcome, EmailProviderSyncError>> + Send + 'a>,
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
            "provider_identity_observed" => {
                "signal.raw.telegram.message.provider_identity.observed".to_owned()
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
