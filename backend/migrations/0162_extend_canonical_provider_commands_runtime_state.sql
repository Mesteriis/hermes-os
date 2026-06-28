ALTER TABLE communication_provider_commands
    ADD COLUMN IF NOT EXISTS provider_state JSONB NOT NULL DEFAULT '{}'::jsonb,
    ADD COLUMN IF NOT EXISTS reconciliation_status TEXT NOT NULL DEFAULT 'not_observed',
    ADD COLUMN IF NOT EXISTS next_attempt_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS last_attempt_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS provider_observed_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS reconciled_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS dead_lettered_at TIMESTAMPTZ;

ALTER TABLE communication_provider_commands
    ADD CONSTRAINT communication_provider_commands_provider_state_is_object
        CHECK (jsonb_typeof(provider_state) = 'object'),
    ADD CONSTRAINT communication_provider_commands_reconciliation_status_not_empty
        CHECK (length(trim(reconciliation_status)) > 0);
