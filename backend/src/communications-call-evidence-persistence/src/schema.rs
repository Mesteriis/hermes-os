use hermes_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use sha2::{Digest, Sha256};

pub const COMMUNICATIONS_CALL_EVIDENCE_STORAGE_BUNDLE_REVISION_V1: u32 = 1;
pub const COMMUNICATIONS_CALL_EVIDENCE_SCHEMA_V1: &[u8] =
    include_bytes!("../migrations/0001_call_evidence.sql");

#[must_use]
pub fn communications_call_evidence_storage_bundle_v1() -> StorageBundleV1 {
    StorageBundleV1 {
        major: 1,
        revision: COMMUNICATIONS_CALL_EVIDENCE_STORAGE_BUNDLE_REVISION_V1,
        bundle_id: "communications_call_evidence_state".to_owned(),
        owner_id: "communications".to_owned(),
        steps: vec![StorageMigrationStepV1 {
            revision: COMMUNICATIONS_CALL_EVIDENCE_STORAGE_BUNDLE_REVISION_V1,
            migration_id: "communications_call_evidence_initial".to_owned(),
            forward_sql_utf8: COMMUNICATIONS_CALL_EVIDENCE_SCHEMA_V1.to_vec(),
            sha256: Sha256::digest(COMMUNICATIONS_CALL_EVIDENCE_SCHEMA_V1).to_vec(),
        }],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundle_is_additive_owner_local_and_private_content_negative() {
        let bundle = communications_call_evidence_storage_bundle_v1();
        assert_eq!(bundle.owner_id, "communications");
        assert_eq!(bundle.steps.len(), 1);
        let sql = String::from_utf8(bundle.steps[0].forward_sql_utf8.clone()).expect("utf8");
        for table in [
            "communications_call_evidence_inbox",
            "communications_call_evidence_projection",
            "communications_call_evidence_history",
            "communications_call_evidence_realtime_sequence",
            "communications_call_evidence_realtime_frames",
        ] {
            assert!(sql.contains(table));
        }
        for forbidden in [
            "provider_call_id",
            "provider_account_id",
            "chat_id",
            "phone_number",
            "username",
            "encryption_key",
            "signaling",
            "pcm",
            "audio_bytes",
            "transcript_text",
            "raw_json",
        ] {
            assert!(!sql.contains(forbidden));
        }
    }
}
