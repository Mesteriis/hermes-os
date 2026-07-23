//! Converts a canonical domain search decision into an owner-local durable job.

use hermes_communications_domain::{
    CommunicationsSearchIndexDecisionV1, CommunicationsSearchIndexRejectionV1,
};
use hermes_communications_persistence::{
    CommunicationsDerivedIndexFailureRecordV1, CommunicationsDerivedIndexFailureV1,
    CommunicationsDerivedIndexJobOperationV1, CommunicationsDerivedIndexJobV1,
    communications_derived_index_job_id_v1,
};

pub enum CommunicationsDerivedIndexWorkV1 {
    Job(CommunicationsDerivedIndexJobV1),
    Failure(CommunicationsDerivedIndexFailureRecordV1),
}

pub fn derived_index_work_from_decision_v1(
    decision: CommunicationsSearchIndexDecisionV1,
    created_at_unix_seconds: i64,
) -> Option<CommunicationsDerivedIndexWorkV1> {
    match decision {
        CommunicationsSearchIndexDecisionV1::Ignore => None,
        CommunicationsSearchIndexDecisionV1::Index(job) => Some(CommunicationsDerivedIndexWorkV1::Job(CommunicationsDerivedIndexJobV1 {
            job_id: communications_derived_index_job_id_v1(job.evidence_id.bytes(), job.message_id.bytes(), job.projection_revision),
            operation: CommunicationsDerivedIndexJobOperationV1::Index,
            evidence_id: job.evidence_id,
            message_id: job.message_id,
            conversation_id: Some(job.conversation_id),
            blob: Some(job.blob),
            projection_revision: job.projection_revision,
            observed_at_unix_seconds: job.observed_at_unix_seconds,
            created_at_unix_seconds,
        })),
        CommunicationsSearchIndexDecisionV1::Remove { evidence_id, message_id, projection_revision, observed_at_unix_seconds } => Some(CommunicationsDerivedIndexWorkV1::Job(CommunicationsDerivedIndexJobV1 {
            job_id: communications_derived_index_job_id_v1(evidence_id.bytes(), message_id.bytes(), projection_revision),
            operation: CommunicationsDerivedIndexJobOperationV1::Remove,
            evidence_id,
            message_id,
            conversation_id: None,
            blob: None,
            projection_revision,
            observed_at_unix_seconds,
            created_at_unix_seconds,
        })),
        CommunicationsSearchIndexDecisionV1::Reject { evidence_id, message_id, projection_revision, observed_at_unix_seconds, reason } => Some(CommunicationsDerivedIndexWorkV1::Failure(CommunicationsDerivedIndexFailureRecordV1 {
            evidence_id,
            message_id,
            projection_revision,
            observed_at_unix_seconds,
            failure: match reason { CommunicationsSearchIndexRejectionV1::DocumentLimit => CommunicationsDerivedIndexFailureV1::DocumentLimit },
            recorded_at_unix_seconds: created_at_unix_seconds,
        })),
    }
}
