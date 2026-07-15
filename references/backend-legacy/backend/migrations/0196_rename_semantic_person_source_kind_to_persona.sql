-- Persona semantic source naming alignment.
--
-- Older AI semantic index rows used `contact` and then `person` as source_kind
-- for Persona records. Keep one latest row per Persona/model and make new writes
-- use `persona`.

ALTER TABLE semantic_embeddings
    DROP CONSTRAINT IF EXISTS semantic_embeddings_source_kind_check;

WITH ranked_persona_sources AS (
    SELECT
        ctid,
        row_number() OVER (
            PARTITION BY source_id, embedding_model
            ORDER BY
                CASE source_kind
                    WHEN 'persona' THEN 0
                    WHEN 'person' THEN 1
                    ELSE 2
                END,
                updated_at DESC,
                created_at DESC
        ) AS rank
    FROM semantic_embeddings
    WHERE source_kind IN ('contact', 'person', 'persona')
)
DELETE FROM semantic_embeddings embedding
USING ranked_persona_sources ranked
WHERE embedding.ctid = ranked.ctid
  AND ranked.rank > 1;

UPDATE semantic_embeddings
SET source_kind = 'persona'
WHERE source_kind IN ('contact', 'person');

ALTER TABLE semantic_embeddings
    ADD CONSTRAINT semantic_embeddings_source_kind_check
    CHECK (source_kind IN ('message', 'document', 'project', 'task', 'persona'));
