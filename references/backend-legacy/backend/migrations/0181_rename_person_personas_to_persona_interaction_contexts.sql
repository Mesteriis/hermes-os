-- Persona interaction-context storage naming alignment.
--
-- `person_personas` came from the legacy "person has personas" model. Persona is
-- now the root entity, and these rows represent named interaction contexts.

DO $$
BEGIN
    IF to_regclass('public.persona_interaction_contexts') IS NULL
       AND to_regclass('public.person_personas') IS NOT NULL THEN
        ALTER TABLE person_personas RENAME TO persona_interaction_contexts;
    END IF;
END $$;

ALTER INDEX IF EXISTS person_personas_pkey
    RENAME TO persona_interaction_contexts_pkey;
ALTER INDEX IF EXISTS person_personas_person_id_idx
    RENAME TO persona_interaction_contexts_person_id_idx;

DO $$
BEGIN
    IF to_regclass('public.persona_interaction_contexts') IS NOT NULL THEN
        IF EXISTS (
            SELECT 1 FROM pg_constraint
            WHERE conrelid = 'public.persona_interaction_contexts'::regclass
              AND conname = 'person_personas_name_not_empty'
        ) THEN
            ALTER TABLE persona_interaction_contexts
                RENAME CONSTRAINT person_personas_name_not_empty
                TO persona_interaction_contexts_name_not_empty;
        END IF;

        IF EXISTS (
            SELECT 1 FROM pg_constraint
            WHERE conrelid = 'public.persona_interaction_contexts'::regclass
              AND conname = 'person_personas_metadata_is_object'
        ) THEN
            ALTER TABLE persona_interaction_contexts
                RENAME CONSTRAINT person_personas_metadata_is_object
                TO persona_interaction_contexts_metadata_is_object;
        END IF;
    END IF;
END $$;

UPDATE person_preferences
SET source = regexp_replace(source, '^person_personas:', 'persona_interaction_contexts:')
WHERE source LIKE 'person_personas:%';
