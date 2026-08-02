use hermes_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use sha2::{Digest, Sha256};

pub const MAIL_CONTACTS_SYNC_STORAGE_BUNDLE_REVISION_V1: u32 = 1;
pub const MAIL_CONTACTS_SYNC_SCHEMA_V1: &[u8] =
    include_bytes!("../migrations/0001_mail_contacts_sync.sql");

#[must_use]
pub fn mail_contacts_sync_storage_bundle_v1() -> StorageBundleV1 {
    StorageBundleV1 {
        major: 1,
        revision: MAIL_CONTACTS_SYNC_STORAGE_BUNDLE_REVISION_V1,
        bundle_id: "mail_contacts_sync".to_owned(),
        owner_id: "mail_contacts_sync".to_owned(),
        steps: vec![StorageMigrationStepV1 {
            revision: MAIL_CONTACTS_SYNC_STORAGE_BUNDLE_REVISION_V1,
            migration_id: "mail_contacts_sync_initial".to_owned(),
            forward_sql_utf8: MAIL_CONTACTS_SYNC_SCHEMA_V1.to_vec(),
            sha256: Sha256::digest(MAIL_CONTACTS_SYNC_SCHEMA_V1).to_vec(),
        }],
    }
}

#[cfg(test)]
mod tests {
    use hermes_storage_protocol::validation::validate_storage_bundle;

    use super::*;

    #[test]
    fn bundle_is_workflow_owned_and_has_no_foreign_tables_or_provider_secrets() {
        let bundle = mail_contacts_sync_storage_bundle_v1();
        validate_storage_bundle(&bundle).expect("valid storage bundle");
        assert_eq!(bundle.owner_id, "mail_contacts_sync");
        let sql = std::str::from_utf8(MAIL_CONTACTS_SYNC_SCHEMA_V1).expect("utf8");
        for required in [
            "mail_contacts_sync_runs",
            "mail_contacts_sync_inbox",
            "mail_contacts_sync_outbox",
            "mail_contacts_sync_realtime",
            "continuation_cursor",
        ] {
            assert!(sql.contains(required), "{required}");
        }
        for forbidden in [
            "contacts_contacts",
            "mail_accounts",
            "communications_",
            "password",
            "access_token",
            "refresh_token",
            "cookie",
        ] {
            assert!(!sql.contains(forbidden), "{forbidden}");
        }
    }
}
