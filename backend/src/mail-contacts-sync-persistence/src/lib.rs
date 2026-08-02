#![forbid(unsafe_code)]

#[cfg(feature = "conformance-test-support")]
mod conformance;
mod model;
mod orchestration;
mod realtime;
mod relay;
mod repository;
mod schema;

#[cfg(feature = "conformance-test-support")]
pub use conformance::MailContactsSyncPersistenceConformanceV1;
pub use model::{
    CreateMailContactsSyncOutcomeV1, CreateMailContactsSyncRunV1, MailContactsSyncContactOutcomeV1,
    MailContactsSyncEntryInputV1, MailContactsSyncEntryOutcomeInputV1,
    MailContactsSyncInboxOutcomeV1, MailContactsSyncPageProgressV1,
    MailContactsSyncPageResultInputV1, MailContactsSyncPersistenceErrorV1,
    MailContactsSyncPersistenceOutcomeV1, MailContactsSyncRealtimeTransitionV1,
    MailContactsSyncTransitionInputV1, OutboxEnvelopeV1, PersistedMailContactsSyncRunV1,
};
pub use repository::MailContactsSyncPersistenceV1;
pub use schema::{
    MAIL_CONTACTS_SYNC_ORCHESTRATION_SCHEMA_V1, MAIL_CONTACTS_SYNC_SCHEMA_V1,
    MAIL_CONTACTS_SYNC_STORAGE_BUNDLE_REVISION_V1, mail_contacts_sync_storage_bundle_v1,
};

pub const PACKAGE: &str = "hermes-mail-contacts-sync-persistence";
