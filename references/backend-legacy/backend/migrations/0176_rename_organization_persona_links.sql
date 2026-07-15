DO $$
BEGIN
    IF to_regclass('public.organization_contact_links') IS NOT NULL
       AND to_regclass('public.organization_persona_links') IS NULL THEN
        ALTER TABLE organization_contact_links RENAME TO organization_persona_links;
    END IF;

    IF to_regclass('public.organization_person_links') IS NOT NULL
       AND to_regclass('public.organization_persona_links') IS NULL THEN
        ALTER TABLE organization_person_links RENAME TO organization_persona_links;
    END IF;
END $$;

DO $$
BEGIN
    IF to_regclass('public.organization_persona_links') IS NOT NULL
       AND EXISTS (
           SELECT 1
           FROM pg_constraint
           WHERE conname = 'org_contact_links_unique'
             AND conrelid = 'public.organization_persona_links'::regclass
       ) THEN
        ALTER TABLE organization_persona_links
            RENAME CONSTRAINT org_contact_links_unique TO org_persona_links_unique;
    END IF;

    IF to_regclass('public.organization_persona_links') IS NOT NULL
       AND EXISTS (
           SELECT 1
           FROM pg_constraint
           WHERE conname = 'org_person_links_unique'
             AND conrelid = 'public.organization_persona_links'::regclass
       ) THEN
        ALTER TABLE organization_persona_links
            RENAME CONSTRAINT org_person_links_unique TO org_persona_links_unique;
    END IF;

    IF to_regclass('public.organization_persona_links') IS NOT NULL
       AND EXISTS (
           SELECT 1
           FROM pg_constraint
           WHERE conname = 'org_contact_links_confidence_range'
             AND conrelid = 'public.organization_persona_links'::regclass
       ) THEN
        ALTER TABLE organization_persona_links
            RENAME CONSTRAINT org_contact_links_confidence_range TO org_persona_links_confidence_range;
    END IF;

    IF to_regclass('public.organization_persona_links') IS NOT NULL
       AND EXISTS (
           SELECT 1
           FROM pg_constraint
           WHERE conname = 'org_person_links_confidence_range'
             AND conrelid = 'public.organization_persona_links'::regclass
       ) THEN
        ALTER TABLE organization_persona_links
            RENAME CONSTRAINT org_person_links_confidence_range TO org_persona_links_confidence_range;
    END IF;
END $$;

ALTER INDEX IF EXISTS org_contact_links_org_id_idx RENAME TO org_persona_links_org_id_idx;
ALTER INDEX IF EXISTS org_contact_links_person_id_idx RENAME TO org_persona_links_person_id_idx;
ALTER INDEX IF EXISTS org_person_links_org_id_idx RENAME TO org_persona_links_org_id_idx;
ALTER INDEX IF EXISTS org_person_links_person_id_idx RENAME TO org_persona_links_person_id_idx;

UPDATE relationships
SET metadata = jsonb_set(metadata, '{compatibility_table}', to_jsonb('organization_persona_links'::text), true),
    updated_at = now()
WHERE metadata->>'compatibility_table' = 'organization_contact_links';

UPDATE relationships
SET metadata = jsonb_set(metadata, '{compatibility_table}', to_jsonb('organization_persona_links'::text), true),
    updated_at = now()
WHERE metadata->>'compatibility_table' = 'organization_person_links';

UPDATE relationship_evidence
SET metadata = jsonb_set(metadata, '{compatibility_table}', to_jsonb('organization_persona_links'::text), true)
WHERE metadata->>'compatibility_table' = 'organization_contact_links';

UPDATE relationship_evidence
SET metadata = jsonb_set(metadata, '{compatibility_table}', to_jsonb('organization_persona_links'::text), true)
WHERE metadata->>'compatibility_table' = 'organization_person_links';
