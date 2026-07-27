//! Mail-owned PostgreSQL persistence for delivery state and Communications outbox.

mod account;
mod attachments;
#[cfg(feature = "conformance-test-support")]
mod conformance;
mod durable;
mod lifecycle;
mod oauth;
mod schema;

pub use account::{MAIL_SCHEMA_V7, MailCredentialBindingV1};
pub use attachments::{
    MAIL_SCHEMA_V5, MailAttachmentDispositionV1, MailAttachmentMaterializationV1,
    MailAttachmentSafetyStateV1, MailAttachmentSafetyTransitionV1,
    MailDeliveryAttachmentManifestV1,
};
#[cfg(feature = "conformance-test-support")]
pub use conformance::MailPersistenceConformanceV1;
pub use durable::{
    MAIL_SCHEMA_V1, MAIL_SCHEMA_V2, MAIL_SCHEMA_V3, MAIL_SCHEMA_V6,
    MailAttachmentAnchorMappingOutcomeV1, MailAttachmentAnchorMappingV1,
    MailAttachmentBlobAdmissionCompletionV1, MailAttachmentBlobAdmissionStartOutcomeV1,
    MailDeliveryAttemptOutcomeV1, MailDeliveryAttemptV1, MailDeliveryEnqueueOutcomeV1,
    MailDeliveryEnqueueRequestV1, MailDurablePersistence, MailDurablePersistenceError,
    MailQueuedDeliveryV1, MailSmtpDeliveryAttemptStateV1,
};
pub use lifecycle::{MAIL_SCHEMA_V8, MailAccountLifecycleBeginV1};
pub use oauth::{
    GmailOAuthAttemptStartV1, GmailOAuthCredentialBindingV1, GmailOAuthEnqueueOutcomeV1,
    GmailOAuthOperationKindV1, GmailOAuthOperationOutcomeV1, GmailOAuthOperationV1,
    GmailOAuthQueuedOperationV1, GmailOAuthStoredAttemptV1, MAIL_SCHEMA_V4,
};
pub use schema::{
    MAIL_STORAGE_BUNDLE_REVISION_V1, MAIL_STORAGE_BUNDLE_REVISION_V2,
    MAIL_STORAGE_BUNDLE_REVISION_V3, MAIL_STORAGE_BUNDLE_REVISION_V4,
    MAIL_STORAGE_BUNDLE_REVISION_V5, MAIL_STORAGE_BUNDLE_REVISION_V6,
    MAIL_STORAGE_BUNDLE_REVISION_V7, MAIL_STORAGE_BUNDLE_REVISION_V8, mail_storage_bundle_v1,
};

pub const PACKAGE: &str = "hermes-mail-persistence";
