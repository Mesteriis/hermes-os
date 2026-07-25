//! Immutable WhatsApp-owned schema bundle for independent Storage admission.

use hermes_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use sha2::{Digest, Sha256};

pub const WHATSAPP_STORAGE_BUNDLE_REVISION_V1: u32 = 1;

pub const WHATSAPP_SCHEMA_V1: &str = r#"
CREATE TABLE IF NOT EXISTS hermes_data.whatsapp_communications_outbox (
    message_id BYTEA PRIMARY KEY,
    envelope_sha256 BYTEA NOT NULL,
    exact_envelope_bytes BYTEA NOT NULL,
    created_at_unix_seconds BIGINT NOT NULL,
    published_at_unix_seconds BIGINT,
    CHECK (octet_length(message_id) = 16),
    CHECK (octet_length(envelope_sha256) = 32),
    CHECK (octet_length(exact_envelope_bytes) > 0)
);
CREATE INDEX IF NOT EXISTS whatsapp_communications_outbox_pending_idx
    ON hermes_data.whatsapp_communications_outbox (created_at_unix_seconds, message_id)
    WHERE published_at_unix_seconds IS NULL;
CREATE TABLE IF NOT EXISTS hermes_data.whatsapp_host_observations (
    account_id TEXT NOT NULL,
    provider_event_id TEXT NOT NULL,
    evidence_kind SMALLINT NOT NULL,
    observed_at_unix_seconds BIGINT NOT NULL,
    PRIMARY KEY (account_id, provider_event_id),
    CHECK (char_length(account_id) BETWEEN 1 AND 256),
    CHECK (char_length(provider_event_id) BETWEEN 1 AND 256),
    CHECK (evidence_kind BETWEEN 1 AND 11)
);
CREATE TABLE IF NOT EXISTS hermes_data.whatsapp_provider_commands (
    operation_id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL,
    exact_command_bytes BYTEA NOT NULL,
    state SMALLINT NOT NULL,
    host_claim_id TEXT,
    lease_expires_at_unix_seconds BIGINT,
    requested_at_unix_seconds BIGINT NOT NULL,
    completed_at_unix_seconds BIGINT,
    CHECK (char_length(operation_id) BETWEEN 1 AND 256),
    CHECK (char_length(account_id) BETWEEN 1 AND 256),
    CHECK (octet_length(exact_command_bytes) BETWEEN 1 AND 524288),
    CHECK (state BETWEEN 1 AND 4),
    CHECK ((state = 1 AND host_claim_id IS NULL AND lease_expires_at_unix_seconds IS NULL AND completed_at_unix_seconds IS NULL)
        OR (state = 2 AND host_claim_id IS NOT NULL AND lease_expires_at_unix_seconds IS NOT NULL AND completed_at_unix_seconds IS NULL)
        OR (state IN (3, 4) AND host_claim_id IS NOT NULL AND lease_expires_at_unix_seconds IS NOT NULL AND completed_at_unix_seconds IS NOT NULL))
);
CREATE INDEX IF NOT EXISTS whatsapp_provider_commands_claimable_idx
    ON hermes_data.whatsapp_provider_commands (account_id, requested_at_unix_seconds, operation_id)
    WHERE state IN (1, 2);
"#;

/// Returns the complete WhatsApp schema as one immutable initial bundle.
///
/// The bundle remains owned by the integration and contains no Communications
/// tables, cross-owner foreign keys, credentials, or provider session state.
#[must_use]
pub fn whatsapp_storage_bundle_v1() -> StorageBundleV1 {
    StorageBundleV1 {
        major: 1,
        revision: WHATSAPP_STORAGE_BUNDLE_REVISION_V1,
        bundle_id: "whatsapp_state".to_owned(),
        owner_id: "whatsapp".to_owned(),
        steps: vec![StorageMigrationStepV1 {
            revision: WHATSAPP_STORAGE_BUNDLE_REVISION_V1,
            migration_id: "whatsapp_state_initial".to_owned(),
            forward_sql_utf8: WHATSAPP_SCHEMA_V1.as_bytes().to_vec(),
            sha256: Sha256::digest(WHATSAPP_SCHEMA_V1.as_bytes()).to_vec(),
        }],
    }
}

#[cfg(test)]
mod tests {
    use hermes_storage_protocol::validation::validate_storage_bundle;

    use super::*;

    #[test]
    fn bundle_is_valid_and_owned_only_by_whatsapp() {
        let bundle = whatsapp_storage_bundle_v1();

        assert_eq!(bundle.owner_id, "whatsapp");
        assert_eq!(bundle.bundle_id, "whatsapp_state");
        assert_eq!(bundle.revision, WHATSAPP_STORAGE_BUNDLE_REVISION_V1);
        assert_eq!(bundle.steps.len(), 1);
        assert_eq!(validate_storage_bundle(&bundle), Ok(()));
        assert_eq!(
            bundle.steps[0].forward_sql_utf8,
            WHATSAPP_SCHEMA_V1.as_bytes()
        );
        let sql = std::str::from_utf8(&bundle.steps[0].forward_sql_utf8)
            .expect("WhatsApp Storage SQL is UTF-8");
        assert_eq!(sql.matches("CREATE TABLE IF NOT EXISTS ").count(), 3);
        assert_eq!(
            sql.matches("CREATE TABLE IF NOT EXISTS hermes_data.")
                .count(),
            3
        );
        assert!(!sql.contains("hermes_data.communications_"));
        assert!(!sql.contains("REFERENCES hermes_data.communications_"));
    }
}
