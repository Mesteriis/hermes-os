//! Immutable Telegram-owned schema bundle for independent Storage admission.

use hermes_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use sha2::{Digest, Sha256};

use crate::{TELEGRAM_SCHEMA_V1, TELEGRAM_SCHEMA_V2};

pub const TELEGRAM_STORAGE_BUNDLE_REVISION_V1: u32 = 1;
pub const TELEGRAM_STORAGE_BUNDLE_REVISION_V2: u32 = 2;

/// Returns the complete Telegram operational schema as one immutable bundle.
///
/// The bundle is an integration assembly artifact. It contains no
/// Communications tables, foreign keys, or domain migration authority.
#[must_use]
pub fn telegram_storage_bundle_v1() -> StorageBundleV1 {
    StorageBundleV1 {
        major: 1,
        revision: TELEGRAM_STORAGE_BUNDLE_REVISION_V2,
        bundle_id: "telegram_state".to_owned(),
        owner_id: "telegram".to_owned(),
        steps: vec![
            StorageMigrationStepV1 {
                revision: TELEGRAM_STORAGE_BUNDLE_REVISION_V1,
                migration_id: "telegram_state_initial".to_owned(),
                forward_sql_utf8: TELEGRAM_SCHEMA_V1.as_bytes().to_vec(),
                sha256: Sha256::digest(TELEGRAM_SCHEMA_V1.as_bytes()).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: TELEGRAM_STORAGE_BUNDLE_REVISION_V2,
                migration_id: "telegram_delivery_route_locators".to_owned(),
                forward_sql_utf8: TELEGRAM_SCHEMA_V2.as_bytes().to_vec(),
                sha256: Sha256::digest(TELEGRAM_SCHEMA_V2.as_bytes()).to_vec(),
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use hermes_storage_protocol::validation::validate_storage_bundle;

    use super::*;

    #[test]
    fn storage_bundle_is_exactly_telegram_owned_and_valid() {
        let bundle = telegram_storage_bundle_v1();

        assert_eq!(bundle.owner_id, "telegram");
        assert_eq!(bundle.bundle_id, "telegram_state");
        assert_eq!(bundle.revision, TELEGRAM_STORAGE_BUNDLE_REVISION_V2);
        assert_eq!(bundle.steps.len(), 2);
        assert_eq!(validate_storage_bundle(&bundle), Ok(()));
    }
}
