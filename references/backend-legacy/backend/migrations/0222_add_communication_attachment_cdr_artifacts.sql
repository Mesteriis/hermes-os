-- Content-disarmed artifacts are derived, immutable replacements for display/download.
-- The original attachment remains in content-addressed blob storage.
CREATE TABLE IF NOT EXISTS communication_attachment_cdr_artifacts (
    attachment_id TEXT PRIMARY KEY REFERENCES communication_attachments(attachment_id) ON DELETE CASCADE,
    status TEXT NOT NULL,
    renderer TEXT NOT NULL,
    source_sha256 TEXT NOT NULL,
    artifact_blob_id TEXT REFERENCES communication_mail_blobs(blob_id) ON DELETE RESTRICT,
    artifact_content_type TEXT,
    artifact_size_bytes BIGINT,
    failure_summary TEXT,
    disarmed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT communication_attachment_cdr_status CHECK (status IN ('executing', 'completed', 'unsupported', 'failed')),
    CONSTRAINT communication_attachment_cdr_completed CHECK (
        (status = 'completed' AND artifact_blob_id IS NOT NULL AND artifact_content_type = 'application/pdf'
         AND artifact_size_bytes IS NOT NULL AND disarmed_at IS NOT NULL)
        OR (status <> 'completed' AND artifact_blob_id IS NULL AND artifact_content_type IS NULL
            AND artifact_size_bytes IS NULL AND disarmed_at IS NULL)
    ),
    CONSTRAINT communication_attachment_cdr_failure CHECK (
        (status = 'failed' AND failure_summary IS NOT NULL AND length(trim(failure_summary)) > 0)
        OR (status <> 'failed' AND failure_summary IS NULL)
    )
);
