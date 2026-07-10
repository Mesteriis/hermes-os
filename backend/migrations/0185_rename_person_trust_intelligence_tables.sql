-- Rename remaining legacy Person storage tables to Persona storage tables.

ALTER TABLE IF EXISTS person_snapshots RENAME TO persona_snapshots;
ALTER TABLE IF EXISTS person_knowledge_conflicts RENAME TO persona_knowledge_conflicts;
ALTER TABLE IF EXISTS person_expertise RENAME TO persona_expertise;
ALTER TABLE IF EXISTS person_promises RENAME TO persona_promises;
ALTER TABLE IF EXISTS person_risks RENAME TO persona_risks;

ALTER INDEX IF EXISTS person_snapshots_person_id_idx RENAME TO persona_snapshots_person_id_idx;
ALTER INDEX IF EXISTS person_knowledge_conflicts_person_id_idx RENAME TO persona_knowledge_conflicts_person_id_idx;
ALTER INDEX IF EXISTS person_expertise_person_id_idx RENAME TO persona_expertise_person_id_idx;
ALTER INDEX IF EXISTS person_expertise_skill_idx RENAME TO persona_expertise_skill_idx;
ALTER INDEX IF EXISTS person_promises_person_id_idx RENAME TO persona_promises_person_id_idx;
ALTER INDEX IF EXISTS person_promises_status_idx RENAME TO persona_promises_status_idx;
ALTER INDEX IF EXISTS person_risks_person_id_idx RENAME TO persona_risks_person_id_idx;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'person_snapshots_data_is_object') THEN
        ALTER TABLE persona_snapshots
            RENAME CONSTRAINT person_snapshots_data_is_object TO persona_snapshots_data_is_object;
    END IF;

    IF EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'person_expertise_confidence_range') THEN
        ALTER TABLE persona_expertise
            RENAME CONSTRAINT person_expertise_confidence_range TO persona_expertise_confidence_range;
    END IF;

    IF EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'person_expertise_skill_not_empty') THEN
        ALTER TABLE persona_expertise
            RENAME CONSTRAINT person_expertise_skill_not_empty TO persona_expertise_skill_not_empty;
    END IF;

    IF EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'person_promises_status_check') THEN
        ALTER TABLE persona_promises
            RENAME CONSTRAINT person_promises_status_check TO persona_promises_status_check;
    END IF;

    IF EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'person_promises_desc_not_empty') THEN
        ALTER TABLE persona_promises
            RENAME CONSTRAINT person_promises_desc_not_empty TO persona_promises_desc_not_empty;
    END IF;

    IF EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'person_risks_severity_check') THEN
        ALTER TABLE persona_risks
            RENAME CONSTRAINT person_risks_severity_check TO persona_risks_severity_check;
    END IF;

    IF EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'person_risks_confidence_range') THEN
        ALTER TABLE persona_risks
            RENAME CONSTRAINT person_risks_confidence_range TO persona_risks_confidence_range;
    END IF;
END $$;
