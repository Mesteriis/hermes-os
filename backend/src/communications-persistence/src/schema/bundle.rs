//! Canonical Communications Storage bundle construction.

use hermes_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use sha2::{Digest, Sha256};

const INITIAL_SCHEMA: &[u8] = include_bytes!("../../migrations/0001_communications_state.sql");
const SEARCH_PROJECTION_SCHEMA: &[u8] =
    include_bytes!("../../migrations/0002_communications_search_projection.sql");
const SEARCH_JOBS_SCHEMA: &[u8] =
    include_bytes!("../../migrations/0003_communications_search_jobs.sql");
const SEARCH_JOB_BLOB_RANGE_SCHEMA: &[u8] =
    include_bytes!("../../migrations/0004_communications_search_job_blob_range.sql");
const SEARCH_JOB_LIFECYCLE_SCHEMA: &[u8] =
    include_bytes!("../../migrations/0005_communications_search_job_lifecycle.sql");
const CANONICAL_MESSAGE_BODY_STATE_SCHEMA: &[u8] =
    include_bytes!("../../migrations/0006_communications_canonical_message_body_state.sql");
const SEARCH_PROJECTION_TOMBSTONES_SCHEMA: &[u8] =
    include_bytes!("../../migrations/0007_communications_search_projection_tombstones.sql");
const BODY_CUSTODY_TRANSFERS_SCHEMA: &[u8] =
    include_bytes!("../../migrations/0008_communications_body_custody_transfers.sql");
const BODY_CUSTODY_TRANSFER_LIFECYCLE_SCHEMA: &[u8] =
    include_bytes!("../../migrations/0009_communications_body_custody_transfer_lifecycle.sql");

pub const COMMUNICATIONS_STORAGE_BUNDLE_REVISION_V1: u32 = 9;

/// Immutable Communications schema admitted and applied only by Storage Control.
#[must_use]
pub fn communications_storage_bundle_v1() -> StorageBundleV1 {
    StorageBundleV1 {
        major: 1,
        revision: COMMUNICATIONS_STORAGE_BUNDLE_REVISION_V1,
        bundle_id: "communications_state".to_owned(),
        owner_id: "communications".to_owned(),
        steps: vec![
            StorageMigrationStepV1 {
                revision: 1,
                migration_id: "communications_state_initial".to_owned(),
                forward_sql_utf8: INITIAL_SCHEMA.to_vec(),
                sha256: Sha256::digest(INITIAL_SCHEMA).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: 2,
                migration_id: "communications_search_projection".to_owned(),
                forward_sql_utf8: SEARCH_PROJECTION_SCHEMA.to_vec(),
                sha256: Sha256::digest(SEARCH_PROJECTION_SCHEMA).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: 3,
                migration_id: "communications_search_jobs".to_owned(),
                forward_sql_utf8: SEARCH_JOBS_SCHEMA.to_vec(),
                sha256: Sha256::digest(SEARCH_JOBS_SCHEMA).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: 4,
                migration_id: "communications_search_job_blob_range".to_owned(),
                forward_sql_utf8: SEARCH_JOB_BLOB_RANGE_SCHEMA.to_vec(),
                sha256: Sha256::digest(SEARCH_JOB_BLOB_RANGE_SCHEMA).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: 5,
                migration_id: "communications_search_job_lifecycle".to_owned(),
                forward_sql_utf8: SEARCH_JOB_LIFECYCLE_SCHEMA.to_vec(),
                sha256: Sha256::digest(SEARCH_JOB_LIFECYCLE_SCHEMA).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: 6,
                migration_id: "communications_canonical_message_body_state".to_owned(),
                forward_sql_utf8: CANONICAL_MESSAGE_BODY_STATE_SCHEMA.to_vec(),
                sha256: Sha256::digest(CANONICAL_MESSAGE_BODY_STATE_SCHEMA).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: 7,
                migration_id: "communications_search_projection_tombstones".to_owned(),
                forward_sql_utf8: SEARCH_PROJECTION_TOMBSTONES_SCHEMA.to_vec(),
                sha256: Sha256::digest(SEARCH_PROJECTION_TOMBSTONES_SCHEMA).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: 8,
                migration_id: "communications_body_custody_transfers".to_owned(),
                forward_sql_utf8: BODY_CUSTODY_TRANSFERS_SCHEMA.to_vec(),
                sha256: Sha256::digest(BODY_CUSTODY_TRANSFERS_SCHEMA).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: 9,
                migration_id: "communications_body_custody_transfer_lifecycle".to_owned(),
                forward_sql_utf8: BODY_CUSTODY_TRANSFER_LIFECYCLE_SCHEMA.to_vec(),
                sha256: Sha256::digest(BODY_CUSTODY_TRANSFER_LIFECYCLE_SCHEMA).to_vec(),
            },
        ],
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
