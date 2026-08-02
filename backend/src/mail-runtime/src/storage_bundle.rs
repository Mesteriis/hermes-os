//! Exact Mail storage successor composed by the managed runtime.

use hermes_mail_persistence::mail_storage_bundle_v1;
use hermes_mail_retained_evidence_replay_persistence::{
    MailRetainedEvidenceReplayDeliverySchemaErrorV1, MailRetainedEvidenceReplaySchemaErrorV1,
    append_mail_retained_evidence_replay_delivery_storage_v1,
    append_mail_retained_evidence_replay_storage_v1,
};
use hermes_storage_protocol::v1::StorageBundleV1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailRuntimeStorageBundleErrorV1 {
    RetainedEvidenceReplay(MailRetainedEvidenceReplaySchemaErrorV1),
    RetainedEvidenceReplayDelivery(MailRetainedEvidenceReplayDeliverySchemaErrorV1),
}

pub fn mail_runtime_storage_bundle_v1() -> Result<StorageBundleV1, MailRuntimeStorageBundleErrorV1>
{
    let bundle = append_mail_retained_evidence_replay_storage_v1(mail_storage_bundle_v1())
        .map_err(MailRuntimeStorageBundleErrorV1::RetainedEvidenceReplay)?;
    append_mail_retained_evidence_replay_delivery_storage_v1(bundle)
        .map_err(MailRuntimeStorageBundleErrorV1::RetainedEvidenceReplayDelivery)
}
