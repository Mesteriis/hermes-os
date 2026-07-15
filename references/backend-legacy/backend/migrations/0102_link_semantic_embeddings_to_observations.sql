ALTER TABLE semantic_embeddings
    ADD COLUMN IF NOT EXISTS observation_id TEXT;

UPDATE semantic_embeddings embedding
SET observation_id = message.observation_id
FROM communication_messages message
WHERE embedding.source_kind = 'message'
  AND embedding.source_id = message.message_id
  AND embedding.observation_id IS NULL;

ALTER TABLE semantic_embeddings
    DROP CONSTRAINT IF EXISTS semantic_embeddings_observation_fk;

ALTER TABLE semantic_embeddings
    ADD CONSTRAINT semantic_embeddings_observation_fk
    FOREIGN KEY (observation_id)
    REFERENCES observations(observation_id);

ALTER TABLE semantic_embeddings
    DROP CONSTRAINT IF EXISTS semantic_embeddings_message_observation_required;

ALTER TABLE semantic_embeddings
    ADD CONSTRAINT semantic_embeddings_message_observation_required CHECK (
        source_kind != 'message'
        OR observation_id IS NOT NULL
    );

CREATE INDEX IF NOT EXISTS semantic_embeddings_observation_idx
    ON semantic_embeddings (observation_id)
    WHERE observation_id IS NOT NULL;
