//! Inbox deduplication and same-ID hash-conflict classification.

use crate::delivery::OutboxRecordV1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InboxDecisionV1 {
    Accept,
    Duplicate,
    HashConflict,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InboxRecordV1 {
    message_id: [u8; 16],
    envelope_sha256: [u8; 32],
}

impl InboxRecordV1 {
    #[must_use]
    pub fn from_outbox(record: &OutboxRecordV1) -> Self {
        Self {
            message_id: *record.message_id(),
            envelope_sha256: *record.envelope_sha256(),
        }
    }

    #[must_use]
    pub const fn message_id(&self) -> &[u8; 16] {
        &self.message_id
    }

    #[must_use]
    pub const fn envelope_sha256(&self) -> &[u8; 32] {
        &self.envelope_sha256
    }

    #[must_use]
    pub fn classify(&self, candidate: &OutboxRecordV1) -> InboxDecisionV1 {
        if self.message_id != *candidate.message_id() {
            return InboxDecisionV1::Accept;
        }
        if self.envelope_sha256 == *candidate.envelope_sha256() {
            InboxDecisionV1::Duplicate
        } else {
            InboxDecisionV1::HashConflict
        }
    }
}
