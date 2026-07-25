//! Immutable Mail-owned schema bundle for future independent Storage admission.

use hermes_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use sha2::{Digest, Sha256};

use crate::MAIL_SCHEMA_V1;

pub const MAIL_STORAGE_BUNDLE_REVISION_V1: u32 = 1;

/// Returns the complete Mail schema as one immutable initial Storage bundle.
///
/// Mail remains an integration owner: this bundle has no Communications SQL,
/// foreign keys, or runtime dependency. Storage Control admits it separately
/// from the Communications first-owner inventory.
#[must_use]
pub fn mail_storage_bundle_v1() -> StorageBundleV1 {
    StorageBundleV1 {
        major: 1,
        revision: MAIL_STORAGE_BUNDLE_REVISION_V1,
        bundle_id: "mail_state".to_owned(),
        owner_id: "mail".to_owned(),
        steps: vec![StorageMigrationStepV1 {
            revision: MAIL_STORAGE_BUNDLE_REVISION_V1,
            migration_id: "mail_state_initial".to_owned(),
            forward_sql_utf8: MAIL_SCHEMA_V1.as_bytes().to_vec(),
            sha256: Sha256::digest(MAIL_SCHEMA_V1.as_bytes()).to_vec(),
        }],
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
        assert_eq!(bundle.revision, MAIL_STORAGE_BUNDLE_REVISION_V1);
        assert_eq!(validate_storage_bundle(&bundle), Ok(()));
        let sql = std::str::from_utf8(&bundle.steps[0].forward_sql_utf8)
            .expect("Mail Storage SQL is UTF-8");
        assert_eq!(sql.matches("CREATE TABLE IF NOT EXISTS ").count(), 8);
        assert_eq!(
            sql.matches("CREATE TABLE IF NOT EXISTS hermes_data.")
                .count(),
            8,
            "every Mail table belongs to the owner-scoped hermes_data schema"
        );
    }
}
