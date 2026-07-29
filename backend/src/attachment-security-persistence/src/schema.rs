//! Immutable owner-local Storage bundle for Attachment Security.

use hermes_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use sha2::{Digest, Sha256};

pub const ATTACHMENT_SECURITY_STORAGE_BUNDLE_REVISION_V1: u32 = 1;
pub const ATTACHMENT_SECURITY_STORAGE_BUNDLE_REVISION_V2: u32 = 2;
pub const ATTACHMENT_SECURITY_STORAGE_BUNDLE_REVISION_V3: u32 = 3;
pub const ATTACHMENT_SECURITY_STORAGE_BUNDLE_REVISION_V4: u32 = 4;
pub const ATTACHMENT_SECURITY_STORAGE_BUNDLE_REVISION_V5: u32 = 5;
pub const ATTACHMENT_SECURITY_SCHEMA_V1: &[u8] =
    include_bytes!("../migrations/0001_attachment_security_state.sql");
pub const ATTACHMENT_SECURITY_SCHEMA_V2: &[u8] =
    include_bytes!("../migrations/0002_attachment_security_blob_custody.sql");
pub const ATTACHMENT_SECURITY_SCHEMA_V3: &[u8] =
    include_bytes!("../migrations/0003_attachment_security_custody_successor_retry_policy.sql");
pub const ATTACHMENT_SECURITY_SCHEMA_V4: &[u8] =
    include_bytes!("../migrations/0004_attachment_security_retry_policy_recovery_index.sql");
pub const ATTACHMENT_SECURITY_SCHEMA_V5: &[u8] = include_bytes!(
    "../migrations/0005_attachment_security_scanner_retry_policy_recovery_index.sql"
);

#[must_use]
pub fn attachment_security_storage_bundle_v1() -> StorageBundleV1 {
    StorageBundleV1 {
        major: 1,
        revision: ATTACHMENT_SECURITY_STORAGE_BUNDLE_REVISION_V5,
        bundle_id: "attachment_security_state".to_owned(),
        owner_id: "attachment_security".to_owned(),
        steps: vec![
            StorageMigrationStepV1 {
                revision: ATTACHMENT_SECURITY_STORAGE_BUNDLE_REVISION_V1,
                migration_id: "attachment_security_state_initial".to_owned(),
                forward_sql_utf8: ATTACHMENT_SECURITY_SCHEMA_V1.to_vec(),
                sha256: Sha256::digest(ATTACHMENT_SECURITY_SCHEMA_V1).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: ATTACHMENT_SECURITY_STORAGE_BUNDLE_REVISION_V2,
                migration_id: "attachment_security_blob_custody".to_owned(),
                forward_sql_utf8: ATTACHMENT_SECURITY_SCHEMA_V2.to_vec(),
                sha256: Sha256::digest(ATTACHMENT_SECURITY_SCHEMA_V2).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: ATTACHMENT_SECURITY_STORAGE_BUNDLE_REVISION_V3,
                migration_id: "attachment_security_custody_successor_retry_policy".to_owned(),
                forward_sql_utf8: ATTACHMENT_SECURITY_SCHEMA_V3.to_vec(),
                sha256: Sha256::digest(ATTACHMENT_SECURITY_SCHEMA_V3).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: ATTACHMENT_SECURITY_STORAGE_BUNDLE_REVISION_V4,
                migration_id: "attachment_security_retry_policy_recovery_index".to_owned(),
                forward_sql_utf8: ATTACHMENT_SECURITY_SCHEMA_V4.to_vec(),
                sha256: Sha256::digest(ATTACHMENT_SECURITY_SCHEMA_V4).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: ATTACHMENT_SECURITY_STORAGE_BUNDLE_REVISION_V5,
                migration_id: "attachment_security_scanner_retry_policy_recovery_index".to_owned(),
                forward_sql_utf8: ATTACHMENT_SECURITY_SCHEMA_V5.to_vec(),
                sha256: Sha256::digest(ATTACHMENT_SECURITY_SCHEMA_V5).to_vec(),
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use hermes_storage_protocol::validation::validate_storage_bundle;

    use super::*;

    #[test]
    fn bundle_is_valid_and_contains_only_owner_scoped_tables() {
        let bundle = attachment_security_storage_bundle_v1();
        let sql = std::str::from_utf8(ATTACHMENT_SECURITY_SCHEMA_V1).expect("UTF-8 schema");

        assert_eq!(bundle.owner_id, "attachment_security");
        assert_eq!(
            bundle.revision,
            ATTACHMENT_SECURITY_STORAGE_BUNDLE_REVISION_V5
        );
        assert_eq!(validate_storage_bundle(&bundle), Ok(()));
        assert_eq!(bundle.steps.len(), 5);
        assert_eq!(sql.matches("CREATE TABLE hermes_data.").count(), 7);
        assert!(!sql.contains("hermes_data.communications_"));
        assert!(!sql.contains("hermes_data.mail_"));
    }
}
