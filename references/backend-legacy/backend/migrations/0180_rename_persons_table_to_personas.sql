-- Persona storage naming alignment.
--
-- Runtime code now uses the Persona domain name. The primary key remains
-- `person_id` in this slice; column/FK renames need a separate migration.

DO $$
BEGIN
    IF to_regclass('public.personas') IS NULL
       AND to_regclass('public.persons') IS NOT NULL THEN
        ALTER TABLE persons RENAME TO personas;
    END IF;
END $$;

ALTER INDEX IF EXISTS persons_email_address_key RENAME TO personas_email_address_key;
ALTER INDEX IF EXISTS persons_trust_score_idx RENAME TO personas_trust_score_idx;
ALTER INDEX IF EXISTS persons_last_interaction_idx RENAME TO personas_last_interaction_idx;
ALTER INDEX IF EXISTS persons_favorite_idx RENAME TO personas_favorite_idx;
ALTER INDEX IF EXISTS persons_watchlist_idx RENAME TO personas_watchlist_idx;
ALTER INDEX IF EXISTS persons_person_type_idx RENAME TO personas_person_type_idx;
ALTER INDEX IF EXISTS persons_single_self_idx RENAME TO personas_single_self_idx;
ALTER INDEX IF EXISTS persons_is_address_book_idx RENAME TO personas_is_address_book_idx;

DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conrelid = 'public.personas'::regclass
          AND conname = 'persons_display_name_not_empty'
    ) THEN
        ALTER TABLE personas
            RENAME CONSTRAINT persons_display_name_not_empty TO personas_display_name_not_empty;
    END IF;

    IF EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conrelid = 'public.personas'::regclass
          AND conname = 'persons_email_not_empty'
    ) THEN
        ALTER TABLE personas
            RENAME CONSTRAINT persons_email_not_empty TO personas_email_not_empty;
    END IF;

    IF EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conrelid = 'public.personas'::regclass
          AND conname = 'persons_pkey'
    ) THEN
        ALTER TABLE personas
            RENAME CONSTRAINT persons_pkey TO personas_pkey;
    END IF;

    IF EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conrelid = 'public.personas'::regclass
          AND conname = 'persons_trust_score_range'
    ) THEN
        ALTER TABLE personas
            RENAME CONSTRAINT persons_trust_score_range TO personas_trust_score_range;
    END IF;

    IF EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conrelid = 'public.personas'::regclass
          AND conname = 'persons_person_metadata_is_object'
    ) THEN
        ALTER TABLE personas
            RENAME CONSTRAINT persons_person_metadata_is_object TO personas_person_metadata_is_object;
    END IF;

    IF EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conrelid = 'public.personas'::regclass
          AND conname = 'persons_health_status_check'
    ) THEN
        ALTER TABLE personas
            RENAME CONSTRAINT persons_health_status_check TO personas_health_status_check;
    END IF;

    IF EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conrelid = 'public.personas'::regclass
          AND conname = 'persons_person_type_check'
    ) THEN
        ALTER TABLE personas
            RENAME CONSTRAINT persons_person_type_check TO personas_person_type_check;
    END IF;
END $$;
