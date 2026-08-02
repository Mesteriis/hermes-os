use hermes_mail_contacts_sync_core::{
    MailContactsSyncDirectionV1, MailContactsSyncDraftV1, MailContactsSyncStatusV1,
    MailContactsSyncTransitionV1,
};
use sha2::{Digest, Sha256};

pub const MAX_ENVELOPE_BYTES_V1: usize = 64 * 1024;
pub const MAIL_CONTACTS_SYNC_OUTBOX_LIMIT_V1: u16 = 256;
pub const MAIL_CONTACTS_SYNC_REALTIME_LIMIT_V1: u16 = 256;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutboxEnvelopeV1 {
    pub message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub envelope_bytes: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MailContactsSyncRealtimeTransitionV1 {
    pub sequence: u64,
    pub run_id: [u8; 16],
    pub state: hermes_mail_contacts_sync_core::MailContactsSyncStateV1,
    pub state_revision: u64,
    pub rejection: Option<hermes_mail_contacts_sync_core::MailContactsSyncRejectCodeV1>,
    pub occurred_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreateMailContactsSyncRunV1 {
    pub logical_owner_id: String,
    pub draft: MailContactsSyncDraftV1,
    pub initial_command: OutboxEnvelopeV1,
    pub created_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailContactsSyncTransitionInputV1 {
    pub logical_owner_id: String,
    pub run_id: [u8; 16],
    pub direction: MailContactsSyncDirectionV1,
    pub message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub transition: MailContactsSyncTransitionV1,
    pub next_command: Option<OutboxEnvelopeV1>,
    pub occurred_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistedMailContactsSyncRunV1 {
    pub logical_owner_id: String,
    pub draft: MailContactsSyncDraftV1,
    pub request_fingerprint: [u8; 32],
    pub status: MailContactsSyncStatusV1,
    pub created_at_unix_millis: i64,
    pub updated_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CreateMailContactsSyncOutcomeV1 {
    Created(PersistedMailContactsSyncRunV1),
    Existing(PersistedMailContactsSyncRunV1),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MailContactsSyncInboxOutcomeV1 {
    Applied(PersistedMailContactsSyncRunV1),
    Duplicate(PersistedMailContactsSyncRunV1),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailContactsSyncPersistenceErrorV1 {
    InvalidInput,
    InvalidRow,
    StorageUnavailable,
    RequestConflict,
    InboxConflict,
    RevisionConflict,
    InvalidTransition,
    NotFound,
}

pub(crate) fn request_fingerprint(draft: &MailContactsSyncDraftV1) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"hermes.mail_contacts_sync.start.v1\0");
    hash.update(draft.account_id.as_bytes());
    hash.update([direction_code(draft.direction) as u8]);
    hash.update([trigger_code(draft.trigger) as u8]);
    hash.finalize().into()
}

pub(crate) const fn direction_code(value: MailContactsSyncDirectionV1) -> i16 {
    match value {
        MailContactsSyncDirectionV1::ProviderToContacts => 1,
        MailContactsSyncDirectionV1::Bidirectional => 2,
    }
}

pub(crate) const fn trigger_code(
    value: hermes_mail_contacts_sync_core::MailContactsSyncTriggerV1,
) -> i16 {
    match value {
        hermes_mail_contacts_sync_core::MailContactsSyncTriggerV1::Manual => 1,
        hermes_mail_contacts_sync_core::MailContactsSyncTriggerV1::Scheduled => 2,
    }
}

pub(crate) fn valid_identity(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
}

pub(crate) fn valid_envelope(value: &OutboxEnvelopeV1) -> bool {
    nonzero(&value.message_id)
        && nonzero(&value.envelope_sha256)
        && !value.envelope_bytes.is_empty()
        && value.envelope_bytes.len() <= MAX_ENVELOPE_BYTES_V1
        && Sha256::digest(&value.envelope_bytes).as_slice() == value.envelope_sha256
}

pub(crate) fn nonzero<const N: usize>(value: &[u8; N]) -> bool {
    value.iter().any(|byte| *byte != 0)
}
