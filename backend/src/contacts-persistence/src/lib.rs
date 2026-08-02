#![forbid(unsafe_code)]

mod model;
mod repository;
mod schema;

#[cfg(feature = "conformance-test-support")]
mod conformance;

#[cfg(feature = "conformance-test-support")]
pub use conformance::ContactsPersistenceConformanceV1;

pub use model::{
    AppliedMailEntryCommandV1, ApplyMailEntryCommandV1, ContactsOutboxRecordV1,
    ContactsPersistenceErrorV1,
};
pub use repository::ContactsPersistenceV1;
pub use schema::{
    CONTACTS_SCHEMA_V1, CONTACTS_STORAGE_BUNDLE_REVISION_V1, contacts_storage_bundle_v1,
};

pub const PACKAGE: &str = "hermes-contacts-persistence";
