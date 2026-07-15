-- Migration 0157: WhatsApp provider-write command outbox foundation
-- ADR-0101: WhatsApp provider writes must be durable, capability-gated and
-- completed only after provider-observed reconciliation.

CREATE TABLE IF NOT EXISTS whatsapp_provider_write_commands (
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
    next_attempt_at         TIMESTAMPTZ,
    last_attempt_at         TIMESTAMPTZ,
    locked_at               TIMESTAMPTZ,
    locked_by               TEXT,
    provider_observed_at    TIMESTAMPTZ,
    provider_state          JSONB NOT NULL DEFAULT '{}'::jsonb,
    reconciliation_status   TEXT NOT NULL DEFAULT 'not_observed',
    reconciled_at           TIMESTAMPTZ,
    dead_lettered_at        TIMESTAMPTZ,
    completed_at            TIMESTAMPTZ,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT whatsapp_provider_write_commands_command_id_not_empty
        CHECK (length(trim(command_id)) > 0),
    CONSTRAINT whatsapp_provider_write_commands_idempotency_key_not_empty
        CHECK (length(trim(idempotency_key)) > 0),
    CONSTRAINT whatsapp_provider_write_commands_provider_chat_id_not_empty
        CHECK (length(trim(provider_chat_id)) > 0),
    CONSTRAINT whatsapp_provider_write_commands_actor_not_empty
        CHECK (length(trim(actor_id)) > 0),
    CONSTRAINT whatsapp_provider_write_commands_command_kind
        CHECK (command_kind IN (
            'download_media',
            'send_text',
            'send_media',
            'send_voice_note',
            'reply',
            'forward',
            'edit',
            'delete',
            'react',
            'unreact',
            'mark_read',
            'mark_unread',
            'archive',
            'unarchive',
            'mute',
            'unmute',
            'pin',
            'unpin',
            'join_group',
            'leave_group',
            'publish_status'
        )),
    CONSTRAINT whatsapp_provider_write_commands_capability_state
        CHECK (capability_state IN ('available', 'blocked', 'degraded', 'unsupported')),
    CONSTRAINT whatsapp_provider_write_commands_action_class
        CHECK (action_class IN ('read', 'local_write', 'provider_write', 'destructive', 'export', 'secret_access', 'automation')),
    CONSTRAINT whatsapp_provider_write_commands_confirmation_decision
        CHECK (confirmation_decision IN ('pending', 'confirmed', 'rejected', 'not_required')),
    CONSTRAINT whatsapp_provider_write_commands_status
        CHECK (status IN (
            'queued',
            'confirmed',
            'executing',
            'retrying',
            'completed',
            'failed',
            'dead_letter',
            'cancelled'
        )),
    CONSTRAINT whatsapp_provider_write_commands_retry_count_non_negative
        CHECK (retry_count >= 0),
    CONSTRAINT whatsapp_provider_write_commands_max_retries_positive
        CHECK (max_retries > 0),
    CONSTRAINT whatsapp_provider_write_commands_target_ref_is_object
        CHECK (jsonb_typeof(target_ref) = 'object'),
    CONSTRAINT whatsapp_provider_write_commands_payload_is_object
        CHECK (jsonb_typeof(payload) = 'object'),
    CONSTRAINT whatsapp_provider_write_commands_result_payload_is_object
        CHECK (jsonb_typeof(result_payload) = 'object'),
    CONSTRAINT whatsapp_provider_write_commands_audit_metadata_is_object
        CHECK (jsonb_typeof(audit_metadata) = 'object'),
    CONSTRAINT whatsapp_provider_write_commands_provider_state_is_object
        CHECK (jsonb_typeof(provider_state) = 'object'),
    CONSTRAINT whatsapp_provider_write_commands_reconciliation_status
        CHECK (reconciliation_status IN (
            'not_observed',
            'awaiting_provider',
            'observed',
            'mismatch',
            'not_required'
        )),
    CONSTRAINT whatsapp_provider_write_commands_locked_by_not_empty
        CHECK (locked_by IS NULL OR length(trim(locked_by)) > 0),
    CONSTRAINT whatsapp_provider_write_commands_idempotency_unique
        UNIQUE (account_id, idempotency_key)
);

CREATE INDEX IF NOT EXISTS whatsapp_provider_write_commands_account_idx
    ON whatsapp_provider_write_commands (account_id, status, created_at DESC);

CREATE INDEX IF NOT EXISTS whatsapp_provider_write_commands_chat_idx
    ON whatsapp_provider_write_commands (account_id, provider_chat_id, created_at DESC);

CREATE INDEX IF NOT EXISTS whatsapp_provider_write_commands_idempotency_idx
    ON whatsapp_provider_write_commands (idempotency_key);

CREATE INDEX IF NOT EXISTS whatsapp_provider_write_commands_due_idx
    ON whatsapp_provider_write_commands (account_id, status, next_attempt_at, created_at);

CREATE INDEX IF NOT EXISTS whatsapp_provider_write_commands_reconciliation_idx
    ON whatsapp_provider_write_commands (account_id, reconciliation_status, updated_at DESC);

INSERT INTO communication_accounts (
    account_id, provider_kind, display_name, external_account_id, config, metadata, created_at, updated_at
)
SELECT
    account_id,
    provider_kind,
    display_name,
    external_account_id,
    config,
    jsonb_build_object('source_table', 'communication_provider_accounts'),
    created_at,
    updated_at
FROM communication_provider_accounts
WHERE provider_kind = 'whatsapp_web'
ON CONFLICT (account_id) DO NOTHING;

INSERT INTO communication_provider_commands (
    command_id, account_id, channel_kind, command_kind, idempotency_key,
    provider_conversation_id, provider_message_id, target_ref, payload, capability_state,
    action_class, confirmation_decision, status, retry_count, max_retries, last_error,
    result_payload, audit_metadata, actor_id, happened_at, completed_at, created_at, updated_at
)
SELECT
    command_id,
    account_id,
    'whatsapp',
    command_kind,
    idempotency_key,
    provider_chat_id,
    provider_message_id,
    target_ref,
    payload,
    capability_state,
    action_class,
    confirmation_decision,
    status,
    retry_count,
    max_retries,
    last_error,
    result_payload,
    audit_metadata,
    actor_id,
    happened_at,
    completed_at,
    created_at,
    updated_at
FROM whatsapp_provider_write_commands
ON CONFLICT (command_id) DO NOTHING;
