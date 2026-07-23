//! Owner-local idempotency seam for canonical Communications evidence.

pub const PACKAGE: &str = "hermes-communications-persistence";

mod durable;
mod schema;
pub use durable::CommunicationsDurablePersistence;
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
    MissingCanonicalMessage,
    StorageUnavailable,
    InvalidRow,
}
