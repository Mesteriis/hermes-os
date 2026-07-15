-- Migration 0087: Telegram provider-write outbox runtime and reconciliation state
-- ADR-0091: provider writes must be durable, retryable and completed only after
-- provider-observed state is recorded.

ALTER TABLE telegram_provider_write_commands
    DROP CONSTRAINT IF EXISTS telegram_provider_write_commands_status;

ALTER TABLE telegram_provider_write_commands
    ADD CONSTRAINT telegram_provider_write_commands_status
        CHECK (status IN (
            'queued',
            'executing',
            'completed',
            'failed',
            'retrying',
            'cancelled',
            'dead_letter'
        ));

ALTER TABLE telegram_provider_write_commands
    ADD COLUMN IF NOT EXISTS next_attempt_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS last_attempt_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS locked_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS locked_by TEXT,
    ADD COLUMN IF NOT EXISTS provider_observed_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS provider_state JSONB NOT NULL DEFAULT '{}'::jsonb,
    ADD COLUMN IF NOT EXISTS reconciliation_status TEXT NOT NULL DEFAULT 'not_observed',
    ADD COLUMN IF NOT EXISTS reconciled_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS dead_lettered_at TIMESTAMPTZ;

ALTER TABLE telegram_provider_write_commands
    DROP CONSTRAINT IF EXISTS telegram_provider_write_commands_provider_state_is_object;

ALTER TABLE telegram_provider_write_commands
    ADD CONSTRAINT telegram_provider_write_commands_provider_state_is_object
        CHECK (jsonb_typeof(provider_state) = 'object');

ALTER TABLE telegram_provider_write_commands
    DROP CONSTRAINT IF EXISTS telegram_provider_write_commands_reconciliation_status;

ALTER TABLE telegram_provider_write_commands
    ADD CONSTRAINT telegram_provider_write_commands_reconciliation_status
        CHECK (reconciliation_status IN (
            'not_observed',
            'awaiting_provider',
            'observed',
            'mismatch',
            'not_required'
        ));

ALTER TABLE telegram_provider_write_commands
    DROP CONSTRAINT IF EXISTS telegram_provider_write_commands_locked_by_not_empty;

ALTER TABLE telegram_provider_write_commands
    ADD CONSTRAINT telegram_provider_write_commands_locked_by_not_empty
        CHECK (locked_by IS NULL OR length(trim(locked_by)) > 0);

CREATE INDEX IF NOT EXISTS telegram_provider_write_commands_due_idx
    ON telegram_provider_write_commands (account_id, status, next_attempt_at, created_at);

CREATE INDEX IF NOT EXISTS telegram_provider_write_commands_reconciliation_idx
    ON telegram_provider_write_commands (account_id, reconciliation_status, updated_at DESC);
