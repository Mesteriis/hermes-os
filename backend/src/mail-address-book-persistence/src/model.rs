#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailAddressBookUpsertAdmissionV1 {
    pub command_message_id: [u8; 16],
    pub command_envelope_sha256: [u8; 32],
    pub command_id: [u8; 16],
    pub run_id: [u8; 16],
    pub logical_owner_id: String,
    pub account_id: String,
    pub contact_snapshot_reference_id: [u8; 16],
    pub contact_snapshot_sha256: [u8; 32],
    pub expected_contact_revision: u64,
    pub contact_snapshot_declared_bytes: u64,
    pub contact_snapshot_custody_source_proof: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingMailAddressBookUpsertV1 {
    pub admission: MailAddressBookUpsertAdmissionV1,
    pub target_snapshot_receipt: Option<MailAddressBookTargetSnapshotReceiptV1>,
    pub execution_attempt: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MailAddressBookTargetSnapshotReceiptV1 {
    pub reference_id: [u8; 16],
    pub receipt_sha256: [u8; 32],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailAddressBookFetchAdmissionV1 {
    pub command_message_id: [u8; 16],
    pub command_envelope_sha256: [u8; 32],
    pub command_id: [u8; 16],
    pub run_id: [u8; 16],
    pub logical_owner_id: String,
    pub account_id: String,
    pub page_sequence: u64,
    pub continuation_cursor: Option<Vec<u8>>,
    pub page_size: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingMailAddressBookFetchV1 {
    pub admission: MailAddressBookFetchAdmissionV1,
    pub execution_attempt: u32,
}
