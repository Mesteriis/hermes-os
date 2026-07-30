use hermes_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use sha2::{Digest, Sha256};

pub const COMMUNICATION_CROSS_CHANNEL_FORWARD_STORAGE_BUNDLE_REVISION_V1: u32 = 1;
pub const COMMUNICATION_CROSS_CHANNEL_FORWARD_SCHEMA_V1: &[u8] =
    include_bytes!("../migrations/0001_forward_state.sql");

#[must_use]
pub fn communication_cross_channel_forward_storage_bundle_v1() -> StorageBundleV1 {
    StorageBundleV1 {
        major: 1,
        revision: COMMUNICATION_CROSS_CHANNEL_FORWARD_STORAGE_BUNDLE_REVISION_V1,
        bundle_id: "communication_cross_channel_forward_state".to_owned(),
        owner_id: "communication_cross_channel_forward".to_owned(),
        steps: vec![StorageMigrationStepV1 {
            revision: COMMUNICATION_CROSS_CHANNEL_FORWARD_STORAGE_BUNDLE_REVISION_V1,
            migration_id: "communication_cross_channel_forward_state_initial".to_owned(),
            forward_sql_utf8: COMMUNICATION_CROSS_CHANNEL_FORWARD_SCHEMA_V1.to_vec(),
            sha256: Sha256::digest(COMMUNICATION_CROSS_CHANNEL_FORWARD_SCHEMA_V1).to_vec(),
        }],
    }
}

#[cfg(test)]
mod tests {
    use hermes_storage_protocol::validation::validate_storage_bundle;

    use super::*;

    #[test]
    fn bundle_is_owner_local_bounded_and_contains_no_plaintext_body() {
        let bundle = communication_cross_channel_forward_storage_bundle_v1();
        validate_storage_bundle(&bundle).expect("valid owner storage bundle");
        assert_eq!(bundle.owner_id, "communication_cross_channel_forward");
        assert_eq!(bundle.revision, 1);
        let sql = std::str::from_utf8(COMMUNICATION_CROSS_CHANNEL_FORWARD_SCHEMA_V1)
            .expect("migration is utf8");
        assert!(sql.contains("communication_cross_channel_forward_operations"));
        assert!(sql.contains("communication_cross_channel_forward_cleanup"));
        assert!(sql.contains("communication_cross_channel_forward_realtime"));
        assert!(sql.contains("attempt_count BETWEEN 0 AND 32"));
        assert!(!sql.contains("body_utf8"));
        assert!(!sql.contains("provider"));
        assert!(!sql.contains("mail_"));
        assert!(!sql.contains("telegram_"));
        assert!(!sql.contains("whatsapp_"));
        assert!(!sql.contains("zulip_"));
    }
}
