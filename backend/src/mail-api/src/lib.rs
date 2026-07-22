//! Mail integration contract crate for ADR-0239.

pub const PACKAGE: &str = "hermes-mail-api";

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

pub fn valid_window(window: u32, windows: u32) -> bool {
    window > 0 && window <= MAX_WINDOW && windows > 0 && windows <= MAX_WINDOWS
}

pub fn valid_message_bytes(bytes: usize) -> bool {
    bytes <= MAX_MESSAGE_BYTES
}

pub fn valid_plain_text_bytes(bytes: usize) -> bool {
    bytes <= MAX_PLAIN_TEXT_BYTES
}
