-- Rename legacy Person memory storage to Persona memory storage.

ALTER TABLE IF EXISTS person_facts RENAME TO persona_facts;
ALTER TABLE IF EXISTS person_memory_cards RENAME TO persona_memory_cards;
ALTER TABLE IF EXISTS person_preferences RENAME TO persona_preferences;

ALTER INDEX IF EXISTS person_facts_person_id_idx RENAME TO persona_facts_person_id_idx;
ALTER INDEX IF EXISTS person_facts_type_idx RENAME TO persona_facts_type_idx;
ALTER INDEX IF EXISTS person_memory_cards_person_id_idx RENAME TO persona_memory_cards_person_id_idx;
ALTER INDEX IF EXISTS person_preferences_person_id_idx RENAME TO persona_preferences_person_id_idx;

DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'person_facts_confidence_range'
    ) THEN
        ALTER TABLE persona_facts
            RENAME CONSTRAINT person_facts_confidence_range TO persona_facts_confidence_range;
    END IF;

    IF EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'person_memory_cards_confidence_range'
    ) THEN
        ALTER TABLE persona_memory_cards
            RENAME CONSTRAINT person_memory_cards_confidence_range TO persona_memory_cards_confidence_range;
    END IF;

    IF EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'person_memory_cards_importance_range'
    ) THEN
        ALTER TABLE persona_memory_cards
            RENAME CONSTRAINT person_memory_cards_importance_range TO persona_memory_cards_importance_range;
    END IF;

    IF EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'person_memory_cards_title_not_empty'
    ) THEN
        ALTER TABLE persona_memory_cards
            RENAME CONSTRAINT person_memory_cards_title_not_empty TO persona_memory_cards_title_not_empty;
    END IF;

    IF EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'person_preferences_unique'
    ) THEN
        ALTER TABLE persona_preferences
            RENAME CONSTRAINT person_preferences_unique TO persona_preferences_unique;
    END IF;

    IF EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'person_preferences_confidence_range'
    ) THEN
        ALTER TABLE persona_preferences
            RENAME CONSTRAINT person_preferences_confidence_range TO persona_preferences_confidence_range;
    END IF;
END $$;
