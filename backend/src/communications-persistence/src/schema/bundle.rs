//! Canonical Communications Storage bundle construction.

use hermes_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use sha2::{Digest, Sha256};

const INITIAL_SCHEMA: &[u8] =
    include_bytes!("../../migrations/0001_communications_state.sql");

pub const COMMUNICATIONS_STORAGE_BUNDLE_REVISION_V1: u32 = 1;

/// Immutable Communications schema admitted and applied only by Storage Control.
#[must_use]
pub fn communications_storage_bundle_v1() -> StorageBundleV1 {
    StorageBundleV1 {
        major: 1,
        revision: COMMUNICATIONS_STORAGE_BUNDLE_REVISION_V1,
        bundle_id: "communications_state".to_owned(),
        owner_id: "communications".to_owned(),
        steps: vec![StorageMigrationStepV1 {
            revision: 1,
            migration_id: "communications_state_initial".to_owned(),
            forward_sql_utf8: INITIAL_SCHEMA.to_vec(),
            sha256: Sha256::digest(INITIAL_SCHEMA).to_vec(),
        }],
    }
}

#[cfg(test)]
mod tests {
    use hermes_storage_protocol::validation::validate_storage_bundle;

    use super::*;

    #[test]
    fn storage_bundle_is_structurally_valid_and_owner_scoped() {
        let bundle = communications_storage_bundle_v1();

        assert_eq!(bundle.owner_id, "communications");
        assert_eq!(bundle.revision, COMMUNICATIONS_STORAGE_BUNDLE_REVISION_V1);
        assert_eq!(validate_storage_bundle(&bundle), Ok(()));
    }
}
