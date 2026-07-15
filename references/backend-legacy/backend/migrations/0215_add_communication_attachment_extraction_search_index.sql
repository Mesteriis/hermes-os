-- Derived search metadata only. Extracted attachment bytes remain in local blob
-- storage; this tsvector is invalidated whenever the source hash changes.

ALTER TABLE communication_attachment_extractions
    ADD COLUMN IF NOT EXISTS search_vector tsvector;

CREATE INDEX IF NOT EXISTS communication_attachment_extractions_search_vector_idx
    ON communication_attachment_extractions USING GIN (search_vector);
