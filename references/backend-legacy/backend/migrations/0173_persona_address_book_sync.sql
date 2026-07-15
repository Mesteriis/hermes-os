ALTER TABLE persons
    ADD COLUMN IF NOT EXISTS is_address_book BOOLEAN NOT NULL DEFAULT false;

CREATE INDEX IF NOT EXISTS persons_is_address_book_idx
    ON persons (updated_at DESC, person_id)
    WHERE is_address_book = true;

CREATE TABLE IF NOT EXISTS communication_address_book_sync_runs (
    run_id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL REFERENCES communication_provider_accounts(account_id) ON DELETE RESTRICT,
    status TEXT NOT NULL,
    trigger TEXT NOT NULL,
    started_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at TIMESTAMPTZ,
    provider_entries_seen INTEGER NOT NULL DEFAULT 0,
    provider_entries_upserted INTEGER NOT NULL DEFAULT 0,
    provider_entries_skipped INTEGER NOT NULL DEFAULT 0,
    local_entries_seen INTEGER NOT NULL DEFAULT 0,
    local_entries_pushed INTEGER NOT NULL DEFAULT 0,
    local_entries_blocked INTEGER NOT NULL DEFAULT 0,
    error_code TEXT,
    error_message TEXT,
    next_run_at TIMESTAMPTZ,

    CONSTRAINT communication_address_book_sync_runs_status_check
        CHECK (status IN ('running', 'completed', 'skipped', 'failed')),
    CONSTRAINT communication_address_book_sync_runs_trigger_check
        CHECK (trigger IN ('scheduled', 'manual')),
    CONSTRAINT communication_address_book_sync_runs_counts_non_negative CHECK (
        provider_entries_seen >= 0
        AND provider_entries_upserted >= 0
        AND provider_entries_skipped >= 0
        AND local_entries_seen >= 0
        AND local_entries_pushed >= 0
        AND local_entries_blocked >= 0
    )
);

CREATE INDEX IF NOT EXISTS communication_address_book_sync_runs_account_started_idx
    ON communication_address_book_sync_runs (account_id, started_at DESC);

CREATE INDEX IF NOT EXISTS communication_address_book_sync_runs_active_idx
    ON communication_address_book_sync_runs (account_id, status)
    WHERE status = 'running';

CREATE TABLE IF NOT EXISTS communication_provider_address_book_links (
    account_id TEXT NOT NULL REFERENCES communication_provider_accounts(account_id) ON DELETE CASCADE,
    person_id TEXT NOT NULL REFERENCES persons(person_id) ON DELETE CASCADE,
    provider_address_book_entry_id TEXT NOT NULL,
    provider_etag TEXT,
    last_provider_seen_at TIMESTAMPTZ,
    last_local_pushed_at TIMESTAMPTZ,
    last_synced_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    sync_state TEXT NOT NULL DEFAULT 'linked',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    PRIMARY KEY (account_id, person_id),
    CONSTRAINT communication_provider_address_book_links_provider_address_book_entry_not_empty
        CHECK (length(trim(provider_address_book_entry_id)) > 0),
    CONSTRAINT communication_provider_address_book_links_sync_state_check
        CHECK (sync_state IN ('linked', 'provider_only', 'local_only', 'conflict', 'blocked')),
    CONSTRAINT communication_provider_address_book_links_metadata_is_object
        CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE UNIQUE INDEX IF NOT EXISTS communication_provider_address_book_links_provider_idx
    ON communication_provider_address_book_links (account_id, provider_address_book_entry_id);

-- `contacts` was renamed to `persons` in migration 0034. Keep this forward
-- cleanup idempotent for databases that still have an abandoned legacy table.
DROP TABLE IF EXISTS contacts;
