-- Migration 0082: Telegram message lifecycle schema
-- ADR-0091: version history, tombstones and provider-write command model

-- ---------------------------------------------------------------------------
-- telegram_message_versions — append-only observed edit version history
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS telegram_message_versions (
    version_id          TEXT PRIMARY KEY,
    message_id          TEXT NOT NULL,
    account_id          TEXT NOT NULL
        REFERENCES communication_provider_accounts(account_id) ON DELETE RESTRICT,
    provider_message_id TEXT NOT NULL,
    provider_chat_id    TEXT NOT NULL,
    version_number      INTEGER NOT NULL,
    body_text           TEXT,
    edit_timestamp      TIMESTAMPTZ NOT NULL,
    source_event        TEXT,
    raw_diff_payload    JSONB NOT NULL DEFAULT '{}'::jsonb,
    provenance          JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT telegram_message_versions_message_id_not_empty
        CHECK (length(trim(message_id)) > 0),
    CONSTRAINT telegram_message_versions_provider_message_id_not_empty
        CHECK (length(trim(provider_message_id)) > 0),
    CONSTRAINT telegram_message_versions_provider_chat_id_not_empty
        CHECK (length(trim(provider_chat_id)) > 0),
    CONSTRAINT telegram_message_versions_version_positive
        CHECK (version_number > 0),
    CONSTRAINT telegram_message_versions_raw_diff_is_object
        CHECK (jsonb_typeof(raw_diff_payload) = 'object'),
    CONSTRAINT telegram_message_versions_provenance_is_object
        CHECK (jsonb_typeof(provenance) = 'object'),
    CONSTRAINT telegram_message_versions_message_version_unique
        UNIQUE (message_id, version_number)
);

CREATE INDEX IF NOT EXISTS telegram_message_versions_message_idx
    ON telegram_message_versions (message_id, version_number DESC);

CREATE INDEX IF NOT EXISTS telegram_message_versions_account_provider_idx
    ON telegram_message_versions (account_id, provider_chat_id, provider_message_id);

-- ---------------------------------------------------------------------------
-- telegram_message_tombstones — local visibility and delete evidence
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS telegram_message_tombstones (
    tombstone_id        TEXT PRIMARY KEY,
    message_id          TEXT NOT NULL,
    account_id          TEXT NOT NULL
        REFERENCES communication_provider_accounts(account_id) ON DELETE RESTRICT,
    provider_message_id TEXT NOT NULL,
    provider_chat_id    TEXT NOT NULL,
    reason_class        TEXT NOT NULL,
    actor_class         TEXT NOT NULL,
    observed_at         TIMESTAMPTZ NOT NULL,
    source_event        TEXT,
    is_provider_delete  BOOLEAN NOT NULL DEFAULT false,
    is_local_visible    BOOLEAN NOT NULL DEFAULT true,
    metadata            JSONB NOT NULL DEFAULT '{}'::jsonb,
    provenance          JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT telegram_message_tombstones_message_id_not_empty
        CHECK (length(trim(message_id)) > 0),
    CONSTRAINT telegram_message_tombstones_provider_message_id_not_empty
        CHECK (length(trim(provider_message_id)) > 0),
    CONSTRAINT telegram_message_tombstones_provider_chat_id_not_empty
        CHECK (length(trim(provider_chat_id)) > 0),
    CONSTRAINT telegram_message_tombstones_reason_class
        CHECK (reason_class IN (
            'deleted_by_owner',
            'deleted_by_counterparty',
            'deleted_by_provider',
            'moderation_removed',
            'account_removed',
            'retention_policy',
            'unknown'
        )),
    CONSTRAINT telegram_message_tombstones_actor_class
        CHECK (actor_class IN (
            'owner',
            'provider',
            'automation',
            'system',
            'unknown'
        )),
    CONSTRAINT telegram_message_tombstones_metadata_is_object
        CHECK (jsonb_typeof(metadata) = 'object'),
    CONSTRAINT telegram_message_tombstones_provenance_is_object
        CHECK (jsonb_typeof(provenance) = 'object')
);

CREATE INDEX IF NOT EXISTS telegram_message_tombstones_message_idx
    ON telegram_message_tombstones (message_id, created_at DESC);

CREATE INDEX IF NOT EXISTS telegram_message_tombstones_account_idx
    ON telegram_message_tombstones (account_id, provider_chat_id, created_at DESC);

-- ---------------------------------------------------------------------------
-- telegram_provider_write_commands — durable provider-write command model
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS telegram_provider_write_commands (
    command_id              TEXT PRIMARY KEY,
    account_id              TEXT NOT NULL
        REFERENCES communication_provider_accounts(account_id) ON DELETE RESTRICT,
    command_kind            TEXT NOT NULL,
    idempotency_key         TEXT NOT NULL,
    provider_chat_id        TEXT NOT NULL,
    provider_message_id     TEXT,
    target_ref              JSONB NOT NULL DEFAULT '{}'::jsonb,
    payload                 JSONB NOT NULL DEFAULT '{}'::jsonb,
    capability_state        TEXT NOT NULL,
    action_class            TEXT NOT NULL,
    confirmation_decision   TEXT NOT NULL DEFAULT 'pending',
    status                  TEXT NOT NULL DEFAULT 'queued',
    retry_count             INTEGER NOT NULL DEFAULT 0,
    max_retries             INTEGER NOT NULL DEFAULT 3,
    last_error              TEXT,
    result_payload          JSONB NOT NULL DEFAULT '{}'::jsonb,
    audit_metadata          JSONB NOT NULL DEFAULT '{}'::jsonb,
    actor_id                TEXT NOT NULL,
    happened_at             TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at            TIMESTAMPTZ,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT telegram_provider_write_commands_command_id_not_empty
        CHECK (length(trim(command_id)) > 0),
    CONSTRAINT telegram_provider_write_commands_idempotency_key_not_empty
        CHECK (length(trim(idempotency_key)) > 0),
    CONSTRAINT telegram_provider_write_commands_provider_chat_id_not_empty
        CHECK (length(trim(provider_chat_id)) > 0),
    CONSTRAINT telegram_provider_write_commands_actor_not_empty
        CHECK (length(trim(actor_id)) > 0),
    CONSTRAINT telegram_provider_write_commands_command_kind
        CHECK (command_kind IN (
            'send_text',
            'send_media',
            'edit',
            'delete',
            'restore_visibility',
            'mark_read',
            'pin',
            'unpin',
            'archive',
            'unarchive',
            'mute',
            'unmute',
            'react',
            'unreact',
            'reply',
            'forward',
            'join',
            'leave',
            'admin_action'
        )),
    CONSTRAINT telegram_provider_write_commands_capability_state
        CHECK (capability_state IN ('available', 'blocked', 'degraded', 'unsupported')),
    CONSTRAINT telegram_provider_write_commands_action_class
        CHECK (action_class IN ('read', 'local_write', 'provider_write', 'destructive', 'export', 'secret_access', 'automation')),
    CONSTRAINT telegram_provider_write_commands_confirmation_decision
        CHECK (confirmation_decision IN ('pending', 'confirmed', 'rejected', 'not_required')),
    CONSTRAINT telegram_provider_write_commands_status
        CHECK (status IN ('queued', 'executing', 'completed', 'failed', 'retrying', 'cancelled')),
    CONSTRAINT telegram_provider_write_commands_retry_count_non_negative
        CHECK (retry_count >= 0),
    CONSTRAINT telegram_provider_write_commands_max_retries_positive
        CHECK (max_retries > 0),
    CONSTRAINT telegram_provider_write_commands_target_ref_is_object
        CHECK (jsonb_typeof(target_ref) = 'object'),
    CONSTRAINT telegram_provider_write_commands_payload_is_object
        CHECK (jsonb_typeof(payload) = 'object'),
    CONSTRAINT telegram_provider_write_commands_result_payload_is_object
        CHECK (jsonb_typeof(result_payload) = 'object'),
    CONSTRAINT telegram_provider_write_commands_audit_metadata_is_object
        CHECK (jsonb_typeof(audit_metadata) = 'object'),
    CONSTRAINT telegram_provider_write_commands_idempotency_unique
        UNIQUE (account_id, idempotency_key)
);

CREATE INDEX IF NOT EXISTS telegram_provider_write_commands_account_idx
    ON telegram_provider_write_commands (account_id, status, created_at DESC);

CREATE INDEX IF NOT EXISTS telegram_provider_write_commands_chat_idx
    ON telegram_provider_write_commands (account_id, provider_chat_id, created_at DESC);

CREATE INDEX IF NOT EXISTS telegram_provider_write_commands_idempotency_idx
    ON telegram_provider_write_commands (idempotency_key);
