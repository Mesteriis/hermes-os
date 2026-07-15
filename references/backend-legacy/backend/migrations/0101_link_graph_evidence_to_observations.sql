ALTER TABLE graph_evidence
    ADD COLUMN IF NOT EXISTS observation_id TEXT;

UPDATE graph_evidence evidence
SET observation_id = message.observation_id
FROM communication_messages message
WHERE evidence.source_kind = 'message'
  AND evidence.source_id = message.message_id
  AND evidence.observation_id IS NULL;

UPDATE graph_evidence evidence
SET observation_id = raw.observation_id
FROM communication_raw_records raw
WHERE evidence.source_kind = 'raw_record'
  AND evidence.source_id = raw.raw_record_id
  AND evidence.observation_id IS NULL;

UPDATE graph_evidence evidence
SET observation_id = observation.observation_id
FROM observations observation
WHERE evidence.source_kind = 'observation'
  AND evidence.source_id = observation.observation_id
  AND evidence.observation_id IS NULL;

ALTER TABLE graph_evidence
    DROP CONSTRAINT IF EXISTS graph_evidence_source_kind;

ALTER TABLE graph_evidence
    ADD CONSTRAINT graph_evidence_source_kind CHECK (
        source_kind IN (
            'contact',
            'person',
            'message',
            'document',
            'raw_record',
            'relationship',
            'decision',
            'obligation',
            'observation'
        )
    );

ALTER TABLE graph_evidence
    DROP CONSTRAINT IF EXISTS graph_evidence_observation_fk;

ALTER TABLE graph_evidence
    ADD CONSTRAINT graph_evidence_observation_fk
    FOREIGN KEY (observation_id)
    REFERENCES observations(observation_id);

ALTER TABLE graph_evidence
    DROP CONSTRAINT IF EXISTS graph_evidence_message_observation_required;

ALTER TABLE graph_evidence
    ADD CONSTRAINT graph_evidence_message_observation_required CHECK (
        source_kind != 'message'
        OR observation_id IS NOT NULL
    );

ALTER TABLE graph_evidence
    DROP CONSTRAINT IF EXISTS graph_evidence_observation_source_check;

ALTER TABLE graph_evidence
    ADD CONSTRAINT graph_evidence_observation_source_check CHECK (
        (
            source_kind = 'observation'
            AND observation_id IS NOT NULL
            AND observation_id = source_id
        )
        OR source_kind != 'observation'
    );

CREATE INDEX IF NOT EXISTS graph_evidence_observation_idx
    ON graph_evidence (observation_id)
    WHERE observation_id IS NOT NULL;
