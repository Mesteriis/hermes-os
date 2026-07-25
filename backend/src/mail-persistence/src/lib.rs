//! Mail-owned PostgreSQL persistence for delivery state and Communications outbox.

#[cfg(feature = "conformance-test-support")]
mod conformance;
mod durable;
mod oauth;
mod schema;

#[cfg(feature = "conformance-test-support")]
pub use conformance::MailPersistenceConformanceV1;
pub use durable::{
    MAIL_SCHEMA_V1, MAIL_SCHEMA_V2, MAIL_SCHEMA_V3, MailAttachmentAnchorMappingOutcomeV1,
    MailAttachmentAnchorMappingV1, MailAttachmentBlobAdmissionStartOutcomeV1,
    MailDeliveryAttemptOutcomeV1, MailDeliveryAttemptV1, MailDeliveryEnqueueOutcomeV1,
    MailDurablePersistence, MailDurablePersistenceError, MailQueuedDeliveryV1,
    MailSmtpDeliveryAttemptStateV1,
};
pub use oauth::{
    GmailOAuthAttemptStartV1, GmailOAuthCredentialBindingV1, GmailOAuthEnqueueOutcomeV1,
    GmailOAuthOperationKindV1, GmailOAuthOperationOutcomeV1, GmailOAuthOperationV1,
    GmailOAuthQueuedOperationV1, GmailOAuthStoredAttemptV1, MAIL_SCHEMA_V4,
};
pub use schema::{
    MAIL_STORAGE_BUNDLE_REVISION_V1, MAIL_STORAGE_BUNDLE_REVISION_V2,
    MAIL_STORAGE_BUNDLE_REVISION_V3, MAIL_STORAGE_BUNDLE_REVISION_V4, mail_storage_bundle_v1,
};

pub const PACKAGE: &str = "hermes-mail-persistence";
