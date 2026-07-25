//! Immutable Zulip-owned schema bundle for independent Storage admission.

use hermes_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use sha2::{Digest, Sha256};

use crate::ZULIP_SCHEMA_V1;

pub const ZULIP_STORAGE_BUNDLE_REVISION_V1: u32 = 1;

/// Returns the complete Zulip schema as one immutable initial Storage bundle.
///
/// Zulip remains an integration owner: this bundle has no
/// Communications-owned SQL, cross-owner foreign keys, or runtime dependency.
/// Storage Control admits it separately from the Communications inventory.
#[must_use]
pub fn zulip_storage_bundle_v1() -> StorageBundleV1 {
    StorageBundleV1 {
        major: 1,
        revision: ZULIP_STORAGE_BUNDLE_REVISION_V1,
        bundle_id: "zulip_state".to_owned(),
        owner_id: "zulip".to_owned(),
        steps: vec![StorageMigrationStepV1 {
            revision: ZULIP_STORAGE_BUNDLE_REVISION_V1,
            migration_id: "zulip_state_initial".to_owned(),
            forward_sql_utf8: ZULIP_SCHEMA_V1.as_bytes().to_vec(),
            sha256: Sha256::digest(ZULIP_SCHEMA_V1.as_bytes()).to_vec(),
        }],
    }
}

#[cfg(test)]
mod tests {
    use hermes_storage_protocol::validation::validate_storage_bundle;

    use super::*;

    #[test]
    fn bundle_is_valid_and_owned_only_by_zulip() {
        let bundle = zulip_storage_bundle_v1();

        assert_eq!(bundle.owner_id, "zulip");
        assert_eq!(bundle.bundle_id, "zulip_state");
        assert_eq!(bundle.revision, ZULIP_STORAGE_BUNDLE_REVISION_V1);
        assert_eq!(bundle.steps.len(), 1);
        assert_eq!(validate_storage_bundle(&bundle), Ok(()));
        assert_eq!(bundle.steps[0].forward_sql_utf8, ZULIP_SCHEMA_V1.as_bytes());
        let sql = std::str::from_utf8(&bundle.steps[0].forward_sql_utf8)
            .expect("Zulip Storage SQL is UTF-8");
        assert_eq!(sql.matches("CREATE TABLE IF NOT EXISTS ").count(), 4);
        assert!(!sql.contains("hermes_communications"));
        assert!(!sql.contains("REFERENCES communications_"));
    }
}
