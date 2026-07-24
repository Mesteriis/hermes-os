//! Mail integration contract crate for ADR-0239.

pub const PACKAGE: &str = "hermes-mail-api";

pub mod wire {
    include!(concat!(env!("OUT_DIR"), "/hermes.mail.v1.rs"));
}
pub mod client_wire;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MailClientRequestV1 {
    SyncInbox(MailSyncInboxRequestV1),
    SendMail(MailSendMailRequestV1),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailSyncInboxRequestV1 {
    pub operation_id: String,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailSendMailRequestV1 {
    pub operation_id: String,
    pub provider_conversation_id: String,
    pub recipients: Vec<String>,
    pub subject: String,
    pub text_body: String,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MailClientResponseV1 {
    SyncInboxCompleted {
        operation_id: String,
        observed_messages: u32,
    },
    MailAccepted {
        operation_id: String,
        response_code: u16,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailAccountConfigurationV1 {
    pub connection_id: String,
    pub inbound: MailInboundTransportV1,
    pub sync_window: u32,
    pub sync_windows: u32,
    pub smtp_endpoint: Option<SmtpEndpointV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MailInboundTransportV1 {
    Imap(MailImapConfigurationV1),
    Gmail(MailGmailConfigurationV1),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailImapConfigurationV1 {
    pub host: String,
    pub port: u16,
    pub username: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailGmailConfigurationV1 {
    pub user_id: String,
    pub from_address: String,
}

pub fn valid_account_configuration(configuration: &MailAccountConfigurationV1) -> bool {
    !configuration.connection_id.trim().is_empty()
        && valid_inbound_transport(&configuration.inbound)
        && valid_window(configuration.sync_window, configuration.sync_windows)
        && (!matches!(configuration.inbound, MailInboundTransportV1::Gmail(_))
            || configuration.smtp_endpoint.is_none())
        && configuration.smtp_endpoint.as_ref().is_none_or(|endpoint| {
            valid_host(&endpoint.host)
                && valid_smtp_port(endpoint.port)
                && !endpoint.username.trim().is_empty()
                && !endpoint.from_address.trim().is_empty()
        })
}

pub fn valid_inbound_transport(transport: &MailInboundTransportV1) -> bool {
    match transport {
        MailInboundTransportV1::Imap(configuration) => {
            valid_host(&configuration.host)
                && valid_port(configuration.port)
                && !configuration.username.trim().is_empty()
        }
        MailInboundTransportV1::Gmail(configuration) => {
            valid_gmail_user_id(&configuration.user_id)
                && valid_mailbox(&configuration.from_address)
        }
    }
}

pub fn valid_gmail_user_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 512
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'@'))
}

pub fn valid_mailbox(value: &str) -> bool {
    value.is_ascii()
        && !value.is_empty()
        && value.len() <= 320
        && !value.contains(char::is_whitespace)
        && !value.contains(['\r', '\n', '\0', '<', '>', '"'])
        && value
            .split_once('@')
            .is_some_and(|(local, domain)| !local.is_empty() && valid_host(domain))
}

/// Mail/IMAP slice admits only these explicit statuses.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailConnectionState {
    Provisioning,
    Ready,
    Syncing,
    Degraded,
    Retired,
}

/// Limited contract errors exposed by mail runtime boundaries.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailContractError {
    InvalidHost,
    InvalidPort,
    InvalidOperation,
    InvalidPayload,
    WindowLimitExceeded,
}

/// Global constraints for the current slice.
pub const IMAP_PORT: u16 = 993;
pub const SMTP_IMPLICIT_TLS_PORT: u16 = 465;
pub const MAX_HOST_LEN: usize = 253;
pub const MAX_MESSAGE_BYTES: usize = 1024 * 1024;
pub const MAX_PLAIN_TEXT_BYTES: usize = 256 * 1024;
pub const DEFAULT_WINDOW: u32 = 5_000;
pub const MAX_WINDOW: u32 = 1_000_000;
pub const MAX_WINDOWS: u32 = 1_000_000;
pub const SYNC_DEADLINE_SECONDS: u64 = 300;
pub const WINDOW_DEADLINE_SECONDS: u64 = 10;

pub type MailConnectionId = String;
pub type MailOperationId = String;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailCredentialPurpose {
    ImapPassword,
    GmailAccessToken,
    SmtpPassword,
}

impl MailCredentialPurpose {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ImapPassword => "mail_imap_password",
            Self::GmailAccessToken => "mail_gmail_access_token",
            Self::SmtpPassword => "mail_smtp_password",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailCredentialBinding {
    pub purpose: MailCredentialPurpose,
    pub secret_ref: String,
    pub revision: u64,
}

#[derive(Clone, Debug)]
pub struct BeginImapConnection {
    pub connection_id: MailConnectionId,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub use_tls: bool,
}

#[derive(Clone, Debug)]
pub struct CompleteImapConnection {
    pub connection_id: MailConnectionId,
    pub operation_id: MailOperationId,
}

#[derive(Clone, Debug)]
pub struct SyncNow {
    pub connection_id: MailConnectionId,
    pub operation_id: MailOperationId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SmtpEndpointV1 {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub from_address: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutgoingMailV1 {
    pub operation_id: MailOperationId,
    pub connection_id: MailConnectionId,
    pub provider_conversation_id: String,
    pub recipients: Vec<String>,
    pub subject: String,
    pub text_body: String,
}

#[derive(Clone, Debug)]
pub struct GetConnection {
    pub connection_id: MailConnectionId,
}

#[derive(Clone, Debug)]
pub struct GetSyncStatus {
    pub connection_id: MailConnectionId,
}

#[derive(Clone, Debug)]
pub struct GetOperationStatus {
    pub operation_id: MailOperationId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MailOperation {
    pub operation_id: MailOperationId,
    pub state: MailConnectionState,
    pub window_size: u32,
}

#[derive(Clone, Debug)]
pub struct MailConnection {
    pub id: MailConnectionId,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub state: MailConnectionState,
    pub operation_id: Option<MailOperationId>,
}

pub fn valid_host(host: &str) -> bool {
    if host.trim().is_empty() {
        return false;
    }
    if host.len() > MAX_HOST_LEN {
        return false;
    }
    if host == "localhost" {
        return true;
    }
    let parts: Vec<&str> = host.split('.').collect();
    if parts.len() < 2 {
        return false;
    }
    parts.iter().all(|part| {
        (!part.is_empty())
            && part.len() <= 63
            && part
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    })
}

pub fn valid_port(port: u16) -> bool {
    port == IMAP_PORT && port > 0
}

pub fn valid_smtp_port(port: u16) -> bool {
    port == SMTP_IMPLICIT_TLS_PORT
}

pub fn valid_window(window: u32, windows: u32) -> bool {
    window > 0 && window <= MAX_WINDOW && windows > 0 && windows <= MAX_WINDOWS
}

pub fn valid_message_bytes(bytes: usize) -> bool {
    bytes <= MAX_MESSAGE_BYTES
}

pub fn valid_plain_text_bytes(bytes: usize) -> bool {
    bytes <= MAX_PLAIN_TEXT_BYTES
}
