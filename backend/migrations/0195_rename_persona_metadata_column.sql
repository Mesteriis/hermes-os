-- Persona metadata naming alignment.
--
-- `person_id` remains the stable storage identifier in this migration slice,
-- but the JSON metadata column now follows Persona domain terminology.

DO $$
BEGIN
    IF to_regclass('public.personas') IS NOT NULL
       AND EXISTS (
           SELECT 1
           FROM information_schema.columns
           WHERE table_schema = 'public'
             AND table_name = 'personas'
             AND column_name = 'person_metadata'
       )
       AND NOT EXISTS (
           SELECT 1
           FROM information_schema.columns
           WHERE table_schema = 'public'
             AND table_name = 'personas'
             AND column_name = 'persona_metadata'
       ) THEN
        ALTER TABLE personas
            RENAME COLUMN person_metadata TO persona_metadata;
    END IF;

    IF to_regclass('public.personas') IS NOT NULL
       AND EXISTS (
           SELECT 1
           FROM pg_constraint
           WHERE conrelid = 'public.personas'::regclass
             AND conname = 'personas_person_metadata_is_object'
       ) THEN
        ALTER TABLE personas
            RENAME CONSTRAINT personas_person_metadata_is_object TO personas_persona_metadata_is_object;
    END IF;

    IF to_regclass('public.personas') IS NOT NULL
       AND EXISTS (
           SELECT 1
           FROM pg_constraint
           WHERE conrelid = 'public.personas'::regclass
             AND conname = 'persons_person_metadata_is_object'
       ) THEN
        ALTER TABLE personas
            RENAME CONSTRAINT persons_person_metadata_is_object TO personas_persona_metadata_is_object;
    END IF;
END $$;
