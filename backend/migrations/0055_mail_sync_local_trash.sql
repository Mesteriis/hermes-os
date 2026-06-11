-- ADR-0080: per-account mail sync progress and local-only trash

ALTER TABLE communication_messages
    ADD COLUMN IF NOT EXISTS local_state TEXT NOT NULL DEFAULT 'active',
    ADD COLUMN IF NOT EXISTS local_state_changed_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS local_state_reason TEXT;

ALTER TABLE communication_messages
    DROP CONSTRAINT IF EXISTS communication_messages_local_state_check;

ALTER TABLE communication_messages
    ADD CONSTRAINT communication_messages_local_state_check CHECK (
        local_state IN ('active', 'trash')
    );

CREATE INDEX IF NOT EXISTS communication_messages_local_state_idx
    ON communication_messages (local_state, COALESCE(occurred_at, projected_at) DESC);

CREATE TABLE IF NOT EXISTS communication_account_sync_settings (
    account_id TEXT PRIMARY KEY REFERENCES communication_provider_accounts(account_id) ON DELETE CASCADE,
    sync_enabled BOOLEAN NOT NULL DEFAULT true,
    batch_size INTEGER NOT NULL DEFAULT 5,
    poll_interval_seconds INTEGER NOT NULL DEFAULT 300,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT communication_account_sync_settings_batch_size_check CHECK (batch_size BETWEEN 1 AND 500),
    CONSTRAINT communication_account_sync_settings_poll_interval_check CHECK (poll_interval_seconds BETWEEN 60 AND 86400)
);

CREATE TABLE IF NOT EXISTS communication_mail_sync_runs (
    run_id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL REFERENCES communication_provider_accounts(account_id) ON DELETE CASCADE,
    trigger TEXT NOT NULL,
    status TEXT NOT NULL,
    phase TEXT NOT NULL,
    progress_mode TEXT NOT NULL DEFAULT 'indeterminate',
    progress_percent INTEGER,
    processed_messages BIGINT NOT NULL DEFAULT 0,
    estimated_total_messages BIGINT,
    current_batch_size INTEGER NOT NULL DEFAULT 0,
    fetched_messages BIGINT NOT NULL DEFAULT 0,
    projected_messages BIGINT NOT NULL DEFAULT 0,
    upserted_persons BIGINT NOT NULL DEFAULT 0,
    upserted_organizations BIGINT NOT NULL DEFAULT 0,
    checkpoint_before JSONB,
    checkpoint_after JSONB,
    checkpoint_saved BOOLEAN NOT NULL DEFAULT false,
    error_code TEXT,
    error_message TEXT,
    started_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at TIMESTAMPTZ,
    next_run_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT communication_mail_sync_runs_trigger_check CHECK (trigger IN ('scheduled', 'manual')),
    CONSTRAINT communication_mail_sync_runs_status_check CHECK (status IN ('queued', 'running', 'completed', 'failed', 'skipped', 'recoverable_full_resync_needed')),
    CONSTRAINT communication_mail_sync_runs_phase_check CHECK (phase IN ('idle', 'waiting_for_vault', 'listing', 'fetching', 'projecting', 'persons_graph', 'completed', 'failed', 'skipped')),
    CONSTRAINT communication_mail_sync_runs_progress_mode_check CHECK (progress_mode IN ('none', 'determinate', 'indeterminate')),
    CONSTRAINT communication_mail_sync_runs_progress_percent_check CHECK (progress_percent IS NULL OR (progress_percent >= 0 AND progress_percent <= 100)),
    CONSTRAINT communication_mail_sync_runs_checkpoint_before_is_object CHECK (checkpoint_before IS NULL OR jsonb_typeof(checkpoint_before) = 'object'),
    CONSTRAINT communication_mail_sync_runs_checkpoint_after_is_object CHECK (checkpoint_after IS NULL OR jsonb_typeof(checkpoint_after) = 'object')
);

CREATE INDEX IF NOT EXISTS communication_mail_sync_runs_account_started_idx
    ON communication_mail_sync_runs (account_id, started_at DESC);

CREATE UNIQUE INDEX IF NOT EXISTS communication_mail_sync_runs_active_account_idx
    ON communication_mail_sync_runs (account_id)
    WHERE status IN ('queued', 'running', 'recoverable_full_resync_needed');

CREATE TABLE IF NOT EXISTS communication_message_participants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    message_id TEXT NOT NULL REFERENCES communication_messages(message_id) ON DELETE CASCADE,
    person_id TEXT NOT NULL REFERENCES persons(person_id) ON DELETE CASCADE,
    email_address TEXT NOT NULL,
    display_name TEXT,
    role TEXT NOT NULL,
    source TEXT NOT NULL DEFAULT 'email_sync',
    confidence REAL NOT NULL DEFAULT 1.0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT communication_message_participants_role_check CHECK (role IN ('sender', 'recipient', 'cc', 'bcc')),
    CONSTRAINT communication_message_participants_email_not_empty CHECK (length(trim(email_address)) > 0),
    CONSTRAINT communication_message_participants_confidence_check CHECK (confidence >= 0 AND confidence <= 1),
    CONSTRAINT communication_message_participants_unique UNIQUE (message_id, person_id, role, email_address)
);

CREATE INDEX IF NOT EXISTS communication_message_participants_message_idx
    ON communication_message_participants (message_id);

CREATE INDEX IF NOT EXISTS communication_message_participants_person_idx
    ON communication_message_participants (person_id);
