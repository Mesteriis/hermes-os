use hermes_storage_protocol::v1::StorageMigrationStepV1;
use sha2::{Digest, Sha256};

pub const TELEGRAM_CALLS_STORAGE_REVISION_V1: u32 = 3;
pub const TELEGRAM_CALLS_STORAGE_REVISION_V2: u32 = 4;
pub const TELEGRAM_CALLS_STORAGE_REVISION_V3: u32 = 5;

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

pub const TELEGRAM_CALLS_SCHEMA_V2: &str = r#"
CREATE TABLE IF NOT EXISTS hermes_data.telegram_call_operations (
    operation_id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL REFERENCES hermes_data.telegram_accounts(account_id),
    call_session_id TEXT NOT NULL,
    operation_kind TEXT NOT NULL CHECK (operation_kind IN (
        'initiate_audio', 'accept_audio', 'decline', 'end', 'set_local_mute'
    )),
    operation_state TEXT NOT NULL CHECK (operation_state IN (
        'accepted', 'dispatching', 'awaiting_provider', 'completed', 'failed'
    )),
    request_fingerprint_sha256 BYTEA NOT NULL
        CHECK (octet_length(request_fingerprint_sha256) = 32),
    provider_user_id TEXT NULL,
    requested_mute BOOLEAN NULL,
    runtime_generation BIGINT NOT NULL CHECK (runtime_generation > 0),
    grant_epoch BIGINT NOT NULL CHECK (grant_epoch > 0),
    tdlib_call_id INTEGER NULL CHECK (tdlib_call_id IS NULL OR tdlib_call_id > 0),
    revision BIGINT NOT NULL CHECK (revision > 0),
    accepted_at_unix_seconds BIGINT NOT NULL CHECK (accepted_at_unix_seconds > 0),
    updated_at_unix_seconds BIGINT NOT NULL CHECK (updated_at_unix_seconds > 0),
    completed_at_unix_seconds BIGINT NULL
        CHECK (completed_at_unix_seconds IS NULL OR completed_at_unix_seconds > 0),
    failure_category TEXT NULL CHECK (
        failure_category IS NULL OR failure_category IN (
            'network', 'not_available', 'permission', 'protocol', 'unknown'
        )
    ),
    CHECK ((operation_kind = 'initiate_audio') = (provider_user_id IS NOT NULL)),
    CHECK ((operation_kind = 'set_local_mute') = (requested_mute IS NOT NULL)),
    CHECK ((operation_state = 'failed') = (failure_category IS NOT NULL)),
    CHECK (
        (operation_state IN ('completed', 'failed')) =
        (completed_at_unix_seconds IS NOT NULL)
    )
);

CREATE TABLE IF NOT EXISTS hermes_data.telegram_call_local_mute (
    call_session_id TEXT PRIMARY KEY
        REFERENCES hermes_data.telegram_call_sessions(call_session_id),
    account_id TEXT NOT NULL,
    muted BOOLEAN NOT NULL,
    operation_id TEXT NOT NULL
        REFERENCES hermes_data.telegram_call_operations(operation_id),
    updated_at_unix_seconds BIGINT NOT NULL CHECK (updated_at_unix_seconds > 0)
);

CREATE TABLE IF NOT EXISTS hermes_data.telegram_call_operation_history (
    operation_id TEXT NOT NULL
        REFERENCES hermes_data.telegram_call_operations(operation_id),
    revision BIGINT NOT NULL CHECK (revision > 0),
    operation_state TEXT NOT NULL,
    tdlib_call_id INTEGER NULL,
    updated_at_unix_seconds BIGINT NOT NULL CHECK (updated_at_unix_seconds > 0),
    completed_at_unix_seconds BIGINT NULL,
    failure_category TEXT NULL,
    PRIMARY KEY (operation_id, revision)
);

CREATE TABLE IF NOT EXISTS hermes_data.telegram_call_realtime_events (
    event_sequence BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    account_id TEXT NOT NULL,
    event_kind TEXT NOT NULL CHECK (event_kind IN ('call', 'operation')),
    call_session_id TEXT NULL,
    call_revision BIGINT NULL CHECK (call_revision IS NULL OR call_revision > 0),
    operation_id TEXT NULL,
    operation_revision BIGINT NULL
        CHECK (operation_revision IS NULL OR operation_revision > 0),
    local_muted BOOLEAN NOT NULL DEFAULT FALSE,
    observed_at_unix_seconds BIGINT NOT NULL CHECK (observed_at_unix_seconds > 0),
    CHECK (
        (event_kind = 'call' AND call_session_id IS NOT NULL
            AND call_revision IS NOT NULL AND operation_id IS NULL
            AND operation_revision IS NULL)
        OR
        (event_kind = 'operation' AND operation_id IS NOT NULL
            AND operation_revision IS NOT NULL AND call_session_id IS NULL
            AND call_revision IS NULL)
    ),
    UNIQUE (call_session_id, call_revision),
    UNIQUE (operation_id, operation_revision)
);

CREATE UNIQUE INDEX IF NOT EXISTS telegram_call_sessions_one_active_per_account_idx
    ON hermes_data.telegram_call_sessions (account_id)
    WHERE provider_state NOT IN ('discarded', 'error');

CREATE UNIQUE INDEX IF NOT EXISTS telegram_call_operations_one_initiate_per_account_idx
    ON hermes_data.telegram_call_operations (account_id)
    WHERE operation_kind = 'initiate_audio'
      AND operation_state NOT IN ('completed', 'failed');

CREATE INDEX IF NOT EXISTS telegram_call_operations_account_id_idx
    ON hermes_data.telegram_call_operations (account_id, operation_id);

CREATE INDEX IF NOT EXISTS telegram_call_realtime_events_account_sequence_idx
    ON hermes_data.telegram_call_realtime_events (account_id, event_sequence);
"#;

pub const TELEGRAM_CALLS_SCHEMA_V3: &str = r#"
CREATE TABLE IF NOT EXISTS hermes_data.telegram_call_media_projection (
    call_session_id TEXT PRIMARY KEY
        REFERENCES hermes_data.telegram_call_sessions(call_session_id),
    account_id TEXT NOT NULL,
    runtime_generation BIGINT NOT NULL CHECK (runtime_generation > 0),
    provider_revision BIGINT NOT NULL CHECK (provider_revision > 0),
    media_state TEXT NOT NULL CHECK (
        media_state IN ('connecting', 'active', 'reconnecting', 'failed')
    ),
    revision BIGINT NOT NULL CHECK (revision > 0),
    connected_at_unix_seconds BIGINT NULL CHECK (
        connected_at_unix_seconds IS NULL OR connected_at_unix_seconds > 0
    ),
    updated_at_unix_seconds BIGINT NOT NULL CHECK (updated_at_unix_seconds > 0),
    failed_at_unix_seconds BIGINT NULL CHECK (
        failed_at_unix_seconds IS NULL OR failed_at_unix_seconds > 0
    )
);

CREATE TABLE IF NOT EXISTS hermes_data.telegram_call_media_state_history (
    call_session_id TEXT NOT NULL
        REFERENCES hermes_data.telegram_call_sessions(call_session_id),
    revision BIGINT NOT NULL CHECK (revision > 0),
    runtime_generation BIGINT NOT NULL CHECK (runtime_generation > 0),
    provider_revision BIGINT NOT NULL CHECK (provider_revision > 0),
    media_state TEXT NOT NULL CHECK (
        media_state IN ('connecting', 'active', 'reconnecting', 'failed')
    ),
    observed_at_unix_seconds BIGINT NOT NULL CHECK (observed_at_unix_seconds > 0),
    PRIMARY KEY (call_session_id, revision)
);

CREATE INDEX IF NOT EXISTS telegram_call_media_projection_account_idx
    ON hermes_data.telegram_call_media_projection (account_id, call_session_id);
"#;

pub fn telegram_calls_storage_migration_v1() -> StorageMigrationStepV1 {
    StorageMigrationStepV1 {
        revision: TELEGRAM_CALLS_STORAGE_REVISION_V1,
        migration_id: "telegram_call_history".to_owned(),
        forward_sql_utf8: TELEGRAM_CALLS_SCHEMA_V1.as_bytes().to_vec(),
        sha256: Sha256::digest(TELEGRAM_CALLS_SCHEMA_V1.as_bytes()).to_vec(),
    }
}

pub fn telegram_calls_storage_migration_v2() -> StorageMigrationStepV1 {
    StorageMigrationStepV1 {
        revision: TELEGRAM_CALLS_STORAGE_REVISION_V2,
        migration_id: "telegram_call_signaling".to_owned(),
        forward_sql_utf8: TELEGRAM_CALLS_SCHEMA_V2.as_bytes().to_vec(),
        sha256: Sha256::digest(TELEGRAM_CALLS_SCHEMA_V2.as_bytes()).to_vec(),
    }
}

pub fn telegram_calls_storage_migration_v3() -> StorageMigrationStepV1 {
    StorageMigrationStepV1 {
        revision: TELEGRAM_CALLS_STORAGE_REVISION_V3,
        migration_id: "telegram_call_media_projection".to_owned(),
        forward_sql_utf8: TELEGRAM_CALLS_SCHEMA_V3.as_bytes().to_vec(),
        sha256: Sha256::digest(TELEGRAM_CALLS_SCHEMA_V3.as_bytes()).to_vec(),
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

        let signaling = telegram_calls_storage_migration_v2();
        assert_eq!(signaling.revision, 4);
        assert_eq!(signaling.migration_id, "telegram_call_signaling");
        assert!(TELEGRAM_CALLS_SCHEMA_V2.contains("telegram_call_operations"));
        assert!(TELEGRAM_CALLS_SCHEMA_V2.contains("telegram_call_local_mute"));
        assert!(TELEGRAM_CALLS_SCHEMA_V2.contains("telegram_call_operation_history"));
        assert!(TELEGRAM_CALLS_SCHEMA_V2.contains("telegram_call_realtime_events"));
        assert!(!TELEGRAM_CALLS_SCHEMA_V2.contains("INSERT INTO"));

        let media = telegram_calls_storage_migration_v3();
        assert_eq!(media.revision, 5);
        assert_eq!(media.migration_id, "telegram_call_media_projection");
        assert!(TELEGRAM_CALLS_SCHEMA_V3.contains("telegram_call_media_projection"));
        assert!(TELEGRAM_CALLS_SCHEMA_V3.contains("telegram_call_media_state_history"));
        assert!(!TELEGRAM_CALLS_SCHEMA_V3.contains("INSERT INTO"));
    }
}
