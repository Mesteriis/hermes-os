#![forbid(unsafe_code)]

mod admission;
mod blob_materialization;
mod client_port;
mod client_realtime;
mod contracts;
mod event_outbox;
mod extraction;
mod managed_runtime;
mod source_results;

pub use admission::{
    COMMUNICATION_TASK_CANDIDATE_BLOB_CAPABILITY_ID_V1,
    COMMUNICATION_TASK_CANDIDATE_STORAGE_CAPABILITY_ID_V1,
    communication_task_candidate_module_descriptor_v1,
    communication_task_candidate_settings_schema_bytes_v1,
    communication_task_candidate_settings_schema_v1,
};
pub use blob_materialization::{
    CommunicationTaskCandidateBlobErrorV1, CommunicationTaskCandidateSourceBlobReceiptV1,
};
pub use client_port::{
    get_communication_task_candidate_payload_v1, start_communication_task_candidate_payload_v1,
};
pub use event_outbox::{
    CommunicationTaskCandidateEventRelayErrorV1, relay_source_prepare_outbox_once_v1,
};
pub use extraction::{
    CommunicationTaskCandidateExtractionErrorV1,
    complete_communication_task_candidate_extraction_v1,
    recover_accepted_communication_task_candidate_once_v1,
};
pub use managed_runtime::{
    CommunicationTaskCandidateManagedRuntimeErrorV1, CommunicationTaskCandidateManagedRuntimeV1,
    CommunicationTaskCandidateRuntimeAdmissionV1,
};
pub use source_results::{
    CommunicationTaskCandidateSourceResultErrorV1, consume_task_source_prepared_once_v1,
    consume_task_source_rejected_once_v1,
};

pub const PACKAGE: &str = "hermes-communication-task-candidate-runtime";
