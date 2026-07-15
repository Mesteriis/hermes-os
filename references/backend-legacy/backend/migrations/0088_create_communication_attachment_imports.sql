-- Migration 0088: Provider-neutral local attachment imports for composer/media upload.
-- Imported rows reference local Communication blobs before a provider message exists.

CREATE TABLE IF NOT EXISTS communication_attachment_imports (
    attachment_id TEXT PRIMARY KEY,
    account_id TEXT,
    channel_kind TEXT,
    blob_id TEXT NOT NULL REFERENCES communication_mail_blobs(blob_id) ON DELETE RESTRICT,
    filename TEXT,
    content_type TEXT NOT NULL,
    size_bytes BIGINT NOT NULL,
    sha256 TEXT NOT NULL,
    source_kind TEXT NOT NULL DEFAULT 'local_import',
    imported_by TEXT NOT NULL,
    scan_status TEXT NOT NULL DEFAULT 'not_scanned',
    scan_engine TEXT,
    scan_checked_at TIMESTAMPTZ,
    scan_summary TEXT,
    scan_metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT communication_attachment_imports_attachment_id_not_empty
        CHECK (length(trim(attachment_id)) > 0),
    CONSTRAINT communication_attachment_imports_account_id_not_empty
        CHECK (account_id IS NULL OR length(trim(account_id)) > 0),
    CONSTRAINT communication_attachment_imports_channel_kind_not_empty
        CHECK (channel_kind IS NULL OR length(trim(channel_kind)) > 0),
    CONSTRAINT communication_attachment_imports_filename_not_empty
        CHECK (filename IS NULL OR length(trim(filename)) > 0),
    CONSTRAINT communication_attachment_imports_content_type_not_empty
        CHECK (length(trim(content_type)) > 0),
    CONSTRAINT communication_attachment_imports_size_positive
        CHECK (size_bytes > 0),
    CONSTRAINT communication_attachment_imports_sha256_format
        CHECK (sha256 ~ '^sha256:[0-9a-f]{64}$'),
    CONSTRAINT communication_attachment_imports_source_kind_not_empty
        CHECK (length(trim(source_kind)) > 0),
    CONSTRAINT communication_attachment_imports_imported_by_not_empty
        CHECK (length(trim(imported_by)) > 0),
    CONSTRAINT communication_attachment_imports_scan_status
        CHECK (scan_status IN ('not_scanned', 'clean', 'suspicious', 'malicious', 'failed')),
    CONSTRAINT communication_attachment_imports_scan_metadata_object
        CHECK (jsonb_typeof(scan_metadata) = 'object'),
    CONSTRAINT communication_attachment_imports_metadata_object
        CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE INDEX IF NOT EXISTS communication_attachment_imports_account_idx
    ON communication_attachment_imports (account_id, created_at DESC);

CREATE INDEX IF NOT EXISTS communication_attachment_imports_blob_idx
    ON communication_attachment_imports (blob_id);

CREATE INDEX IF NOT EXISTS communication_attachment_imports_sha256_idx
    ON communication_attachment_imports (sha256);
