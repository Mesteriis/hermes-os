-- Derived attachment text remains in local blob storage. PostgreSQL retains
-- only a reference, bounded processing status, and source identity metadata.

CREATE TABLE IF NOT EXISTS communication_attachment_extractions (
    attachment_id TEXT PRIMARY KEY REFERENCES communication_attachments(attachment_id) ON DELETE CASCADE,
    status TEXT NOT NULL,
    extractor TEXT NOT NULL,
    source_sha256 TEXT NOT NULL,
    extracted_blob_id TEXT REFERENCES communication_mail_blobs(blob_id) ON DELETE RESTRICT,
    extracted_size_bytes BIGINT,
    failure_summary TEXT,
    extracted_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT communication_attachment_extractions_status
        CHECK (status IN ('completed', 'unsupported', 'failed')),
    CONSTRAINT communication_attachment_extractions_extractor_not_empty
        CHECK (length(trim(extractor)) > 0),
    CONSTRAINT communication_attachment_extractions_source_sha256_not_empty
        CHECK (length(trim(source_sha256)) > 0),
    CONSTRAINT communication_attachment_extractions_completed_blob
        CHECK (
            (status = 'completed' AND extracted_blob_id IS NOT NULL AND extracted_size_bytes IS NOT NULL AND extracted_at IS NOT NULL)
            OR (status <> 'completed' AND extracted_blob_id IS NULL AND extracted_size_bytes IS NULL AND extracted_at IS NULL)
        ),
    CONSTRAINT communication_attachment_extractions_size_non_negative
        CHECK (extracted_size_bytes IS NULL OR extracted_size_bytes >= 0),
    CONSTRAINT communication_attachment_extractions_failure_summary
        CHECK (
            (status = 'failed' AND failure_summary IS NOT NULL AND length(trim(failure_summary)) > 0)
            OR (status <> 'failed' AND failure_summary IS NULL)
        )
);

CREATE INDEX IF NOT EXISTS communication_attachment_extractions_status_idx
    ON communication_attachment_extractions (status, updated_at DESC);

CREATE INDEX IF NOT EXISTS communication_attachment_extractions_source_sha256_idx
    ON communication_attachment_extractions (source_sha256);
