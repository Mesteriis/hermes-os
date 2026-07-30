//! Exact Communications storage successor composed by the managed runtime.

use hermes_communications_call_evidence_persistence::{
    CommunicationsCallEvidenceSchemaErrorV1, append_communications_call_evidence_storage_v1,
};
use hermes_communications_persistence::communications_storage_bundle_v1;
use hermes_storage_protocol::v1::StorageBundleV1;

pub fn communications_runtime_storage_bundle_v1()
-> Result<StorageBundleV1, CommunicationsCallEvidenceSchemaErrorV1> {
    append_communications_call_evidence_storage_v1(communications_storage_bundle_v1())
}
