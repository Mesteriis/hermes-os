use hermes_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use sha2::{Digest, Sha256};

pub const AI_INFERENCE_STORAGE_BUNDLE_REVISION_V1: u32 = 2;
pub const AI_INFERENCE_SCHEMA_V1: &[u8] =
    include_bytes!("../migrations/0001_ai_inference_runs.sql");
pub const AI_SUMMARY_SCHEMA_V1: &[u8] = include_bytes!("../migrations/0002_ai_summary_runs.sql");

#[must_use]
pub fn ai_inference_storage_bundle_v1() -> StorageBundleV1 {
    StorageBundleV1 {
        major: 1,
        revision: AI_INFERENCE_STORAGE_BUNDLE_REVISION_V1,
        bundle_id: "ai_inference_runs".to_owned(),
        owner_id: "ai".to_owned(),
        steps: vec![
            StorageMigrationStepV1 {
                revision: 1,
                migration_id: "ai_inference_runs_initial".to_owned(),
                forward_sql_utf8: AI_INFERENCE_SCHEMA_V1.to_vec(),
                sha256: Sha256::digest(AI_INFERENCE_SCHEMA_V1).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: 2,
                migration_id: "ai_summary_runs".to_owned(),
                forward_sql_utf8: AI_SUMMARY_SCHEMA_V1.to_vec(),
                sha256: Sha256::digest(AI_SUMMARY_SCHEMA_V1).to_vec(),
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use hermes_storage_protocol::validation::validate_storage_bundle;

    use super::*;

    #[test]
    fn bundle_is_ai_owned_typed_and_private_source_negative() {
        let bundle = ai_inference_storage_bundle_v1();
        validate_storage_bundle(&bundle).expect("valid AI storage bundle");
        assert_eq!(bundle.owner_id, "ai");
        let sql = std::str::from_utf8(AI_INFERENCE_SCHEMA_V1).expect("utf8");
        let summary_sql = std::str::from_utf8(AI_SUMMARY_SCHEMA_V1).expect("utf8");
        for required in [
            "ai_inference_runs",
            "request_digest",
            "source_reference_id",
            "source_sha256",
            "selected_provider_settings_revision",
            "result_body_utf8",
            "result_prompt_policy_sha256",
        ] {
            assert!(sql.contains(required), "{required}");
        }
        for required in ["ai_summary_runs", "result_summary_utf8", "requested_length"] {
            assert!(summary_sql.contains(required), "{required}");
        }
        for forbidden in [
            "communications_",
            "mail_",
            "telegram_",
            "whatsapp_",
            "zulip_",
            "message_body",
            "provider_id",
            "model_id",
            "endpoint",
            "prompt_text",
        ] {
            assert!(!sql.contains(forbidden), "{forbidden}");
            assert!(!summary_sql.contains(forbidden), "{forbidden}");
        }
    }
}
