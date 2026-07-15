ALTER TABLE task_candidates
    ADD COLUMN IF NOT EXISTS observation_id TEXT;

UPDATE task_candidates candidate
SET observation_id = message.observation_id
FROM communication_messages message
WHERE candidate.source_kind = 'message'
  AND candidate.source_id = message.message_id
  AND candidate.observation_id IS NULL;

ALTER TABLE task_candidates
    DROP CONSTRAINT IF EXISTS task_candidates_observation_fk;

ALTER TABLE task_candidates
    ADD CONSTRAINT task_candidates_observation_fk
    FOREIGN KEY (observation_id)
    REFERENCES observations(observation_id);

ALTER TABLE task_candidates
    DROP CONSTRAINT IF EXISTS task_candidates_message_observation_required;

ALTER TABLE task_candidates
    ADD CONSTRAINT task_candidates_message_observation_required CHECK (
        source_kind != 'message'
        OR observation_id IS NOT NULL
    );

CREATE INDEX IF NOT EXISTS task_candidates_observation_idx
    ON task_candidates (observation_id)
    WHERE observation_id IS NOT NULL;
