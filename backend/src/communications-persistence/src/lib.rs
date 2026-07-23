//! Owner-local idempotency seam for canonical Communications evidence.

pub const PACKAGE: &str = "hermes-communications-persistence";

mod durable;
mod search;
mod search_job;
mod schema;
pub use durable::CommunicationsDurablePersistence;
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
    MissingCanonicalMessage,
    StorageUnavailable,
    InvalidRow,
}
