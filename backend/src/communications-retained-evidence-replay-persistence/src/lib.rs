#![forbid(unsafe_code)]

mod repository;
mod schema;

pub use repository::{
    CommunicationsRetainedEvidenceReplayPersistenceV1, RetainedCommunicationsEvidenceV1,
    RetainedCommunicationsReplayAuditV1, RetainedCommunicationsReplayErrorV1,
    RetainedCommunicationsReplayPhaseV1,
};
pub use schema::{
    COMMUNICATIONS_RETAINED_EVIDENCE_REPLAY_SCHEMA_V1,
    COMMUNICATIONS_RETAINED_EVIDENCE_REPLAY_STORAGE_BUNDLE_REVISION_V1,
    CommunicationsRetainedEvidenceReplaySchemaErrorV1,
    append_communications_retained_evidence_replay_storage_v1,
};

pub const PACKAGE: &str = "hermes-communications-retained-evidence-replay-persistence";
