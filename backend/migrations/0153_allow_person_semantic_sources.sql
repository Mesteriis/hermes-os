ALTER TABLE semantic_embeddings
    DROP CONSTRAINT IF EXISTS semantic_embeddings_source_kind_check;

ALTER TABLE semantic_embeddings
    ADD CONSTRAINT semantic_embeddings_source_kind_check
    CHECK (source_kind IN ('message', 'document', 'project', 'task', 'contact', 'person'));
