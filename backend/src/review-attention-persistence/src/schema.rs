use hermes_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use sha2::{Digest, Sha256};

pub const REVIEW_ATTENTION_STORAGE_BUNDLE_REVISION_V1: u32 = 1;
pub const REVIEW_ATTENTION_SCHEMA_V1: &[u8] =
    include_bytes!("../migrations/0001_review_attention.sql");

#[must_use]
pub fn review_attention_storage_bundle_v1() -> StorageBundleV1 {
    StorageBundleV1 {
        major: 1,
        revision: REVIEW_ATTENTION_STORAGE_BUNDLE_REVISION_V1,
        bundle_id: "review_attention_state".to_owned(),
        owner_id: "review".to_owned(),
        steps: vec![StorageMigrationStepV1 {
            revision: REVIEW_ATTENTION_STORAGE_BUNDLE_REVISION_V1,
            migration_id: "review_attention_initial".to_owned(),
            forward_sql_utf8: REVIEW_ATTENTION_SCHEMA_V1.to_vec(),
            sha256: Sha256::digest(REVIEW_ATTENTION_SCHEMA_V1).to_vec(),
        }],
    }
}

#[cfg(test)]
mod tests {
    use hermes_storage_protocol::validation::validate_storage_bundle;

    use super::*;

    #[test]
    fn bundle_is_additive_owner_local_and_content_negative() {
        let bundle = review_attention_storage_bundle_v1();
        validate_storage_bundle(&bundle).expect("valid Review storage bundle");
        assert_eq!(bundle.owner_id, "review");
        assert_eq!(bundle.revision, 1);
        let sql = std::str::from_utf8(REVIEW_ATTENTION_SCHEMA_V1).expect("utf8");
        assert!(sql.contains("review_attention_state"));
        assert!(sql.contains("review_attention_operations"));
        assert!(sql.contains("request_sha256 BYTEA"));
        assert!(sql.contains("expected_revision BIGINT"));
        for forbidden in [
            "communications_",
            "mail_",
            "telegram_",
            "provider",
            "message_body",
            "subject",
            "email_address",
            "phone_number",
        ] {
            assert!(!sql.contains(forbidden), "{forbidden}");
        }
    }
}
