use hermes_storage_protocol::v1::StorageMigrationStepV1;
use sha2::{Digest, Sha256};

pub const TELEGRAM_CALLS_STORAGE_REVISION_V1: u32 = 3;

pub const TELEGRAM_CALLS_SCHEMA_V1: &str = r#"
CREATE TABLE IF NOT EXISTS hermes_data.telegram_call_sessions (
    call_session_id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL REFERENCES hermes_data.telegram_accounts(account_id),
    runtime_generation BIGINT NOT NULL CHECK (runtime_generation > 0),
    tdlib_call_id INTEGER NOT NULL CHECK (tdlib_call_id > 0),
    provider_call_unique_id BIGINT NULL CHECK (provider_call_unique_id IS NULL OR provider_call_unique_id > 0),
    provider_user_id TEXT NOT NULL,
    direction TEXT NOT NULL CHECK (direction IN ('incoming', 'outgoing')),
    provider_state TEXT NOT NULL CHECK (provider_state IN ('pending', 'exchanging_keys', 'media_ready', 'hanging_up', 'discarded', 'error')),
    pending_created BOOLEAN NOT NULL,
    pending_received BOOLEAN NOT NULL,
    discard_reason TEXT NULL CHECK (discard_reason IS NULL OR discard_reason IN ('empty', 'missed', 'declined', 'disconnected', 'hung_up')),
    failure_category TEXT NULL CHECK (failure_category IS NULL OR failure_category IN ('network', 'not_available', 'permission', 'protocol', 'unknown')),
    revision BIGINT NOT NULL CHECK (revision > 0),
    created_at_unix_seconds BIGINT NOT NULL CHECK (created_at_unix_seconds > 0),
    updated_at_unix_seconds BIGINT NOT NULL CHECK (updated_at_unix_seconds > 0),
    ended_at_unix_seconds BIGINT NULL CHECK (ended_at_unix_seconds IS NULL OR ended_at_unix_seconds > 0),
    UNIQUE (account_id, runtime_generation, tdlib_call_id),
    UNIQUE (account_id, provider_call_unique_id)
);

CREATE TABLE IF NOT EXISTS hermes_data.telegram_call_state_history (
    call_session_id TEXT NOT NULL REFERENCES hermes_data.telegram_call_sessions(call_session_id),
    revision BIGINT NOT NULL CHECK (revision > 0),
    provider_state TEXT NOT NULL,
    pending_created BOOLEAN NOT NULL,
    pending_received BOOLEAN NOT NULL,
    discard_reason TEXT NULL,
    failure_category TEXT NULL,
    observed_at_unix_seconds BIGINT NOT NULL CHECK (observed_at_unix_seconds > 0),
    PRIMARY KEY (call_session_id, revision)
);

CREATE TABLE IF NOT EXISTS hermes_data.telegram_call_realtime_frames (
    frame_sequence BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    account_id TEXT NOT NULL,
    call_session_id TEXT NOT NULL REFERENCES hermes_data.telegram_call_sessions(call_session_id),
    call_revision BIGINT NOT NULL CHECK (call_revision > 0),
    provider_state TEXT NOT NULL,
    pending_created BOOLEAN NOT NULL,
    pending_received BOOLEAN NOT NULL,
    discard_reason TEXT NULL,
    failure_category TEXT NULL,
    observed_at_unix_seconds BIGINT NOT NULL CHECK (observed_at_unix_seconds > 0),
    UNIQUE (call_session_id, call_revision)
);

CREATE INDEX IF NOT EXISTS telegram_call_sessions_account_idx
    ON hermes_data.telegram_call_sessions (account_id, call_session_id);

CREATE INDEX IF NOT EXISTS telegram_call_realtime_account_sequence_idx
    ON hermes_data.telegram_call_realtime_frames (account_id, frame_sequence);
"#;

pub fn telegram_calls_storage_migration_v1() -> StorageMigrationStepV1 {
    StorageMigrationStepV1 {
        revision: TELEGRAM_CALLS_STORAGE_REVISION_V1,
        migration_id: "telegram_call_history".to_owned(),
        forward_sql_utf8: TELEGRAM_CALLS_SCHEMA_V1.as_bytes().to_vec(),
        sha256: Sha256::digest(TELEGRAM_CALLS_SCHEMA_V1.as_bytes()).to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migration_is_revisioned_and_owner_local() {
        let migration = telegram_calls_storage_migration_v1();

        assert_eq!(migration.revision, 3);
        assert_eq!(migration.migration_id, "telegram_call_history");
        assert!(TELEGRAM_CALLS_SCHEMA_V1.contains("hermes_data.telegram_call_sessions"));
        assert!(TELEGRAM_CALLS_SCHEMA_V1.contains("telegram_call_realtime_frames"));
    }
}
