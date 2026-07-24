//! Mail-owned PostgreSQL persistence for delivery state and Communications outbox.

mod durable;
mod schema;

pub use durable::{
    MAIL_SCHEMA_V1, MailAttachmentAnchorMappingOutcomeV1, MailAttachmentAnchorMappingV1,
    MailAttachmentBlobAdmissionStartOutcomeV1, MailDurablePersistence, MailDurablePersistenceError,
    MailSmtpDeliveryAttemptStateV1,
};
pub use schema::{MAIL_STORAGE_BUNDLE_REVISION_V1, mail_storage_bundle_v1};

pub const PACKAGE: &str = "hermes-mail-persistence";
