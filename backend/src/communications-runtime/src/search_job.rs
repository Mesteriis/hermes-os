//! Converts a canonical domain search decision into an owner-local durable job.

use hermes_communications_domain::CommunicationsSearchIndexDecisionV1;
use hermes_communications_persistence::{
    CommunicationsDerivedIndexJobOperationV1, CommunicationsDerivedIndexJobV1,
};
use sha2::{Digest, Sha256};

pub fn derived_index_job_from_decision_v1(
    decision: CommunicationsSearchIndexDecisionV1,
    created_at_unix_seconds: i64,
) -> Option<CommunicationsDerivedIndexJobV1> {
    match decision {
        CommunicationsSearchIndexDecisionV1::Ignore => None,
        CommunicationsSearchIndexDecisionV1::Index(job) => Some(CommunicationsDerivedIndexJobV1 {
            job_id: job_id(job.evidence_id.bytes(), job.message_id.bytes(), job.projection_revision),
            operation: CommunicationsDerivedIndexJobOperationV1::Index,
            evidence_id: job.evidence_id,
            message_id: job.message_id,
            conversation_id: Some(job.conversation_id),
            blob: Some(job.blob),
            projection_revision: job.projection_revision,
            observed_at_unix_seconds: job.observed_at_unix_seconds,
            created_at_unix_seconds,
        }),
        CommunicationsSearchIndexDecisionV1::Remove { evidence_id, message_id, projection_revision, observed_at_unix_seconds } => Some(CommunicationsDerivedIndexJobV1 {
            job_id: job_id(evidence_id.bytes(), message_id.bytes(), projection_revision),
            operation: CommunicationsDerivedIndexJobOperationV1::Remove,
            evidence_id,
            message_id,
            conversation_id: None,
            blob: None,
            projection_revision,
            observed_at_unix_seconds,
            created_at_unix_seconds,
        }),
    }
}

fn job_id(evidence_id: [u8; 16], message_id: [u8; 16], revision: u32) -> [u8; 16] {
    let mut digest = Sha256::new();
    digest.update(b"hermes.communications.derived-index-job.v1\0");
    digest.update(evidence_id);
    digest.update(message_id);
    digest.update(revision.to_be_bytes());
    let value: [u8; 32] = digest.finalize().into();
    value[..16].try_into().expect("fixed SHA-256 prefix")
}
