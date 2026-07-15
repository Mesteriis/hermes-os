use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::accounts::CommunicationProviderKind;
use crate::email_sync::EmailSyncBatch;

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

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum MailProviderResourceParseError {
    #[error("invalid stored mail provider resource kind: {0}")]
    InvalidResourceKind(String),
    #[error("invalid stored mail provider semantic role: {0}")]
    InvalidSemanticRole(String),
}

impl TryFrom<&str> for MailProviderResourceKind {
    type Error = MailProviderResourceParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "folder" => Ok(Self::Folder),
            "label" => Ok(Self::Label),
            other => Err(MailProviderResourceParseError::InvalidResourceKind(
                other.to_owned(),
            )),
        }
    }
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

impl TryFrom<&str> for MailProviderSemanticRole {
    type Error = MailProviderResourceParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "inbox" => Ok(Self::Inbox),
            "sent" => Ok(Self::Sent),
            "drafts" => Ok(Self::Drafts),
            "archive" => Ok(Self::Archive),
            "trash" => Ok(Self::Trash),
            "junk" => Ok(Self::Junk),
            "all" => Ok(Self::All),
            "flagged" => Ok(Self::Flagged),
            "important" => Ok(Self::Important),
            "user" => Ok(Self::User),
            other => Err(MailProviderResourceParseError::InvalidSemanticRole(
                other.to_owned(),
            )),
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

#[derive(Debug, thiserror::Error)]
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

#[derive(Clone, Debug, thiserror::Error, Eq, PartialEq)]
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
