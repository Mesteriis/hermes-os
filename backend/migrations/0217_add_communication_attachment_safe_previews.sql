-- Safe previews are derived artifacts. The original attachment remains in
-- blob storage and is never made browser-renderable through this table.

CREATE TABLE IF NOT EXISTS communication_attachment_safe_previews (
    attachment_id TEXT PRIMARY KEY REFERENCES communication_attachments(attachment_id) ON DELETE CASCADE,
    status TEXT NOT NULL,
    renderer TEXT NOT NULL,
    source_sha256 TEXT NOT NULL,
    preview_blob_id TEXT REFERENCES communication_mail_blobs(blob_id) ON DELETE RESTRICT,
    preview_content_type TEXT,
    preview_size_bytes BIGINT,
    failure_summary TEXT,
    rendered_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT communication_attachment_safe_previews_status
        CHECK (status IN ('executing', 'completed', 'unsupported', 'failed')),
    CONSTRAINT communication_attachment_safe_previews_renderer_not_empty
        CHECK (length(trim(renderer)) > 0),
    CONSTRAINT communication_attachment_safe_previews_source_sha256_not_empty
        CHECK (length(trim(source_sha256)) > 0),
    CONSTRAINT communication_attachment_safe_previews_completed_artifact
        CHECK (
            (status = 'completed'
                AND preview_blob_id IS NOT NULL
                AND preview_content_type = 'image/png'
                AND preview_size_bytes IS NOT NULL
                AND rendered_at IS NOT NULL)
            OR (status <> 'completed'
                AND preview_blob_id IS NULL
                AND preview_content_type IS NULL
                AND preview_size_bytes IS NULL
                AND rendered_at IS NULL)
        ),
    CONSTRAINT communication_attachment_safe_previews_size_non_negative
        CHECK (preview_size_bytes IS NULL OR preview_size_bytes >= 0),
    CONSTRAINT communication_attachment_safe_previews_failure_summary
        CHECK (
            (status = 'failed' AND failure_summary IS NOT NULL AND length(trim(failure_summary)) > 0)
            OR (status <> 'failed' AND failure_summary IS NULL)
        )
);

CREATE INDEX IF NOT EXISTS communication_attachment_safe_previews_status_idx
    ON communication_attachment_safe_previews (status, updated_at DESC);
