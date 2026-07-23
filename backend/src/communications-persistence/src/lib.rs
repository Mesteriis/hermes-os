//! Owner-local idempotency seam for canonical Communications evidence.

pub const PACKAGE: &str = "hermes-communications-persistence";

use hermes_communications_api::CommunicationObservationIdV1;

mod durable;
mod custody_transfer;
mod search;
mod search_job;
mod schema;
pub use durable::CommunicationsDurablePersistence;
pub use custody_transfer::{
    ClaimedCommunicationsBodyCustodyTransferV1, CommunicationsBodyCustodyTransferErrorV1,
};
pub use search::{
    CommunicationsSearchProjectionWriteV1, CommunicationsSearchProjectionWriteErrorV1,
};
pub use search_job::{
    ClaimedCommunicationsDerivedIndexJobV1, CommunicationsDerivedIndexFailureRecordV1,
    CommunicationsDerivedIndexFailureV1,
    CommunicationsDerivedIndexJobErrorV1, CommunicationsDerivedIndexJobOperationV1,
    CommunicationsDerivedIndexJobV1, communications_derived_index_job_id_v1,
};
pub use schema::{
    COMMUNICATIONS_SCHEMA_V1, COMMUNICATIONS_STORAGE_BUNDLE_REVISION_V1,
    communications_storage_bundle_v1,
};

/// Private Communications-owned work item for an admitted producer body. It
/// never becomes a canonical Blob reference or public query field.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingCommunicationsBodyCustodyTransferV1 {
    pub evidence_id: CommunicationObservationIdV1,
    pub envelope_sha256: [u8; 32],
    pub source_blob_ref: String,
    pub source_reference_id: [u8; 16],
    pub declared_bytes: u64,
    pub plaintext_sha256: [u8; 32],
    pub source_custody_proof: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsConsumeOutcomeV1 {
    Applied,
    Duplicate,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsPersistenceError {
    DuplicateOperation,
    InboxHashConflict,
    InvalidDerivedIndexJob,
    InvalidCustodyTransfer,
    MissingCanonicalMessage,
    StorageUnavailable,
    InvalidRow,
}
