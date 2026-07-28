//! Immutable Mail-owned schema bundle for future independent Storage admission.

use hermes_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use sha2::{Digest, Sha256};

use crate::{
    MAIL_SCHEMA_V1, MAIL_SCHEMA_V2, MAIL_SCHEMA_V3, MAIL_SCHEMA_V4, MAIL_SCHEMA_V5, MAIL_SCHEMA_V6,
    MAIL_SCHEMA_V7, MAIL_SCHEMA_V8, MAIL_SCHEMA_V9, MAIL_SCHEMA_V10, MAIL_SCHEMA_V11,
    MAIL_SCHEMA_V12, MAIL_SCHEMA_V13, MAIL_SCHEMA_V14, MAIL_SCHEMA_V15, MAIL_SCHEMA_V16,
    MAIL_SCHEMA_V17,
};

pub const MAIL_STORAGE_BUNDLE_REVISION_V1: u32 = 1;
pub const MAIL_STORAGE_BUNDLE_REVISION_V2: u32 = 2;
pub const MAIL_STORAGE_BUNDLE_REVISION_V3: u32 = 3;
pub const MAIL_STORAGE_BUNDLE_REVISION_V4: u32 = 4;
pub const MAIL_STORAGE_BUNDLE_REVISION_V5: u32 = 5;
pub const MAIL_STORAGE_BUNDLE_REVISION_V6: u32 = 6;
pub const MAIL_STORAGE_BUNDLE_REVISION_V7: u32 = 7;
pub const MAIL_STORAGE_BUNDLE_REVISION_V8: u32 = 8;
pub const MAIL_STORAGE_BUNDLE_REVISION_V9: u32 = 9;
pub const MAIL_STORAGE_BUNDLE_REVISION_V10: u32 = 10;
pub const MAIL_STORAGE_BUNDLE_REVISION_V11: u32 = 11;
pub const MAIL_STORAGE_BUNDLE_REVISION_V12: u32 = 12;
pub const MAIL_STORAGE_BUNDLE_REVISION_V13: u32 = 13;
pub const MAIL_STORAGE_BUNDLE_REVISION_V14: u32 = 14;
pub const MAIL_STORAGE_BUNDLE_REVISION_V15: u32 = 15;
pub const MAIL_STORAGE_BUNDLE_REVISION_V16: u32 = 16;
pub const MAIL_STORAGE_BUNDLE_REVISION_V17: u32 = 17;

/// Returns the complete Mail schema as one immutable initial Storage bundle.
///
/// Mail remains an integration owner: this bundle has no Communications SQL,
/// foreign keys, or runtime dependency. Storage Control admits it separately
/// from the Communications first-owner inventory.
#[must_use]
pub fn mail_storage_bundle_v1() -> StorageBundleV1 {
    StorageBundleV1 {
        major: 1,
        revision: MAIL_STORAGE_BUNDLE_REVISION_V17,
        bundle_id: "mail_state".to_owned(),
        owner_id: "mail".to_owned(),
        steps: vec![
            StorageMigrationStepV1 {
                revision: MAIL_STORAGE_BUNDLE_REVISION_V1,
                migration_id: "mail_state_initial".to_owned(),
                forward_sql_utf8: MAIL_SCHEMA_V1.as_bytes().to_vec(),
                sha256: Sha256::digest(MAIL_SCHEMA_V1.as_bytes()).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: MAIL_STORAGE_BUNDLE_REVISION_V2,
                migration_id: "mail_attachment_security_outbox".to_owned(),
                forward_sql_utf8: MAIL_SCHEMA_V2.as_bytes().to_vec(),
                sha256: Sha256::digest(MAIL_SCHEMA_V2.as_bytes()).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: MAIL_STORAGE_BUNDLE_REVISION_V3,
                migration_id: "mail_delivery_command_queue".to_owned(),
                forward_sql_utf8: MAIL_SCHEMA_V3.as_bytes().to_vec(),
                sha256: Sha256::digest(MAIL_SCHEMA_V3.as_bytes()).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: MAIL_STORAGE_BUNDLE_REVISION_V4,
                migration_id: "mail_gmail_oauth_operations".to_owned(),
                forward_sql_utf8: MAIL_SCHEMA_V4.as_bytes().to_vec(),
                sha256: Sha256::digest(MAIL_SCHEMA_V4.as_bytes()).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: MAIL_STORAGE_BUNDLE_REVISION_V5,
                migration_id: "mail_outbound_attachment_manifest".to_owned(),
                forward_sql_utf8: MAIL_SCHEMA_V5.as_bytes().to_vec(),
                sha256: Sha256::digest(MAIL_SCHEMA_V5.as_bytes()).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: MAIL_STORAGE_BUNDLE_REVISION_V6,
                migration_id: "mail_communications_outbox_causal_order".to_owned(),
                forward_sql_utf8: MAIL_SCHEMA_V6.as_bytes().to_vec(),
                sha256: Sha256::digest(MAIL_SCHEMA_V6.as_bytes()).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: MAIL_STORAGE_BUNDLE_REVISION_V7,
                migration_id: "mail_account_credential_bindings".to_owned(),
                forward_sql_utf8: MAIL_SCHEMA_V7.as_bytes().to_vec(),
                sha256: Sha256::digest(MAIL_SCHEMA_V7.as_bytes()).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: MAIL_STORAGE_BUNDLE_REVISION_V8,
                migration_id: "mail_account_lifecycle".to_owned(),
                forward_sql_utf8: MAIL_SCHEMA_V8.as_bytes().to_vec(),
                sha256: Sha256::digest(MAIL_SCHEMA_V8.as_bytes()).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: MAIL_STORAGE_BUNDLE_REVISION_V9,
                migration_id: "mail_operational_projection".to_owned(),
                forward_sql_utf8: MAIL_SCHEMA_V9.as_bytes().to_vec(),
                sha256: Sha256::digest(MAIL_SCHEMA_V9.as_bytes()).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: MAIL_STORAGE_BUNDLE_REVISION_V10,
                migration_id: "mail_sync_health".to_owned(),
                forward_sql_utf8: MAIL_SCHEMA_V10.as_bytes().to_vec(),
                sha256: Sha256::digest(MAIL_SCHEMA_V10.as_bytes()).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: MAIL_STORAGE_BUNDLE_REVISION_V11,
                migration_id: "mail_composition".to_owned(),
                forward_sql_utf8: MAIL_SCHEMA_V11.as_bytes().to_vec(),
                sha256: Sha256::digest(MAIL_SCHEMA_V11.as_bytes()).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: MAIL_STORAGE_BUNDLE_REVISION_V12,
                migration_id: "mail_message_flag_operations".to_owned(),
                forward_sql_utf8: MAIL_SCHEMA_V12.as_bytes().to_vec(),
                sha256: Sha256::digest(MAIL_SCHEMA_V12.as_bytes()).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: MAIL_STORAGE_BUNDLE_REVISION_V13,
                migration_id: "mail_stable_message_identity_and_imap_locator".to_owned(),
                forward_sql_utf8: MAIL_SCHEMA_V13.as_bytes().to_vec(),
                sha256: Sha256::digest(MAIL_SCHEMA_V13.as_bytes()).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: MAIL_STORAGE_BUNDLE_REVISION_V14,
                migration_id: "mail_stable_message_identity_indexes".to_owned(),
                forward_sql_utf8: MAIL_SCHEMA_V14.as_bytes().to_vec(),
                sha256: Sha256::digest(MAIL_SCHEMA_V14.as_bytes()).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: MAIL_STORAGE_BUNDLE_REVISION_V15,
                migration_id: "mail_message_location_operations".to_owned(),
                forward_sql_utf8: MAIL_SCHEMA_V15.as_bytes().to_vec(),
                sha256: Sha256::digest(MAIL_SCHEMA_V15.as_bytes()).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: MAIL_STORAGE_BUNDLE_REVISION_V16,
                migration_id: "mail_gmail_oauth_authority".to_owned(),
                forward_sql_utf8: MAIL_SCHEMA_V16.as_bytes().to_vec(),
                sha256: Sha256::digest(MAIL_SCHEMA_V16.as_bytes()).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: MAIL_STORAGE_BUNDLE_REVISION_V17,
                migration_id: "mail_message_permanent_delete_operations".to_owned(),
                forward_sql_utf8: MAIL_SCHEMA_V17.as_bytes().to_vec(),
                sha256: Sha256::digest(MAIL_SCHEMA_V17.as_bytes()).to_vec(),
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use hermes_storage_protocol::validation::validate_storage_bundle;

    use super::*;

    #[test]
    fn bundle_is_valid_and_owned_only_by_mail() {
        let bundle = mail_storage_bundle_v1();

        assert_eq!(bundle.owner_id, "mail");
        assert_eq!(bundle.bundle_id, "mail_state");
        assert_eq!(bundle.revision, MAIL_STORAGE_BUNDLE_REVISION_V17);
        assert_eq!(validate_storage_bundle(&bundle), Ok(()));
        assert_eq!(bundle.steps.len(), 17);
        let sql = bundle
            .steps
            .iter()
            .map(|step| {
                std::str::from_utf8(&step.forward_sql_utf8).expect("Mail Storage SQL is UTF-8")
            })
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(sql.matches("CREATE TABLE IF NOT EXISTS ").count(), 33);
        assert_eq!(
            sql.matches("CREATE TABLE IF NOT EXISTS hermes_data.")
                .count(),
            33,
            "every Mail table belongs to the owner-scoped hermes_data schema"
        );
        assert!(sql.contains("mail_attachment_security_outbox"));
        assert!(sql.contains("mail_delivery_queue"));
        assert!(sql.contains("mail_gmail_oauth_attempts"));
        assert!(sql.contains("mail_gmail_oauth_operations"));
        assert!(sql.contains("mail_attachment_safety_projections"));
        assert!(sql.contains("mail_attachment_materializations"));
        assert!(sql.contains("mail_delivery_attachment_manifest"));
        assert!(sql.contains("causal_sequence"));
        assert!(sql.contains("mail_account_credential_bindings"));
        assert!(sql.contains("mail_account_lifecycle_operations"));
        assert!(sql.contains("mail_account_lifecycle_credentials"));
        assert!(sql.contains("mail_account_tombstones"));
        assert!(sql.contains("mail_operational_folders"));
        assert!(sql.contains("mail_operational_threads"));
        assert!(sql.contains("mail_operational_messages"));
        assert!(sql.contains("mail_operational_message_folders"));
        assert!(sql.contains("mail_sync_runs"));
        assert!(sql.contains("mail_sync_status"));
        assert!(sql.contains("mail_message_flag_operations"));
        assert!(sql.contains("mail_message_location_operations"));
        assert!(sql.contains("mail_message_permanent_delete_operations"));
        assert!(sql.contains("mail_imap_message_locators"));
        assert!(!sql.contains("hermes_data.attachment_security_"));
    }
}
