use hermes_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use sha2::{Digest, Sha256};

pub const OLLAMA_AI_STORAGE_BUNDLE_REVISION_V1: u32 = 1;
pub const OLLAMA_AI_SCHEMA_V1: &[u8] = include_bytes!("../migrations/0001_ollama_ai_runs.sql");

#[must_use]
pub fn ollama_ai_storage_bundle_v1() -> StorageBundleV1 {
    StorageBundleV1 {
        major: 1,
        revision: OLLAMA_AI_STORAGE_BUNDLE_REVISION_V1,
        bundle_id: "ollama_ai_runs".to_owned(),
        owner_id: "ollama".to_owned(),
        steps: vec![StorageMigrationStepV1 {
            revision: OLLAMA_AI_STORAGE_BUNDLE_REVISION_V1,
            migration_id: "ollama_ai_runs_initial".to_owned(),
            forward_sql_utf8: OLLAMA_AI_SCHEMA_V1.to_vec(),
            sha256: Sha256::digest(OLLAMA_AI_SCHEMA_V1).to_vec(),
        }],
    }
}

#[cfg(test)]
mod tests {
    use hermes_storage_protocol::validation::validate_storage_bundle;

    use super::*;

    #[test]
    fn schema_is_ollama_owned_and_never_persists_private_input() {
        let bundle = ollama_ai_storage_bundle_v1();
        validate_storage_bundle(&bundle).expect("storage bundle");
        assert_eq!(bundle.owner_id, "ollama");
        let sql = std::str::from_utf8(OLLAMA_AI_SCHEMA_V1).expect("schema");
        for required in [
            "request_digest",
            "settings_revision",
            "result_model_revision_sha256",
            "result_body_utf8",
        ] {
            assert!(sql.contains(required), "{required}");
        }
        for forbidden in [
            "prompt",
            "input_utf8",
            "http_body",
            "endpoint",
            "credential",
            "communications_",
        ] {
            assert!(!sql.contains(forbidden), "{forbidden}");
        }
    }
}
