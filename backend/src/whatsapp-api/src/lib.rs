//! WhatsApp provider contracts. This package contains no business-domain or WebView code.

use serde::{Deserialize, Serialize};

pub mod host_bridge;
pub mod capabilities;
pub mod client_wire;

pub mod wire {
    include!(concat!(env!("OUT_DIR"), "/hermes.whatsapp.v1.rs"));
}

pub const PACKAGE: &str = "hermes-whatsapp-api";
pub const MAX_ID_LEN: usize = 256;
pub const MAX_TEXT_BYTES: usize = 256 * 1024;

pub type WhatsAppAccountId = String;
pub type WhatsAppOperationId = String;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum WhatsAppCredentialPurpose {
    WebSessionKey,
}

impl WhatsAppCredentialPurpose {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WebSessionKey => "whatsapp_web_session_key",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WhatsAppCredentialBinding {
    pub purpose: WhatsAppCredentialPurpose,
    pub secret_ref: String,
    pub revision: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum WhatsAppProviderShape {
    WebCompanion,
}

impl WhatsAppProviderShape {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WebCompanion => "whatsapp_web_companion",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum WhatsAppRuntimeKind {
    HiddenWebView,
    Fixture,
}

impl WhatsAppRuntimeKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::HiddenWebView => "webview_companion",
            Self::Fixture => "fixture",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum WhatsAppAccountState {
    Provisioning,
    LinkRequired,
    Linked,
    Degraded,
    Revoked,
    Retired,
}

impl WhatsAppAccountState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Provisioning => "provisioning",
            Self::LinkRequired => "link_required",
            Self::Linked => "linked",
            Self::Degraded => "degraded",
            Self::Revoked => "revoked",
            Self::Retired => "retired",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum WhatsAppRuntimeState {
    Stopped,
    Starting,
    Running,
    Degraded,
    Blocked,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum WhatsAppConversationCommandKind {
    MarkRead,
    MarkUnread,
    Archive,
    Unarchive,
    Mute,
    Unmute,
    Pin,
    Unpin,
}

impl WhatsAppConversationCommandKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MarkRead => "mark_read",
            Self::MarkUnread => "mark_unread",
            Self::Archive => "archive",
            Self::Unarchive => "unarchive",
            Self::Mute => "mute",
            Self::Unmute => "unmute",
            Self::Pin => "pin",
            Self::Unpin => "unpin",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WhatsAppAccountSetup {
    pub account_id: WhatsAppAccountId,
    pub display_name: String,
    pub external_account_id: String,
    pub provider_shape: WhatsAppProviderShape,
    pub runtime_kind: WhatsAppRuntimeKind,
    pub credentials: Vec<WhatsAppCredentialBinding>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WhatsAppAccount {
    pub account_id: WhatsAppAccountId,
    pub display_name: String,
    pub external_account_id: String,
    pub provider_shape: WhatsAppProviderShape,
    pub runtime_kind: WhatsAppRuntimeKind,
    pub account_state: WhatsAppAccountState,
    pub runtime_state: WhatsAppRuntimeState,
    #[serde(default)]
    pub credentials: Vec<WhatsAppCredentialBinding>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum WhatsAppLifecycleRequest {
    Provision(WhatsAppAccountSetup),
    Start { account_id: WhatsAppAccountId },
    Stop { account_id: WhatsAppAccountId },
    Revoke { account_id: WhatsAppAccountId },
    Relink { account_id: WhatsAppAccountId },
    Remove { account_id: WhatsAppAccountId },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WhatsAppRuntimeStatus {
    pub account_id: WhatsAppAccountId,
    pub account_state: Option<WhatsAppAccountState>,
    pub runtime_state: Option<WhatsAppRuntimeState>,
    pub capabilities: Vec<capabilities::WhatsAppCapability>,
    pub host_command_queue_available: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WhatsAppMessage {
    pub account_id: WhatsAppAccountId,
    pub provider_chat_id: String,
    pub provider_message_id: String,
    pub sender_id: String,
    pub sender_display_name: String,
    pub text: Option<String>,
    pub reply_to_provider_message_id: Option<String>,
    pub occurred_at_unix_seconds: i64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WhatsAppDialog {
    pub account_id: WhatsAppAccountId,
    pub provider_chat_id: String,
    pub title: String,
    pub kind: String,
    pub is_archived: Option<bool>,
    pub is_pinned: Option<bool>,
    pub is_muted: Option<bool>,
    pub is_unread: Option<bool>,
    pub unread_count: Option<u64>,
    pub participant_count: Option<u64>,
    pub observed_at_unix_seconds: i64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WhatsAppParticipant {
    pub account_id: WhatsAppAccountId,
    pub provider_chat_id: String,
    pub provider_identity_id: String,
    pub display_name: String,
    pub role: String,
    pub status: String,
    pub is_self: bool,
    pub observed_at_unix_seconds: i64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WhatsAppMedia {
    pub account_id: WhatsAppAccountId,
    pub provider_chat_id: String,
    pub provider_message_id: String,
    pub provider_media_id: String,
    pub media_kind: String,
    pub filename: Option<String>,
    pub content_type: Option<String>,
    pub declared_size: Option<u64>,
    pub observed_at_unix_seconds: i64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum WhatsAppProviderEvent {
    RuntimeStateChanged {
        account_id: WhatsAppAccountId,
        state: WhatsAppRuntimeState,
        observed_at_unix_seconds: i64,
    },
    SessionStateChanged {
        account_id: WhatsAppAccountId,
        linked: bool,
        secret_ref: Option<String>,
        revision: Option<u64>,
        observed_at_unix_seconds: i64,
    },
    CommandResultObserved {
        account_id: WhatsAppAccountId,
        operation_id: WhatsAppOperationId,
        provider_request_id: Option<String>,
        succeeded: bool,
        observed_at_unix_seconds: i64,
    },
    MessageObserved(WhatsAppMessage),
    MessageEdited {
        account_id: WhatsAppAccountId,
        provider_chat_id: String,
        provider_message_id: String,
        text: Option<String>,
        observed_at_unix_seconds: i64,
    },
    MessageDeleted {
        account_id: WhatsAppAccountId,
        provider_chat_id: String,
        provider_message_id: String,
        observed_at_unix_seconds: i64,
    },
    ReceiptChanged {
        account_id: WhatsAppAccountId,
        provider_chat_id: String,
        provider_message_id: String,
        delivery_state: String,
        observed_at_unix_seconds: i64,
    },
    ReactionChanged {
        account_id: WhatsAppAccountId,
        provider_chat_id: String,
        provider_message_id: String,
        actor_id: String,
        emoji: Option<String>,
        is_active: bool,
        observed_at_unix_seconds: i64,
    },
    DialogObserved(WhatsAppDialog),
    ParticipantObserved(WhatsAppParticipant),
    PresenceChanged {
        account_id: WhatsAppAccountId,
        provider_chat_id: String,
        provider_identity_id: String,
        state: String,
        observed_at_unix_seconds: i64,
    },
    CallObserved {
        account_id: WhatsAppAccountId,
        provider_call_id: String,
        provider_chat_id: String,
        direction: String,
        state: String,
        observed_at_unix_seconds: i64,
    },
    StatusObserved {
        account_id: WhatsAppAccountId,
        provider_status_id: String,
        sender_id: String,
        text: Option<String>,
        observed_at_unix_seconds: i64,
    },
    StatusViewObserved {
        account_id: WhatsAppAccountId,
        provider_status_id: String,
        viewer_id: String,
        observed_at_unix_seconds: i64,
    },
    StatusDeleted {
        account_id: WhatsAppAccountId,
        provider_status_id: String,
        observed_at_unix_seconds: i64,
    },
    MediaObserved(WhatsAppMedia),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum WhatsAppProviderCommand {
    SendText {
        operation_id: WhatsAppOperationId,
        account_id: WhatsAppAccountId,
        provider_chat_id: String,
        text: String,
    },
    Reply {
        operation_id: WhatsAppOperationId,
        account_id: WhatsAppAccountId,
        provider_chat_id: String,
        reply_to_provider_message_id: String,
        text: String,
    },
    Forward {
        operation_id: WhatsAppOperationId,
        account_id: WhatsAppAccountId,
        provider_chat_id: String,
        source_provider_chat_id: String,
        source_provider_message_id: String,
    },
    Edit {
        operation_id: WhatsAppOperationId,
        account_id: WhatsAppAccountId,
        provider_chat_id: String,
        provider_message_id: String,
        text: String,
    },
    Delete {
        operation_id: WhatsAppOperationId,
        account_id: WhatsAppAccountId,
        provider_chat_id: String,
        provider_message_id: String,
    },
    React {
        operation_id: WhatsAppOperationId,
        account_id: WhatsAppAccountId,
        provider_chat_id: String,
        provider_message_id: String,
        emoji: String,
    },
    Unreact {
        operation_id: WhatsAppOperationId,
        account_id: WhatsAppAccountId,
        provider_chat_id: String,
        provider_message_id: String,
        emoji: String,
    },
    SendMedia {
        operation_id: WhatsAppOperationId,
        account_id: WhatsAppAccountId,
        provider_chat_id: String,
        blob_ref: String,
        media_kind: String,
        caption: Option<String>,
        filename: Option<String>,
    },
    SendVoiceNote {
        operation_id: WhatsAppOperationId,
        account_id: WhatsAppAccountId,
        provider_chat_id: String,
        attachment_id: String,
        blob_ref: String,
        content_type: String,
        declared_size: u64,
        sha256: String,
        scan_status: String,
        filename: Option<String>,
    },
    DownloadMedia {
        operation_id: WhatsAppOperationId,
        account_id: WhatsAppAccountId,
        provider_chat_id: String,
        provider_message_id: String,
        provider_media_id: String,
    },
    PublishStatus {
        operation_id: WhatsAppOperationId,
        account_id: WhatsAppAccountId,
        text: String,
    },
    JoinConversation {
        operation_id: WhatsAppOperationId,
        account_id: WhatsAppAccountId,
        provider_chat_id: String,
        invite_link: String,
    },
    LeaveConversation {
        operation_id: WhatsAppOperationId,
        account_id: WhatsAppAccountId,
        provider_chat_id: String,
    },
    Conversation {
        operation_id: WhatsAppOperationId,
        account_id: WhatsAppAccountId,
        provider_chat_id: String,
        action: WhatsAppConversationCommandKind,
    },
}

pub fn provider_command_account_id(command: &WhatsAppProviderCommand) -> &str {
    match command {
        WhatsAppProviderCommand::SendText { account_id, .. }
        | WhatsAppProviderCommand::Reply { account_id, .. }
        | WhatsAppProviderCommand::Forward { account_id, .. }
        | WhatsAppProviderCommand::Edit { account_id, .. }
        | WhatsAppProviderCommand::Delete { account_id, .. }
        | WhatsAppProviderCommand::React { account_id, .. }
        | WhatsAppProviderCommand::Unreact { account_id, .. }
        | WhatsAppProviderCommand::SendMedia { account_id, .. }
        | WhatsAppProviderCommand::SendVoiceNote { account_id, .. }
        | WhatsAppProviderCommand::DownloadMedia { account_id, .. }
        | WhatsAppProviderCommand::PublishStatus { account_id, .. }
        | WhatsAppProviderCommand::JoinConversation { account_id, .. }
        | WhatsAppProviderCommand::LeaveConversation { account_id, .. } => account_id,
        WhatsAppProviderCommand::Conversation { account_id, .. } => account_id,
    }
}

pub fn provider_command_operation_id(command: &WhatsAppProviderCommand) -> &str {
    match command {
        WhatsAppProviderCommand::SendText { operation_id, .. }
        | WhatsAppProviderCommand::Reply { operation_id, .. }
        | WhatsAppProviderCommand::Forward { operation_id, .. }
        | WhatsAppProviderCommand::Edit { operation_id, .. }
        | WhatsAppProviderCommand::Delete { operation_id, .. }
        | WhatsAppProviderCommand::React { operation_id, .. }
        | WhatsAppProviderCommand::Unreact { operation_id, .. }
        | WhatsAppProviderCommand::SendMedia { operation_id, .. }
        | WhatsAppProviderCommand::SendVoiceNote { operation_id, .. }
        | WhatsAppProviderCommand::DownloadMedia { operation_id, .. }
        | WhatsAppProviderCommand::PublishStatus { operation_id, .. }
        | WhatsAppProviderCommand::JoinConversation { operation_id, .. }
        | WhatsAppProviderCommand::LeaveConversation { operation_id, .. } => operation_id,
        WhatsAppProviderCommand::Conversation { operation_id, .. } => operation_id,
    }
}

pub fn provider_event_account_id(event: &WhatsAppProviderEvent) -> &str {
    match event {
        WhatsAppProviderEvent::RuntimeStateChanged { account_id, .. }
        | WhatsAppProviderEvent::SessionStateChanged { account_id, .. }
        | WhatsAppProviderEvent::CommandResultObserved { account_id, .. }
        | WhatsAppProviderEvent::MessageEdited { account_id, .. }
        | WhatsAppProviderEvent::MessageDeleted { account_id, .. }
        | WhatsAppProviderEvent::ReceiptChanged { account_id, .. }
        | WhatsAppProviderEvent::ReactionChanged { account_id, .. }
        | WhatsAppProviderEvent::PresenceChanged { account_id, .. }
        | WhatsAppProviderEvent::CallObserved { account_id, .. }
        | WhatsAppProviderEvent::StatusObserved { account_id, .. }
        | WhatsAppProviderEvent::StatusViewObserved { account_id, .. }
        | WhatsAppProviderEvent::StatusDeleted { account_id, .. } => account_id,
        WhatsAppProviderEvent::MessageObserved(value) => &value.account_id,
        WhatsAppProviderEvent::DialogObserved(value) => &value.account_id,
        WhatsAppProviderEvent::ParticipantObserved(value) => &value.account_id,
        WhatsAppProviderEvent::MediaObserved(value) => &value.account_id,
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WhatsAppRealtimeFrame {
    pub account_id: WhatsAppAccountId,
    pub sequence: u64,
    pub event: WhatsAppProviderEvent,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WhatsAppProviderEventKind {
    RuntimeState,
    Message,
    MessageEdited,
    MessageDeleted,
    Receipt,
    Reaction,
    Dialog,
    Participant,
    Presence,
    Call,
    Status,
    StatusView,
    StatusDeleted,
    Media,
    Session,
    CommandResult,
}

pub fn provider_event_kind(event: &WhatsAppProviderEvent) -> WhatsAppProviderEventKind {
    match event {
        WhatsAppProviderEvent::RuntimeStateChanged { .. } => WhatsAppProviderEventKind::RuntimeState,
        WhatsAppProviderEvent::SessionStateChanged { .. } => WhatsAppProviderEventKind::Session,
        WhatsAppProviderEvent::CommandResultObserved { .. } => WhatsAppProviderEventKind::CommandResult,
        WhatsAppProviderEvent::MessageObserved(_) => WhatsAppProviderEventKind::Message,
        WhatsAppProviderEvent::MessageEdited { .. } => WhatsAppProviderEventKind::MessageEdited,
        WhatsAppProviderEvent::MessageDeleted { .. } => WhatsAppProviderEventKind::MessageDeleted,
        WhatsAppProviderEvent::ReceiptChanged { .. } => WhatsAppProviderEventKind::Receipt,
        WhatsAppProviderEvent::ReactionChanged { .. } => WhatsAppProviderEventKind::Reaction,
        WhatsAppProviderEvent::DialogObserved(_) => WhatsAppProviderEventKind::Dialog,
        WhatsAppProviderEvent::ParticipantObserved(_) => WhatsAppProviderEventKind::Participant,
        WhatsAppProviderEvent::PresenceChanged { .. } => WhatsAppProviderEventKind::Presence,
        WhatsAppProviderEvent::CallObserved { .. } => WhatsAppProviderEventKind::Call,
        WhatsAppProviderEvent::StatusObserved { .. } => WhatsAppProviderEventKind::Status,
        WhatsAppProviderEvent::StatusViewObserved { .. } => WhatsAppProviderEventKind::StatusView,
        WhatsAppProviderEvent::StatusDeleted { .. } => WhatsAppProviderEventKind::StatusDeleted,
        WhatsAppProviderEvent::MediaObserved(_) => WhatsAppProviderEventKind::Media,
    }
}

pub fn provider_event_chat_id(event: &WhatsAppProviderEvent) -> Option<&str> {
    match event {
        WhatsAppProviderEvent::MessageObserved(value) => Some(&value.provider_chat_id),
        WhatsAppProviderEvent::MessageEdited { provider_chat_id, .. }
        | WhatsAppProviderEvent::MessageDeleted { provider_chat_id, .. }
        | WhatsAppProviderEvent::ReceiptChanged { provider_chat_id, .. }
        | WhatsAppProviderEvent::ReactionChanged { provider_chat_id, .. }
        | WhatsAppProviderEvent::PresenceChanged { provider_chat_id, .. }
        | WhatsAppProviderEvent::CallObserved { provider_chat_id, .. }
        | WhatsAppProviderEvent::MediaObserved(WhatsAppMedia { provider_chat_id, .. }) => {
            Some(provider_chat_id)
        }
        WhatsAppProviderEvent::DialogObserved(value) => Some(&value.provider_chat_id),
        WhatsAppProviderEvent::ParticipantObserved(value) => Some(&value.provider_chat_id),
        WhatsAppProviderEvent::RuntimeStateChanged { .. }
        | WhatsAppProviderEvent::SessionStateChanged { .. }
        | WhatsAppProviderEvent::CommandResultObserved { .. }
        | WhatsAppProviderEvent::StatusObserved { .. }
        | WhatsAppProviderEvent::StatusViewObserved { .. }
        | WhatsAppProviderEvent::StatusDeleted { .. } => None,
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum WhatsAppProviderQuery {
    Account {
        account_id: WhatsAppAccountId,
    },
    RuntimeStatus {
        account_id: WhatsAppAccountId,
    },
    CachedMessages {
        account_id: WhatsAppAccountId,
        provider_chat_id: Option<String>,
        limit: u32,
    },
    SearchMessages {
        account_id: WhatsAppAccountId,
        provider_chat_id: Option<String>,
        query: String,
        limit: u32,
    },
    Dialogs {
        account_id: WhatsAppAccountId,
        limit: u32,
    },
    Participants {
        account_id: WhatsAppAccountId,
        provider_chat_id: String,
        limit: u32,
    },
    Replay {
        account_id: WhatsAppAccountId,
        after_sequence: u64,
        limit: u32,
    },
    PendingCommands {
        account_id: WhatsAppAccountId,
        limit: u32,
    },
    Events {
        account_id: WhatsAppAccountId,
        kind: WhatsAppProviderEventKind,
        provider_chat_id: Option<String>,
        limit: u32,
    },
    ClaimPendingCommands {
        account_id: WhatsAppAccountId,
        host_claim_id: String,
        lease_seconds: u32,
        limit: u32,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum WhatsAppProviderQueryResponse {
    Account(Option<WhatsAppAccount>),
    RuntimeStatus(WhatsAppRuntimeStatus),
    Messages(Vec<WhatsAppMessage>),
    Dialogs(Vec<WhatsAppDialog>),
    Participants(Vec<WhatsAppParticipant>),
    Realtime(Vec<WhatsAppRealtimeFrame>),
    Commands(Vec<WhatsAppProviderCommand>),
    Events(Vec<WhatsAppProviderEvent>),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum WhatsAppClientRequest {
    Lifecycle(WhatsAppLifecycleRequest),
    HostObservation(host_bridge::WhatsAppHostBridgeEnvelopeV1),
    HostCommandFailed {
        operation_id: WhatsAppOperationId,
        host_claim_id: String,
        reason: String,
    },
    RetryCommand { operation_id: WhatsAppOperationId },
    DeadLetterCommand { operation_id: WhatsAppOperationId, reason: String },
    Command(WhatsAppProviderCommand),
    Query(WhatsAppProviderQuery),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum WhatsAppClientResponse {
    Account(WhatsAppAccount),
    Accepted { operation_id: WhatsAppOperationId },
    ObservationAccepted { provider_event_id: String },
    HostCommandFailureRecorded { operation_id: WhatsAppOperationId },
    CommandLifecycleUpdated { operation_id: WhatsAppOperationId, state: String },
    Query(WhatsAppProviderQueryResponse),
}

pub fn validate_provider_query(query: &WhatsAppProviderQuery) -> Result<(), WhatsAppContractError> {
    let (account_id, limit) = match query {
        WhatsAppProviderQuery::Account { account_id }
        | WhatsAppProviderQuery::RuntimeStatus { account_id } => (account_id, 1),
        WhatsAppProviderQuery::CachedMessages { account_id, limit, .. }
        | WhatsAppProviderQuery::SearchMessages { account_id, limit, .. }
        | WhatsAppProviderQuery::Dialogs { account_id, limit }
        | WhatsAppProviderQuery::Participants { account_id, limit, .. }
        | WhatsAppProviderQuery::Replay { account_id, limit, .. }
        | WhatsAppProviderQuery::PendingCommands { account_id, limit }
        | WhatsAppProviderQuery::Events { account_id, limit, .. }
        | WhatsAppProviderQuery::ClaimPendingCommands { account_id, limit, .. } => (account_id, *limit),
    };
    validate_id(account_id)?;
    if limit == 0 || limit > 500 {
        return Err(WhatsAppContractError::FieldTooLong);
    }
    match query {
        WhatsAppProviderQuery::Account { .. }
        | WhatsAppProviderQuery::RuntimeStatus { .. } => {}
        WhatsAppProviderQuery::CachedMessages { provider_chat_id, .. } => {
            if let Some(chat_id) = provider_chat_id {
                validate_id(chat_id)?;
            }
        }
        WhatsAppProviderQuery::SearchMessages { provider_chat_id, query, .. } => {
            if let Some(chat_id) = provider_chat_id {
                validate_id(chat_id)?;
            }
            validate_text(query)?;
        }
        WhatsAppProviderQuery::Participants { provider_chat_id, .. } => validate_id(provider_chat_id)?,
        WhatsAppProviderQuery::Events { provider_chat_id, .. } => {
            if let Some(chat_id) = provider_chat_id {
                validate_id(chat_id)?;
            }
        }
        WhatsAppProviderQuery::Dialogs { .. }
        | WhatsAppProviderQuery::Replay { .. }
        | WhatsAppProviderQuery::PendingCommands { .. } => {}
        WhatsAppProviderQuery::ClaimPendingCommands { host_claim_id, lease_seconds, .. } => {
            validate_id(host_claim_id)?;
            if *lease_seconds == 0 || *lease_seconds > 3600 {
                return Err(WhatsAppContractError::FieldTooLong);
            }
        }
    }
    Ok(())
}

pub fn provider_query_account_id(query: &WhatsAppProviderQuery) -> &str {
    match query {
        WhatsAppProviderQuery::Account { account_id }
        | WhatsAppProviderQuery::RuntimeStatus { account_id }
        | WhatsAppProviderQuery::CachedMessages { account_id, .. }
        | WhatsAppProviderQuery::SearchMessages { account_id, .. }
        | WhatsAppProviderQuery::Dialogs { account_id, .. }
        | WhatsAppProviderQuery::Participants { account_id, .. }
        | WhatsAppProviderQuery::Replay { account_id, .. }
        | WhatsAppProviderQuery::PendingCommands { account_id, .. }
        | WhatsAppProviderQuery::Events { account_id, .. }
        | WhatsAppProviderQuery::ClaimPendingCommands { account_id, .. } => account_id,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum WhatsAppContractError {
    EmptyField,
    FieldTooLong,
    InvalidText,
    InvalidTimestamp,
    InvalidTransition,
    CredentialLeaseRejected,
}

fn validate_id(value: &str) -> Result<(), WhatsAppContractError> {
    if value.trim().is_empty() {
        return Err(WhatsAppContractError::EmptyField);
    }
    if value.len() > MAX_ID_LEN {
        return Err(WhatsAppContractError::FieldTooLong);
    }
    Ok(())
}

fn validate_text(value: &str) -> Result<(), WhatsAppContractError> {
    validate_id(value)?;
    if value.len() > MAX_TEXT_BYTES || value.chars().any(char::is_control) {
        return Err(WhatsAppContractError::InvalidText);
    }
    Ok(())
}

pub fn validate_account_setup(setup: &WhatsAppAccountSetup) -> Result<(), WhatsAppContractError> {
    validate_id(&setup.account_id)?;
    validate_text(&setup.display_name)?;
    validate_id(&setup.external_account_id)?;
    for binding in &setup.credentials {
        validate_id(&binding.secret_ref)?;
        if binding.revision == 0 {
            return Err(WhatsAppContractError::InvalidTransition);
        }
        if binding.purpose != WhatsAppCredentialPurpose::WebSessionKey {
            return Err(WhatsAppContractError::InvalidTransition);
        }
    }
    Ok(())
}

pub fn validate_provider_command(
    command: &WhatsAppProviderCommand,
) -> Result<(), WhatsAppContractError> {
    validate_id(provider_command_operation_id(command))?;
    validate_id(provider_command_account_id(command))?;
    match command {
        WhatsAppProviderCommand::SendText { provider_chat_id, text, .. }
        | WhatsAppProviderCommand::Reply { provider_chat_id, text, .. }
        | WhatsAppProviderCommand::Edit { provider_chat_id, text, .. } => {
            validate_id(provider_chat_id)?;
            validate_text(text)
        }
        WhatsAppProviderCommand::Forward { provider_chat_id, source_provider_chat_id, source_provider_message_id, .. } => {
            validate_id(provider_chat_id)?;
            validate_id(source_provider_chat_id)?;
            validate_id(source_provider_message_id)
        }
        WhatsAppProviderCommand::Delete { provider_chat_id, provider_message_id, .. }
        | WhatsAppProviderCommand::DownloadMedia { provider_chat_id, provider_message_id, .. } => {
            validate_id(provider_chat_id)?;
            validate_id(provider_message_id)
        }
        WhatsAppProviderCommand::React { provider_chat_id, provider_message_id, emoji, .. }
        | WhatsAppProviderCommand::Unreact { provider_chat_id, provider_message_id, emoji, .. } => {
            validate_id(provider_chat_id)?;
            validate_id(provider_message_id)?;
            validate_text(emoji)
        }
        WhatsAppProviderCommand::SendMedia { provider_chat_id, blob_ref, media_kind, caption, filename, .. } => {
            validate_id(provider_chat_id)?;
            validate_id(blob_ref)?;
            validate_id(media_kind)?;
            if let Some(caption) = caption { validate_text(caption)?; }
            if let Some(filename) = filename { validate_id(filename)?; }
            Ok(())
        }
        WhatsAppProviderCommand::SendVoiceNote { provider_chat_id, attachment_id, blob_ref, content_type, sha256, scan_status, filename, .. } => {
            validate_id(provider_chat_id)?;
            validate_id(attachment_id)?;
            validate_id(blob_ref)?;
            validate_id(content_type)?;
            validate_id(sha256)?;
            validate_id(scan_status)?;
            if let Some(filename) = filename { validate_id(filename)?; }
            Ok(())
        }
        WhatsAppProviderCommand::PublishStatus { text, .. } => validate_text(text),
        WhatsAppProviderCommand::JoinConversation { provider_chat_id, invite_link, .. } => {
            validate_id(provider_chat_id)?;
            validate_id(invite_link)
        }
        WhatsAppProviderCommand::LeaveConversation { provider_chat_id, .. } => validate_id(provider_chat_id),
        WhatsAppProviderCommand::Conversation { provider_chat_id, action, .. } => {
            validate_id(provider_chat_id)?;
            let _ = action;
            Ok(())
        }
    }
}

pub fn validate_event(event: &WhatsAppProviderEvent) -> Result<(), WhatsAppContractError> {
    let timestamp = match event {
        WhatsAppProviderEvent::RuntimeStateChanged { observed_at_unix_seconds, .. }
        | WhatsAppProviderEvent::SessionStateChanged { observed_at_unix_seconds, .. }
        | WhatsAppProviderEvent::CommandResultObserved { observed_at_unix_seconds, .. }
        | WhatsAppProviderEvent::MessageEdited { observed_at_unix_seconds, .. }
        | WhatsAppProviderEvent::MessageDeleted { observed_at_unix_seconds, .. }
        | WhatsAppProviderEvent::ReceiptChanged { observed_at_unix_seconds, .. }
        | WhatsAppProviderEvent::ReactionChanged { observed_at_unix_seconds, .. }
        | WhatsAppProviderEvent::PresenceChanged { observed_at_unix_seconds, .. }
        | WhatsAppProviderEvent::CallObserved { observed_at_unix_seconds, .. }
        | WhatsAppProviderEvent::StatusObserved { observed_at_unix_seconds, .. }
        | WhatsAppProviderEvent::StatusViewObserved { observed_at_unix_seconds, .. }
        | WhatsAppProviderEvent::StatusDeleted { observed_at_unix_seconds, .. } => *observed_at_unix_seconds,
        WhatsAppProviderEvent::MessageObserved(value) => value.occurred_at_unix_seconds,
        WhatsAppProviderEvent::DialogObserved(value) => value.observed_at_unix_seconds,
        WhatsAppProviderEvent::ParticipantObserved(value) => value.observed_at_unix_seconds,
        WhatsAppProviderEvent::MediaObserved(value) => value.observed_at_unix_seconds,
    };
    if timestamp <= 0 { return Err(WhatsAppContractError::InvalidTimestamp); }
    if let WhatsAppProviderEvent::CommandResultObserved {
        account_id,
        operation_id,
        provider_request_id,
        ..
    } = event {
        validate_id(account_id)?;
        validate_id(operation_id)?;
        if provider_request_id.as_deref().is_some_and(|value| value.trim().is_empty()) {
            return Err(WhatsAppContractError::EmptyField);
        }
    }
    if let WhatsAppProviderEvent::SessionStateChanged { linked, secret_ref, revision, .. } = event {
        if *linked != secret_ref.is_some()
            || *linked != revision.is_some()
            || revision.is_some_and(|value| value == 0)
        {
            return Err(WhatsAppContractError::InvalidTransition);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_validation_rejects_empty_provider_payload() {
        let command = WhatsAppProviderCommand::SendText {
            operation_id: "op".to_owned(),
            account_id: "account".to_owned(),
            provider_chat_id: "chat".to_owned(),
            text: "".to_owned(),
        };
        assert_eq!(validate_provider_command(&command), Err(WhatsAppContractError::EmptyField));
    }

    #[test]
    fn media_command_requires_opaque_blob_reference() {
        let command = WhatsAppProviderCommand::SendMedia {
            operation_id: "op".to_owned(),
            account_id: "account".to_owned(),
            provider_chat_id: "chat".to_owned(),
            blob_ref: "".to_owned(),
            media_kind: "image".to_owned(),
            caption: None,
            filename: None,
        };
        assert_eq!(validate_provider_command(&command), Err(WhatsAppContractError::EmptyField));
    }
}
