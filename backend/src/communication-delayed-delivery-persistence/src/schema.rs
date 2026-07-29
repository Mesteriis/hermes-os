use hermes_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use sha2::{Digest, Sha256};

pub const COMMUNICATION_DELAYED_DELIVERY_STORAGE_BUNDLE_REVISION_V1: u32 = 1;
pub const COMMUNICATION_DELAYED_DELIVERY_STORAGE_BUNDLE_REVISION_V2: u32 = 2;
pub const COMMUNICATION_DELAYED_DELIVERY_SCHEMA_V1: &[u8] =
    include_bytes!("../migrations/0001_delayed_delivery_state.sql");
pub const COMMUNICATION_DELAYED_DELIVERY_SCHEDULER_RECEIPT_SCHEMA_V2: &[u8] =
    include_bytes!("../migrations/0002_scheduler_receipt_outbox.sql");

#[must_use]
pub fn communication_delayed_delivery_storage_bundle_v1() -> StorageBundleV1 {
    StorageBundleV1 {
        major: 1,
        revision: COMMUNICATION_DELAYED_DELIVERY_STORAGE_BUNDLE_REVISION_V2,
        bundle_id: "communication_delayed_delivery_state".to_owned(),
        owner_id: "communication_delayed_delivery".to_owned(),
        steps: vec![
            StorageMigrationStepV1 {
                revision: COMMUNICATION_DELAYED_DELIVERY_STORAGE_BUNDLE_REVISION_V1,
                migration_id: "communication_delayed_delivery_state_initial".to_owned(),
                forward_sql_utf8: COMMUNICATION_DELAYED_DELIVERY_SCHEMA_V1.to_vec(),
                sha256: Sha256::digest(COMMUNICATION_DELAYED_DELIVERY_SCHEMA_V1).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: COMMUNICATION_DELAYED_DELIVERY_STORAGE_BUNDLE_REVISION_V2,
                migration_id: "communication_delayed_delivery_scheduler_receipts".to_owned(),
                forward_sql_utf8: COMMUNICATION_DELAYED_DELIVERY_SCHEDULER_RECEIPT_SCHEMA_V2
                    .to_vec(),
                sha256: Sha256::digest(COMMUNICATION_DELAYED_DELIVERY_SCHEDULER_RECEIPT_SCHEMA_V2)
                    .to_vec(),
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundle_is_owner_local_and_stores_no_plaintext_body() {
        let bundle = communication_delayed_delivery_storage_bundle_v1();
        assert_eq!(bundle.owner_id, "communication_delayed_delivery");
        assert_eq!(bundle.revision, 2);
        assert_eq!(bundle.steps.len(), 2);
        let sql = std::str::from_utf8(COMMUNICATION_DELAYED_DELIVERY_SCHEMA_V1)
            .expect("migration is UTF-8");
        assert!(sql.contains("communication_delayed_delivery_operations"));
        assert!(sql.contains("communication_delayed_delivery_scheduler_inbox"));
        assert!(sql.contains("communication_delayed_delivery_outbox"));
        assert!(sql.contains("body_reference_id"));
        assert!(!sql.contains("body_utf8"));
        assert!(!sql.contains("provider"));
        assert!(!sql.contains("account_id"));
        let receipt_sql =
            std::str::from_utf8(COMMUNICATION_DELAYED_DELIVERY_SCHEDULER_RECEIPT_SCHEMA_V2)
                .expect("receipt migration is UTF-8");
        assert!(receipt_sql.contains("scheduler.job_run.acceptance.v1"));
        assert!(receipt_sql.contains("scheduler.job_run.result.v1"));
    }
}
