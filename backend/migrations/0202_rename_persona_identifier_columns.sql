-- Persona identifier storage alignment for the tables enumerated below.
--
-- Earlier migrations moved table/API names from contacts/persons to Personas
-- while keeping several physical `person_id` columns for compatibility. Current
-- Persona storage uses `persona_id` columns directly.

DO $$
BEGIN
    IF to_regclass('public.personas') IS NOT NULL
       AND EXISTS (
           SELECT 1 FROM information_schema.columns
           WHERE table_schema = 'public'
             AND table_name = 'personas'
             AND column_name = 'person_id'
       )
       AND NOT EXISTS (
           SELECT 1 FROM information_schema.columns
           WHERE table_schema = 'public'
             AND table_name = 'personas'
             AND column_name = 'persona_id'
       ) THEN
        ALTER TABLE personas RENAME COLUMN person_id TO persona_id;
    END IF;
END $$;

DO $$
DECLARE
    target_table text;
BEGIN
    FOREACH target_table IN ARRAY ARRAY[
        'persona_identities',
        'persona_roles',
        'persona_facts',
        'persona_memory_cards',
        'persona_preferences',
        'persona_snapshots',
        'persona_knowledge_conflicts',
        'relationship_events',
        'enrichment_results',
        'persona_expertise',
        'persona_promises',
        'persona_risks',
        'organization_persona_links',
        'communication_message_participants',
        'communication_provider_address_book_links',
        'communication_conversation_participants',
        'event_participants'
    ]
    LOOP
        IF to_regclass(format('public.%I', target_table)) IS NOT NULL
           AND EXISTS (
               SELECT 1 FROM information_schema.columns
               WHERE table_schema = 'public'
                 AND table_name = target_table
                 AND column_name = 'person_id'
           )
           AND NOT EXISTS (
               SELECT 1 FROM information_schema.columns
               WHERE table_schema = 'public'
                 AND table_name = target_table
                 AND column_name = 'persona_id'
           ) THEN
            EXECUTE format('ALTER TABLE %I RENAME COLUMN person_id TO persona_id', target_table);
        END IF;
    END LOOP;
END $$;

DO $$
BEGIN
    IF to_regclass('public.persona_identity_candidates') IS NOT NULL THEN
        IF EXISTS (
            SELECT 1 FROM information_schema.columns
            WHERE table_schema = 'public'
              AND table_name = 'persona_identity_candidates'
              AND column_name = 'left_person_id'
        )
        AND NOT EXISTS (
            SELECT 1 FROM information_schema.columns
            WHERE table_schema = 'public'
              AND table_name = 'persona_identity_candidates'
              AND column_name = 'left_persona_id'
        ) THEN
            ALTER TABLE persona_identity_candidates
                RENAME COLUMN left_person_id TO left_persona_id;
        END IF;

        IF EXISTS (
            SELECT 1 FROM information_schema.columns
            WHERE table_schema = 'public'
              AND table_name = 'persona_identity_candidates'
              AND column_name = 'right_person_id'
        )
        AND NOT EXISTS (
            SELECT 1 FROM information_schema.columns
            WHERE table_schema = 'public'
              AND table_name = 'persona_identity_candidates'
              AND column_name = 'right_persona_id'
        ) THEN
            ALTER TABLE persona_identity_candidates
                RENAME COLUMN right_person_id TO right_persona_id;
        END IF;
    END IF;
END $$;

DO $$
BEGIN
    IF to_regclass('public.persona_expertise') IS NOT NULL
       AND EXISTS (
           SELECT 1 FROM information_schema.columns
           WHERE table_schema = 'public'
             AND table_name = 'persona_expertise'
             AND column_name = 'endorsed_by_person_id'
       )
       AND NOT EXISTS (
           SELECT 1 FROM information_schema.columns
           WHERE table_schema = 'public'
             AND table_name = 'persona_expertise'
             AND column_name = 'endorsed_by_persona_id'
       ) THEN
        ALTER TABLE persona_expertise
            RENAME COLUMN endorsed_by_person_id TO endorsed_by_persona_id;
    END IF;
END $$;

DO $$
BEGIN
    IF to_regclass('public.persona_interaction_contexts') IS NOT NULL THEN
        IF EXISTS (
            SELECT 1 FROM information_schema.columns
            WHERE table_schema = 'public'
              AND table_name = 'persona_interaction_contexts'
              AND column_name = 'persona_id'
        )
        AND NOT EXISTS (
            SELECT 1 FROM information_schema.columns
            WHERE table_schema = 'public'
              AND table_name = 'persona_interaction_contexts'
              AND column_name = 'interaction_context_id'
        ) THEN
            ALTER TABLE persona_interaction_contexts
                RENAME COLUMN persona_id TO interaction_context_id;
        END IF;

        IF EXISTS (
            SELECT 1 FROM information_schema.columns
            WHERE table_schema = 'public'
              AND table_name = 'persona_interaction_contexts'
              AND column_name = 'person_id'
        )
        AND NOT EXISTS (
            SELECT 1 FROM information_schema.columns
            WHERE table_schema = 'public'
              AND table_name = 'persona_interaction_contexts'
              AND column_name = 'source_persona_id'
        ) THEN
            ALTER TABLE persona_interaction_contexts
                RENAME COLUMN person_id TO source_persona_id;
        END IF;
    END IF;
END $$;

ALTER INDEX IF EXISTS personas_pkey RENAME TO personas_persona_id_pkey;
ALTER INDEX IF EXISTS personas_watchlist_idx RENAME TO personas_watchlist_persona_id_idx;
ALTER INDEX IF EXISTS personas_is_address_book_idx RENAME TO personas_address_book_persona_id_idx;
ALTER INDEX IF EXISTS persona_identities_person_id_idx RENAME TO persona_identities_persona_id_idx;
ALTER INDEX IF EXISTS persona_roles_person_id_idx RENAME TO persona_roles_persona_id_idx;
ALTER INDEX IF EXISTS persona_facts_person_id_idx RENAME TO persona_facts_persona_id_idx;
ALTER INDEX IF EXISTS persona_memory_cards_person_id_idx RENAME TO persona_memory_cards_persona_id_idx;
ALTER INDEX IF EXISTS persona_preferences_person_id_idx RENAME TO persona_preferences_persona_id_idx;
ALTER INDEX IF EXISTS persona_snapshots_person_id_idx RENAME TO persona_snapshots_persona_id_idx;
ALTER INDEX IF EXISTS persona_knowledge_conflicts_person_id_idx RENAME TO persona_knowledge_conflicts_persona_id_idx;
ALTER INDEX IF EXISTS relationship_events_person_id_idx RENAME TO relationship_events_persona_id_idx;
ALTER INDEX IF EXISTS enrichment_results_person_id_idx RENAME TO enrichment_results_persona_id_idx;
ALTER INDEX IF EXISTS enrichment_results_status_idx RENAME TO enrichment_results_persona_status_idx;
ALTER INDEX IF EXISTS persona_expertise_person_id_idx RENAME TO persona_expertise_persona_id_idx;
ALTER INDEX IF EXISTS persona_promises_person_id_idx RENAME TO persona_promises_persona_id_idx;
ALTER INDEX IF EXISTS persona_promises_status_idx RENAME TO persona_promises_persona_status_idx;
ALTER INDEX IF EXISTS persona_risks_person_id_idx RENAME TO persona_risks_persona_id_idx;
ALTER INDEX IF EXISTS org_persona_links_person_id_idx RENAME TO organization_persona_links_persona_id_idx;
ALTER INDEX IF EXISTS communication_message_participants_person_idx RENAME TO communication_message_participants_persona_id_idx;
ALTER INDEX IF EXISTS persona_interaction_contexts_person_id_idx RENAME TO persona_interaction_contexts_source_persona_id_idx;
ALTER INDEX IF EXISTS event_participants_person_idx RENAME TO event_participants_persona_idx;
ALTER INDEX IF EXISTS persona_identity_left_person_idx RENAME TO persona_identity_left_persona_idx;
ALTER INDEX IF EXISTS persona_identity_right_person_idx RENAME TO persona_identity_right_persona_idx;
