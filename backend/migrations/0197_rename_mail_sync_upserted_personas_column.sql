-- Mail sync run counters use Persona terminology in active storage.

DO $$
BEGIN
    IF to_regclass('public.communication_mail_sync_runs') IS NOT NULL
       AND EXISTS (
           SELECT 1
           FROM information_schema.columns
           WHERE table_schema = 'public'
             AND table_name = 'communication_mail_sync_runs'
             AND column_name = 'upserted_persons'
       )
       AND NOT EXISTS (
           SELECT 1
           FROM information_schema.columns
           WHERE table_schema = 'public'
             AND table_name = 'communication_mail_sync_runs'
             AND column_name = 'upserted_personas'
       ) THEN
        ALTER TABLE communication_mail_sync_runs
            RENAME COLUMN upserted_persons TO upserted_personas;
    END IF;
END $$;
