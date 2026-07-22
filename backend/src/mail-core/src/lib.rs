//! Mail integration implementation helpers for ADR-0239.

use std::collections::HashMap;

use hermes_communications_ingress::CommunicationObservationDraft;
use hermes_mail_api::{
    DEFAULT_WINDOW, MAX_PLAIN_TEXT_BYTES, MAX_WINDOWS, MailContractError::WindowLimitExceeded,
    SYNC_DEADLINE_SECONDS, WINDOW_DEADLINE_SECONDS, valid_host, valid_message_bytes, valid_port,
    valid_window,
};

pub use hermes_mail_api::{
    MailConnection, MailConnectionId, MailConnectionState, MailContractError, MailOperation,
    MailOperationId,
};

pub const PACKAGE: &str = "hermes-mail-core";

#[derive(Clone, Debug)]
pub struct SyncPlan {
    pub window: u32,
    pub windows: u32,
    pub total_deadline_seconds: u64,
    pub window_deadline_seconds: u64,
}

impl SyncPlan {
    pub const fn default() -> Self {
        Self {
            window: DEFAULT_WINDOW,
            windows: 1,
            total_deadline_seconds: SYNC_DEADLINE_SECONDS,
            window_deadline_seconds: WINDOW_DEADLINE_SECONDS,
        }
    }

    pub fn bounded(window: u32, windows: u32) -> Option<Self> {
        valid_window(window, windows).then_some(Self {
            window,
            windows,
            total_deadline_seconds: SYNC_DEADLINE_SECONDS,
            window_deadline_seconds: WINDOW_DEADLINE_SECONDS,
        })
    }
}

#[derive(Clone, Debug)]
pub struct MailStatePolicy {
    pub max_sync_windows: u32,
}

impl MailStatePolicy {
    pub const fn new() -> Self {
        Self {
            max_sync_windows: MAX_WINDOWS,
        }
    }
}

impl Default for MailStatePolicy {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Default)]
pub struct ConnectionTracker {
    operations: HashMap<MailOperationId, MailOperation>,
    status: HashMap<MailConnectionId, MailConnectionState>,
}

impl ConnectionTracker {
    pub fn new() -> Self {
        Self {
            operations: HashMap::new(),
            status: HashMap::new(),
        }
    }

    pub fn register_connection(&mut self, connection: &MailConnection) {
        self.status
            .insert(connection.id.clone(), MailConnectionState::Provisioning);
    }

    pub fn set_ready(&mut self, connection_id: &str) {
        self.status
            .insert(connection_id.to_string(), MailConnectionState::Ready);
    }

    pub fn set_syncing(&mut self, connection_id: &str, operation: MailOperation) {
        self.status
            .insert(connection_id.to_string(), MailConnectionState::Syncing);
        self.operations
            .insert(operation.operation_id.clone(), operation);
    }

    pub fn set_degraded(&mut self, connection_id: &str) {
        self.status
            .insert(connection_id.to_string(), MailConnectionState::Degraded);
    }

    pub fn set_retired(&mut self, connection_id: &str) {
        self.status
            .insert(connection_id.to_string(), MailConnectionState::Retired);
    }

    pub fn operation_status(&self, operation_id: &str) -> Option<&MailOperation> {
        self.operations.get(operation_id)
    }

    pub fn status_of(&self, connection_id: &str) -> Option<MailConnectionState> {
        self.status.get(connection_id).copied()
    }
}

pub fn validate_sync_request(
    host: &str,
    port: u16,
    body_bytes: usize,
) -> Result<(), MailContractError> {
    if !valid_host(host) {
        return Err(MailContractError::InvalidHost);
    }
    if !valid_port(port) {
        return Err(MailContractError::InvalidPort);
    }
    if body_bytes > MAX_PLAIN_TEXT_BYTES {
        return Err(MailContractError::InvalidPayload);
    }
    if !valid_message_bytes(body_bytes) {
        return Err(MailContractError::InvalidPayload);
    }
    Ok(())
}

pub fn draft_ingress_observation(
    operation_id: &str,
    source_kind: impl Into<String>,
    source_id: impl Into<String>,
    body_bytes: usize,
    preview: Option<String>,
) -> Result<CommunicationObservationDraft, MailContractError> {
    if body_bytes > MAX_PLAIN_TEXT_BYTES {
        return Err(MailContractError::InvalidPayload);
    }
    let text_preview = preview.filter(|value| !value.trim().is_empty());
    Ok(CommunicationObservationDraft {
        operation_id: operation_id.to_string(),
        source_id: source_id.into(),
        source_kind: source_kind.into(),
        text_preview,
        has_body: body_bytes > 0,
        is_final_window: true,
    })
}

pub fn bounded_window(window: u32, windows: u32) -> Result<SyncPlan, MailContractError> {
    SyncPlan::bounded(window, windows).ok_or(WindowLimitExceeded)
}

pub mod constants {
    pub use hermes_mail_api::MAX_WINDOWS;
}
