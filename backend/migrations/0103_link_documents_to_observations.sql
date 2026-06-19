ALTER TABLE documents
    ADD COLUMN IF NOT EXISTS observation_id TEXT;

INSERT INTO observations (
    observation_id,
    kind_definition_id,
    origin_kind,
    vault_source_id,
    observed_at,
    captured_at,
    payload,
    confidence,
    content_hash,
    source_ref,
    provenance
)
SELECT
    'observation:v1:legacy-document:' || md5(
        COALESCE(document_id, '') || '|' || imported_at::text || '|' || source_fingerprint || '|' || COALESCE(extracted_text, '')
    ),
    kind.kind_definition_id,
    'file_import',
    NULL,
    imported_at,
    imported_at,
    jsonb_build_object(
        'legacy_document_id', document_id,
        'document_kind', document_kind,
        'title', title,
        'source_fingerprint', source_fingerprint,
        'extracted_text', extracted_text,
        'legacy_backfill', true
    ),
    1.0,
    'sha256:' || md5(
        COALESCE(document_id, '') || '|' || COALESCE(document_kind, '') || '|' || COALESCE(title, '') || '|' ||
        COALESCE(source_fingerprint, '') || '|' || COALESCE(extracted_text, '')
    ),
    'document://' || document_id,
    jsonb_build_object('legacy_backfill', true)
FROM documents
LEFT JOIN observation_kind_definitions kind
  ON kind.code = 'DOCUMENT'
 AND kind.version = 1
WHERE documents.observation_id IS NULL
  AND kind.kind_definition_id IS NOT NULL
ON CONFLICT (observation_id) DO NOTHING;

UPDATE documents
SET observation_id = 'observation:v1:legacy-document:' || md5(
    COALESCE(documents.document_id, '') || '|' || documents.imported_at::text || '|' || documents.source_fingerprint || '|' || COALESCE(documents.extracted_text, '')
)
FROM observation_kind_definitions kind
WHERE documents.observation_id IS NULL
  AND kind.code = 'DOCUMENT'
  AND kind.version = 1
  AND EXISTS (
        SELECT 1
        FROM observations ob
        WHERE ob.observation_id = 'observation:v1:legacy-document:' || md5(
            COALESCE(documents.document_id, '') || '|' || documents.imported_at::text || '|' || documents.source_fingerprint || '|' || COALESCE(documents.extracted_text, '')
        )
        AND ob.source_ref = 'document://' || documents.document_id
    );

ALTER TABLE documents
    ALTER COLUMN observation_id SET NOT NULL;

ALTER TABLE documents
    DROP CONSTRAINT IF EXISTS documents_observation_fk;

ALTER TABLE documents
    ADD CONSTRAINT documents_observation_fk
    FOREIGN KEY (observation_id)
    REFERENCES observations(observation_id);

ALTER TABLE documents
    DROP CONSTRAINT IF EXISTS documents_source_kind_observation_check;

ALTER TABLE documents
    ADD CONSTRAINT documents_source_kind_observation_check CHECK (
        observation_id IS NOT NULL
    );

CREATE INDEX IF NOT EXISTS documents_observation_idx
    ON documents (observation_id);
