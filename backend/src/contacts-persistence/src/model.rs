use hermes_contacts_core::{ContactUpsertDraftV1, ContactUpsertOutcomeV1};
use sha2::{Digest, Sha256};

pub const CONTACTS_MAX_EVENT_BYTES_V1: usize = 64 * 1024;
pub const CONTACTS_OUTBOX_LIMIT_V1: u16 = 128;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContactsOutboxRecordV1 {
    pub message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub envelope_bytes: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplyMailEntryCommandV1 {
    pub command_message_id: [u8; 16],
    pub command_envelope_sha256: [u8; 32],
    pub command_id: [u8; 16],
    pub draft: ContactUpsertDraftV1,
    pub received_at_unix_millis: i64,
    pub completed_at_unix_millis: i64,
}

impl ApplyMailEntryCommandV1 {
    #[must_use]
    pub fn command_fingerprint(&self) -> [u8; 32] {
        command_fingerprint(
            self.command_envelope_sha256,
            self.command_id,
            self.draft.provenance.entry_digest,
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i16)]
pub enum ContactMailEntryRejectCodeV1 {
    InvalidRequest = 1,
    IdentityAmbiguous = 2,
    ProviderLinkConflict = 3,
    StaleSource = 4,
    Policy = 5,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RejectMailEntryCommandV1 {
    pub command_message_id: [u8; 16],
    pub command_envelope_sha256: [u8; 32],
    pub command_id: [u8; 16],
    pub logical_owner_id: String,
    pub entry_digest: [u8; 32],
    pub received_at_unix_millis: i64,
    pub completed_at_unix_millis: i64,
    pub code: ContactMailEntryRejectCodeV1,
    pub terminal_result: ContactsOutboxRecordV1,
}

impl RejectMailEntryCommandV1 {
    #[must_use]
    pub fn command_fingerprint(&self) -> [u8; 32] {
        command_fingerprint(
            self.command_envelope_sha256,
            self.command_id,
            self.entry_digest,
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RejectedMailEntryCommandV1 {
    pub code: ContactMailEntryRejectCodeV1,
    pub terminal_result: ContactsOutboxRecordV1,
    pub replayed: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppliedMailEntryCommandV1 {
    pub contact_id: [u8; 16],
    pub contact_revision: u64,
    pub outcome: ContactUpsertOutcomeV1,
    pub terminal_result: ContactsOutboxRecordV1,
    pub replayed: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContactsPersistenceErrorV1 {
    InvalidInput,
    InvalidRow,
    StorageUnavailable,
    CommandConflict,
    InboxConflict,
    IdentityAmbiguous,
    ProviderLinkConflict,
    StaleSource,
    PolicyRejected,
    NotFound,
}

pub(crate) fn valid_apply(value: &ApplyMailEntryCommandV1) -> bool {
    nonzero(&value.command_message_id)
        && nonzero(&value.command_envelope_sha256)
        && nonzero(&value.command_id)
        && hermes_contacts_core::upsert_fingerprint_v1(&value.draft).is_ok()
        && value.received_at_unix_millis > 0
        && value.completed_at_unix_millis >= value.received_at_unix_millis
}

pub(crate) fn valid_reject(value: &RejectMailEntryCommandV1) -> bool {
    nonzero(&value.command_message_id)
        && nonzero(&value.command_envelope_sha256)
        && nonzero(&value.command_id)
        && valid_owner(&value.logical_owner_id)
        && nonzero(&value.entry_digest)
        && value.received_at_unix_millis > 0
        && value.completed_at_unix_millis >= value.received_at_unix_millis
        && valid_outbox(&value.terminal_result)
}

pub(crate) fn valid_outbox(value: &ContactsOutboxRecordV1) -> bool {
    nonzero(&value.message_id)
        && nonzero(&value.envelope_sha256)
        && !value.envelope_bytes.is_empty()
        && value.envelope_bytes.len() <= CONTACTS_MAX_EVENT_BYTES_V1
        && Sha256::digest(&value.envelope_bytes).as_slice() == value.envelope_sha256
}

pub(crate) fn valid_owner(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-' | b'.')
        })
}

pub(crate) fn nonzero<const N: usize>(value: &[u8; N]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

fn command_fingerprint(
    command_envelope_sha256: [u8; 32],
    command_id: [u8; 16],
    entry_digest: [u8; 32],
) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"hermes.contacts.mail-entry.command.v1\0");
    hash.update(command_envelope_sha256);
    hash.update(command_id);
    hash.update(entry_digest);
    hash.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use hermes_contacts_core::{
        ContactProviderKindV1, ContactProviderProvenanceV1, ContactTimestampV1,
    };

    #[test]
    fn fingerprint_binds_command_and_canonical_input() {
        let mut input = sample();
        let first = input.command_fingerprint();
        input.command_envelope_sha256[0] ^= 1;
        assert_ne!(first, input.command_fingerprint());
    }

    fn sample() -> ApplyMailEntryCommandV1 {
        ApplyMailEntryCommandV1 {
            command_message_id: [1; 16],
            command_envelope_sha256: [2; 32],
            command_id: [3; 16],
            draft: ContactUpsertDraftV1 {
                logical_owner_id: "owner-1".to_owned(),
                display_name: "Ada".to_owned(),
                email_addresses: vec!["ada@example.test".to_owned()],
                phone_numbers: Vec::new(),
                provenance: ContactProviderProvenanceV1 {
                    source_account_id: "mail-1".to_owned(),
                    provider_kind: ContactProviderKindV1::Gmail,
                    provider_entry_id: "people/c1".to_owned(),
                    provider_etag: Some("etag-1".to_owned()),
                    source_revision: 1,
                    entry_digest: [4; 32],
                    observed_at: ContactTimestampV1 {
                        unix_seconds: 1_800_000_000,
                        nanos: 0,
                    },
                },
            },
            received_at_unix_millis: 1_800_000_000_000,
            completed_at_unix_millis: 1_800_000_000_001,
        }
    }
}
