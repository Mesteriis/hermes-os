#![forbid(unsafe_code)]

mod delivery;
mod repository;
mod schema;

pub use delivery::{
    MailReplayCommandAdmissionV1, MailReplayCommandInboxOutcomeV1, MailReplayResultStoreOutcomeV1,
};
pub use repository::{
    MailRetainedEvidenceReplayPersistenceV1, RetainedMailEvidenceV1, RetainedMailReplayAuditV1,
    RetainedMailReplayErrorV1, RetainedMailReplayPhaseV1,
};
pub use schema::{
    MAIL_RETAINED_EVIDENCE_REPLAY_DELIVERY_SCHEMA_V1,
    MAIL_RETAINED_EVIDENCE_REPLAY_DELIVERY_STORAGE_BUNDLE_REVISION_V1,
    MAIL_RETAINED_EVIDENCE_REPLAY_SCHEMA_V1,
    MAIL_RETAINED_EVIDENCE_REPLAY_STORAGE_BUNDLE_REVISION_V1,
    MailRetainedEvidenceReplayDeliverySchemaErrorV1, MailRetainedEvidenceReplaySchemaErrorV1,
    append_mail_retained_evidence_replay_delivery_storage_v1,
    append_mail_retained_evidence_replay_storage_v1,
};

pub const PACKAGE: &str = "hermes-mail-retained-evidence-replay-persistence";
