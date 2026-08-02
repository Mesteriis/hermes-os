#![forbid(unsafe_code)]

mod repository;
mod schema;

pub use repository::{
    MailRetainedEvidenceReplayPersistenceV1, RetainedMailEvidenceV1, RetainedMailReplayAuditV1,
    RetainedMailReplayErrorV1, RetainedMailReplayPhaseV1,
};
pub use schema::{
    MAIL_RETAINED_EVIDENCE_REPLAY_SCHEMA_V1,
    MAIL_RETAINED_EVIDENCE_REPLAY_STORAGE_BUNDLE_REVISION_V1,
    MailRetainedEvidenceReplaySchemaErrorV1, append_mail_retained_evidence_replay_storage_v1,
};

pub const PACKAGE: &str = "hermes-mail-retained-evidence-replay-persistence";
