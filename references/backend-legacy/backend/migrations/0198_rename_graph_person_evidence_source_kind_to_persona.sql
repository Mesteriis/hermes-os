-- Graph evidence source naming alignment for Persona evidence.

ALTER TABLE graph_evidence
    DROP CONSTRAINT IF EXISTS graph_evidence_source_kind;

WITH ranked_persona_evidence AS (
    SELECT
        ctid,
        row_number() OVER (
            PARTITION BY edge_id, source_id
            ORDER BY
                CASE source_kind
                    WHEN 'persona' THEN 0
                    WHEN 'person' THEN 1
                    ELSE 2
                END,
                created_at DESC
        ) AS rank
    FROM graph_evidence
    WHERE source_kind IN ('contact', 'person', 'persona')
)
DELETE FROM graph_evidence evidence
USING ranked_persona_evidence ranked
WHERE evidence.ctid = ranked.ctid
  AND ranked.rank > 1;

UPDATE graph_evidence
SET source_kind = 'persona'
WHERE source_kind IN ('contact', 'person');

ALTER TABLE graph_evidence
    ADD CONSTRAINT graph_evidence_source_kind CHECK (
        source_kind IN (
            'persona',
            'message',
            'document',
            'raw_record',
            'relationship',
            'decision',
            'obligation',
            'observation'
        )
    );
