use hermes_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use sha2::{Digest, Sha256};

pub const COMMUNICATION_DELIVERY_INTENT_STORAGE_BUNDLE_REVISION_V1: u32 = 1;
pub const COMMUNICATION_DELIVERY_INTENT_SCHEMA_V1: &[u8] =
    include_bytes!("../migrations/0001_delivery_intent_state.sql");

#[must_use]
pub fn communication_delivery_intent_storage_bundle_v1() -> StorageBundleV1 {
    StorageBundleV1 {
        major: 1,
        revision: COMMUNICATION_DELIVERY_INTENT_STORAGE_BUNDLE_REVISION_V1,
        bundle_id: "communication_delivery_intent_state".to_owned(),
        owner_id: "communication_delivery_intent".to_owned(),
        steps: vec![StorageMigrationStepV1 {
            revision: COMMUNICATION_DELIVERY_INTENT_STORAGE_BUNDLE_REVISION_V1,
            migration_id: "communication_delivery_intent_state_initial".to_owned(),
            forward_sql_utf8: COMMUNICATION_DELIVERY_INTENT_SCHEMA_V1.to_vec(),
            sha256: Sha256::digest(COMMUNICATION_DELIVERY_INTENT_SCHEMA_V1).to_vec(),
        }],
    }
}

#[cfg(test)]
mod tests {
    use hermes_storage_protocol::validation::validate_storage_bundle;

    use super::*;

    #[test]
    fn bundle_is_valid_owner_scoped_and_ciphertext_only() {
        let bundle = communication_delivery_intent_storage_bundle_v1();
        validate_storage_bundle(&bundle).expect("valid owner storage bundle");
        assert_eq!(bundle.owner_id, "communication_delivery_intent");
        assert_eq!(bundle.steps.len(), 1);
        let sql = std::str::from_utf8(&bundle.steps[0].forward_sql_utf8).expect("utf8");
        assert!(sql.contains("hermes_data.communication_delivery_intent_jobs"));
        assert!(sql.contains("body_ciphertext"));
        assert!(sql.contains("claim_epoch"));
        assert!(sql.contains("PRIMARY KEY (logical_owner_id, intent_id)"));
        assert!(!sql.contains("body_utf8"));
        assert!(!sql.contains("communications_messages"));
        assert!(!sql.contains("mail_"));
        assert!(!sql.contains("telegram_"));
    }
}
