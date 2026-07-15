ALTER TABLE decision_evidence
    ADD COLUMN IF NOT EXISTS observation_id TEXT;

ALTER TABLE obligation_evidence
    ADD COLUMN IF NOT EXISTS observation_id TEXT;

ALTER TABLE relationship_evidence
    ADD COLUMN IF NOT EXISTS observation_id TEXT;

UPDATE decision_evidence evidence
SET observation_id = observation.observation_id
FROM observations observation
WHERE evidence.observation_id IS NULL
  AND evidence.source_id = observation.observation_id;

UPDATE obligation_evidence evidence
SET observation_id = observation.observation_id
FROM observations observation
WHERE evidence.observation_id IS NULL
  AND evidence.source_id = observation.observation_id;

UPDATE relationship_evidence evidence
SET observation_id = observation.observation_id
FROM observations observation
WHERE evidence.observation_id IS NULL
  AND evidence.source_id = observation.observation_id;

ALTER TABLE decision_evidence
    DROP CONSTRAINT IF EXISTS decision_evidence_source_kind_check;

ALTER TABLE decision_evidence
    ADD CONSTRAINT decision_evidence_source_kind_check CHECK (
        source_kind IN (
            'observation',
            'communication',
            'document',
            'event',
            'memory',
            'knowledge',
            'decision',
            'obligation',
            'task',
            'relationship',
            'project',
            'organization',
            'persona',
            'raw_record'
        )
    );

ALTER TABLE obligation_evidence
    DROP CONSTRAINT IF EXISTS obligation_evidence_source_kind_check;

ALTER TABLE obligation_evidence
    ADD CONSTRAINT obligation_evidence_source_kind_check CHECK (
        source_kind IN (
            'observation',
            'communication',
            'document',
            'event',
            'memory',
            'knowledge',
            'decision',
            'obligation',
            'task',
            'project',
            'organization',
            'persona',
            'raw_record'
        )
    );

ALTER TABLE relationship_evidence
    DROP CONSTRAINT IF EXISTS relationship_evidence_source_kind_check;

ALTER TABLE relationship_evidence
    ADD CONSTRAINT relationship_evidence_source_kind_check CHECK (
        source_kind IN (
            'observation',
            'communication',
            'document',
            'event',
            'memory',
            'knowledge',
            'decision',
            'obligation',
            'task',
            'project',
            'organization',
            'persona',
            'raw_record'
        )
    );

ALTER TABLE decision_evidence
    DROP CONSTRAINT IF EXISTS decision_evidence_observation_fk;

ALTER TABLE decision_evidence
    ADD CONSTRAINT decision_evidence_observation_fk
    FOREIGN KEY (observation_id)
    REFERENCES observations(observation_id);

ALTER TABLE obligation_evidence
    DROP CONSTRAINT IF EXISTS obligation_evidence_observation_fk;

ALTER TABLE obligation_evidence
    ADD CONSTRAINT obligation_evidence_observation_fk
    FOREIGN KEY (observation_id)
    REFERENCES observations(observation_id);

ALTER TABLE relationship_evidence
    DROP CONSTRAINT IF EXISTS relationship_evidence_observation_fk;

ALTER TABLE relationship_evidence
    ADD CONSTRAINT relationship_evidence_observation_fk
    FOREIGN KEY (observation_id)
    REFERENCES observations(observation_id);

ALTER TABLE decision_evidence
    DROP CONSTRAINT IF EXISTS decision_evidence_observation_source_check;

ALTER TABLE decision_evidence
    ADD CONSTRAINT decision_evidence_observation_source_check CHECK (
        (
            source_kind = 'observation'
            AND observation_id IS NOT NULL
            AND observation_id = source_id
        )
        OR source_kind != 'observation'
    );

ALTER TABLE obligation_evidence
    DROP CONSTRAINT IF EXISTS obligation_evidence_observation_source_check;

ALTER TABLE obligation_evidence
    ADD CONSTRAINT obligation_evidence_observation_source_check CHECK (
        (
            source_kind = 'observation'
            AND observation_id IS NOT NULL
            AND observation_id = source_id
        )
        OR source_kind != 'observation'
    );

ALTER TABLE relationship_evidence
    DROP CONSTRAINT IF EXISTS relationship_evidence_observation_source_check;

ALTER TABLE relationship_evidence
    ADD CONSTRAINT relationship_evidence_observation_source_check CHECK (
        (
            source_kind = 'observation'
            AND observation_id IS NOT NULL
            AND observation_id = source_id
        )
        OR source_kind != 'observation'
    );

CREATE INDEX IF NOT EXISTS decision_evidence_observation_idx
    ON decision_evidence (observation_id)
    WHERE observation_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS obligation_evidence_observation_idx
    ON obligation_evidence (observation_id)
    WHERE observation_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS relationship_evidence_observation_idx
    ON relationship_evidence (observation_id)
    WHERE observation_id IS NOT NULL;
