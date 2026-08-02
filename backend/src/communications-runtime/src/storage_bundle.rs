//! Exact Communications storage successor composed by the managed runtime.

use hermes_communications_call_evidence_persistence::{
    CommunicationsCallEvidenceSchemaErrorV1, append_communications_call_evidence_storage_v1,
};
use hermes_communications_persistence::communications_storage_bundle_v1;
use hermes_communications_retained_evidence_replay_persistence::{
    CommunicationsRetainedEvidenceReplayDeliverySchemaErrorV1,
    CommunicationsRetainedEvidenceReplayScanSchemaErrorV1,
    CommunicationsRetainedEvidenceReplaySchemaErrorV1,
    append_communications_retained_evidence_replay_delivery_storage_v1,
    append_communications_retained_evidence_replay_scan_storage_v1,
    append_communications_retained_evidence_replay_storage_v1,
};
use hermes_storage_protocol::v1::StorageBundleV1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsRuntimeStorageBundleErrorV1 {
    CallEvidence(CommunicationsCallEvidenceSchemaErrorV1),
    RetainedEvidenceReplay(CommunicationsRetainedEvidenceReplaySchemaErrorV1),
    RetainedEvidenceReplayDelivery(CommunicationsRetainedEvidenceReplayDeliverySchemaErrorV1),
    RetainedEvidenceReplayScan(CommunicationsRetainedEvidenceReplayScanSchemaErrorV1),
}

pub fn communications_runtime_storage_bundle_v1()
-> Result<StorageBundleV1, CommunicationsRuntimeStorageBundleErrorV1> {
    let bundle = append_communications_call_evidence_storage_v1(communications_storage_bundle_v1())
        .map_err(CommunicationsRuntimeStorageBundleErrorV1::CallEvidence)?;
    let bundle = append_communications_retained_evidence_replay_storage_v1(bundle)
        .map_err(CommunicationsRuntimeStorageBundleErrorV1::RetainedEvidenceReplay)?;
    let bundle = append_communications_retained_evidence_replay_delivery_storage_v1(bundle)
        .map_err(CommunicationsRuntimeStorageBundleErrorV1::RetainedEvidenceReplayDelivery)?;
    append_communications_retained_evidence_replay_scan_storage_v1(bundle)
        .map_err(CommunicationsRuntimeStorageBundleErrorV1::RetainedEvidenceReplayScan)
}
