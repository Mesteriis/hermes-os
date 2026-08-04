use hermes_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use sha2::{Digest, Sha256};

pub const STORAGE_BUNDLE_REVISION_V1: u32 = 1;
pub const SCHEMA_V1: &[u8] = include_bytes!("../migrations/0001_desktop_call_recording.sql");

#[must_use]
pub fn desktop_call_recording_storage_bundle_v1() -> StorageBundleV1 {
    StorageBundleV1 {
        major: 1,
        revision: STORAGE_BUNDLE_REVISION_V1,
        bundle_id: "desktop_call_recording".to_owned(),
        owner_id: "desktop_call_recording".to_owned(),
        steps: vec![StorageMigrationStepV1 {
            revision: STORAGE_BUNDLE_REVISION_V1,
            migration_id: "desktop_call_recording_initial".to_owned(),
            forward_sql_utf8: SCHEMA_V1.to_vec(),
            sha256: Sha256::digest(SCHEMA_V1).to_vec(),
        }],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hermes_storage_protocol::validation::validate_storage_bundle;

    #[test]
    fn storage_is_integration_owned_and_content_negative() {
        let bundle = desktop_call_recording_storage_bundle_v1();
        validate_storage_bundle(&bundle).expect("storage bundle");
        assert_eq!(bundle.owner_id, "desktop_call_recording");
        let sql = std::str::from_utf8(SCHEMA_V1).expect("utf8");
        for forbidden in [
            "audio_bytes",
            "canonical_wav",
            "filesystem_path",
            "audio_input_label",
            "consent_body",
            "custody_transfer_source_proof",
        ] {
            assert!(!sql.contains(forbidden), "forbidden {forbidden}");
        }
    }
}
