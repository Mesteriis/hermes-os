DO $$
BEGIN
    IF to_regclass('public.communication_contact_sync_runs') IS NOT NULL
       AND to_regclass('public.communication_address_book_sync_runs') IS NULL THEN
        ALTER TABLE communication_contact_sync_runs RENAME TO communication_address_book_sync_runs;
    END IF;
END $$;

DO $$
BEGIN
    IF to_regclass('public.communication_address_book_sync_runs') IS NOT NULL THEN
        IF EXISTS (
            SELECT 1
            FROM information_schema.columns
            WHERE table_schema = 'public'
              AND table_name = 'communication_address_book_sync_runs'
              AND column_name = 'provider_contacts_seen'
        )
        AND NOT EXISTS (
            SELECT 1
            FROM information_schema.columns
            WHERE table_schema = 'public'
              AND table_name = 'communication_address_book_sync_runs'
              AND column_name = 'provider_entries_seen'
        ) THEN
            ALTER TABLE communication_address_book_sync_runs
                RENAME COLUMN provider_contacts_seen TO provider_entries_seen;
            ALTER TABLE communication_address_book_sync_runs
                RENAME COLUMN provider_contacts_upserted TO provider_entries_upserted;
            ALTER TABLE communication_address_book_sync_runs
                RENAME COLUMN provider_contacts_skipped TO provider_entries_skipped;
            ALTER TABLE communication_address_book_sync_runs
                RENAME COLUMN local_contacts_seen TO local_entries_seen;
            ALTER TABLE communication_address_book_sync_runs
                RENAME COLUMN local_contacts_pushed TO local_entries_pushed;
            ALTER TABLE communication_address_book_sync_runs
                RENAME COLUMN local_contacts_blocked TO local_entries_blocked;
        END IF;

        IF EXISTS (
            SELECT 1
            FROM pg_constraint
            WHERE conname = 'communication_contact_sync_runs_status_check'
              AND conrelid = 'public.communication_address_book_sync_runs'::regclass
        ) THEN
            ALTER TABLE communication_address_book_sync_runs
                RENAME CONSTRAINT communication_contact_sync_runs_status_check
                TO communication_address_book_sync_runs_status_check;
        END IF;

        IF EXISTS (
            SELECT 1
            FROM pg_constraint
            WHERE conname = 'communication_contact_sync_runs_trigger_check'
              AND conrelid = 'public.communication_address_book_sync_runs'::regclass
        ) THEN
            ALTER TABLE communication_address_book_sync_runs
                RENAME CONSTRAINT communication_contact_sync_runs_trigger_check
                TO communication_address_book_sync_runs_trigger_check;
        END IF;

        IF EXISTS (
            SELECT 1
            FROM pg_constraint
            WHERE conname = 'communication_contact_sync_runs_counts_non_negative'
              AND conrelid = 'public.communication_address_book_sync_runs'::regclass
        ) THEN
            ALTER TABLE communication_address_book_sync_runs
                RENAME CONSTRAINT communication_contact_sync_runs_counts_non_negative
                TO communication_address_book_sync_runs_counts_non_negative;
        END IF;
    END IF;
END $$;

ALTER INDEX IF EXISTS communication_contact_sync_runs_account_started_idx
    RENAME TO communication_address_book_sync_runs_account_started_idx;
ALTER INDEX IF EXISTS communication_contact_sync_runs_active_idx
    RENAME TO communication_address_book_sync_runs_active_idx;

DO $$
BEGIN
    IF to_regclass('public.communication_provider_contact_links') IS NOT NULL
       AND to_regclass('public.communication_provider_address_book_links') IS NULL THEN
        ALTER TABLE communication_provider_contact_links RENAME TO communication_provider_address_book_links;
    END IF;
END $$;

DO $$
BEGIN
    IF to_regclass('public.communication_provider_address_book_links') IS NOT NULL THEN
        IF EXISTS (
            SELECT 1
            FROM pg_constraint
            WHERE conname = 'communication_provider_contact_links_provider_contact_not_empty'
              AND conrelid = 'public.communication_provider_address_book_links'::regclass
        ) THEN
            ALTER TABLE communication_provider_address_book_links
                RENAME CONSTRAINT communication_provider_contact_links_provider_contact_not_empty
                TO communication_provider_address_book_links_provider_contact_not_empty;
        END IF;

        IF EXISTS (
            SELECT 1
            FROM pg_constraint
            WHERE conname = 'communication_provider_contact_links_sync_state_check'
              AND conrelid = 'public.communication_provider_address_book_links'::regclass
        ) THEN
            ALTER TABLE communication_provider_address_book_links
                RENAME CONSTRAINT communication_provider_contact_links_sync_state_check
                TO communication_provider_address_book_links_sync_state_check;
        END IF;

        IF EXISTS (
            SELECT 1
            FROM pg_constraint
            WHERE conname = 'communication_provider_contact_links_metadata_is_object'
              AND conrelid = 'public.communication_provider_address_book_links'::regclass
        ) THEN
            ALTER TABLE communication_provider_address_book_links
                RENAME CONSTRAINT communication_provider_contact_links_metadata_is_object
                TO communication_provider_address_book_links_metadata_is_object;
        END IF;
    END IF;
END $$;

ALTER INDEX IF EXISTS communication_provider_contact_links_provider_idx
    RENAME TO communication_provider_address_book_links_provider_idx;

UPDATE communication_provider_accounts
SET config = config
    || CASE
        WHEN jsonb_typeof(config->'contacts_sync_enabled') = 'boolean'
        THEN jsonb_build_object('address_book_sync_enabled', config->'contacts_sync_enabled')
        ELSE '{}'::jsonb
    END
    || CASE
        WHEN config ? 'contacts_sync_unsupported_reason'
        THEN jsonb_build_object('address_book_sync_unsupported_reason', config->'contacts_sync_unsupported_reason')
        ELSE '{}'::jsonb
    END
    || CASE
        WHEN config ? 'contacts_sync_direction'
        THEN jsonb_build_object('address_book_sync_direction', config->'contacts_sync_direction')
        ELSE '{}'::jsonb
    END
    || CASE
        WHEN jsonb_typeof(config->'contacts_bidirectional_enabled') = 'boolean'
        THEN jsonb_build_object('address_book_bidirectional_enabled', config->'contacts_bidirectional_enabled')
        ELSE '{}'::jsonb
    END
    || CASE
        WHEN jsonb_typeof(config->'contacts_remote_write_enabled') = 'boolean'
        THEN jsonb_build_object('address_book_remote_write_enabled', config->'contacts_remote_write_enabled')
        ELSE '{}'::jsonb
    END,
    updated_at = now()
WHERE config ?| array[
    'contacts_sync_enabled',
    'contacts_sync_unsupported_reason',
    'contacts_sync_direction',
    'contacts_bidirectional_enabled',
    'contacts_remote_write_enabled'
];
