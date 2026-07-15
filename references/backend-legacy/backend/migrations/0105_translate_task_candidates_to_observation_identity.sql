UPDATE task_candidates candidate
SET observation_id = message.observation_id
FROM communication_messages message
WHERE candidate.source_kind = 'message'
  AND candidate.source_id = message.message_id
  AND candidate.observation_id IS NULL;

UPDATE task_candidates candidate
SET observation_id = document.observation_id
FROM documents document
WHERE candidate.source_kind = 'document'
  AND candidate.source_id = document.document_id
  AND candidate.observation_id IS NULL;

UPDATE task_candidates
SET
    source_kind = 'observation',
    source_id = observation_id
WHERE observation_id IS NOT NULL
  AND source_kind IN ('message', 'document');

ALTER TABLE task_candidates
    DROP CONSTRAINT IF EXISTS task_candidates_source_kind_check;

ALTER TABLE task_candidates
    ADD CONSTRAINT task_candidates_source_kind_check
    CHECK (source_kind IN ('observation'));

ALTER TABLE task_candidates
    DROP CONSTRAINT IF EXISTS task_candidates_message_observation_required;

ALTER TABLE task_candidates
    DROP CONSTRAINT IF EXISTS task_candidates_observation_required;

ALTER TABLE task_candidates
    ADD CONSTRAINT task_candidates_observation_required CHECK (
        observation_id IS NOT NULL
        AND source_id = observation_id
    );
