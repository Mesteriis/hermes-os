//! Exact Mail storage successor composed by the managed runtime.

use hermes_mail_persistence::mail_storage_bundle_v1;
use hermes_mail_retained_evidence_replay_persistence::{
    MailRetainedEvidenceReplaySchemaErrorV1, append_mail_retained_evidence_replay_storage_v1,
};
use hermes_storage_protocol::v1::StorageBundleV1;

pub fn mail_runtime_storage_bundle_v1()
-> Result<StorageBundleV1, MailRetainedEvidenceReplaySchemaErrorV1> {
    append_mail_retained_evidence_replay_storage_v1(mail_storage_bundle_v1())
}
