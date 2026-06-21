ALTER TABLE persons
    ADD COLUMN IF NOT EXISTS is_self BOOLEAN NOT NULL DEFAULT false;

UPDATE persons
SET person_type = 'human'
WHERE person_type IS NULL
   OR person_type NOT IN ('human', 'ai_agent', 'organization_proxy', 'system');

ALTER TABLE persons
    ALTER COLUMN person_type SET DEFAULT 'human',
    ALTER COLUMN person_type SET NOT NULL;

DO $$
BEGIN
    ALTER TABLE persons
        ADD CONSTRAINT persons_person_type_check
        CHECK (person_type IN ('human', 'ai_agent', 'organization_proxy', 'system'));
EXCEPTION
    WHEN duplicate_object THEN NULL;
END $$;

CREATE INDEX IF NOT EXISTS persons_person_type_idx
    ON persons (person_type);

CREATE UNIQUE INDEX IF NOT EXISTS persons_single_self_idx
    ON persons (is_self)
    WHERE is_self = true;
