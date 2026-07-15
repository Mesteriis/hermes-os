CREATE TABLE IF NOT EXISTS communication_mail_blobs (
    blob_id TEXT PRIMARY KEY,
    storage_kind TEXT NOT NULL,
    storage_path TEXT NOT NULL,
    sha256 TEXT NOT NULL,
    size_bytes BIGINT NOT NULL,
    content_type TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT communication_mail_blob_storage_kind CHECK (storage_kind IN ('local_fs')),
    CONSTRAINT communication_mail_blob_id_not_empty CHECK (length(trim(blob_id)) > 0),
    CONSTRAINT communication_mail_blob_storage_path_not_empty CHECK (length(trim(storage_path)) > 0),
    CONSTRAINT communication_mail_blob_sha256_not_empty CHECK (length(trim(sha256)) > 0),
    CONSTRAINT communication_mail_blob_size_non_negative CHECK (size_bytes >= 0),
    CONSTRAINT communication_mail_blob_content_type_not_empty CHECK (
        content_type IS NULL OR length(trim(content_type)) > 0
    ),
    CONSTRAINT communication_mail_blob_storage_path_unique UNIQUE (storage_kind, storage_path),
    CONSTRAINT communication_mail_blob_sha256_unique UNIQUE (sha256)
);

CREATE TABLE IF NOT EXISTS communication_attachments (
    attachment_id TEXT PRIMARY KEY,
    message_id TEXT NOT NULL REFERENCES communication_messages(message_id) ON DELETE RESTRICT,
    raw_record_id TEXT NOT NULL REFERENCES communication_raw_records(raw_record_id) ON DELETE RESTRICT,
    blob_id TEXT NOT NULL REFERENCES communication_mail_blobs(blob_id) ON DELETE RESTRICT,
    provider_attachment_id TEXT NOT NULL,
    filename TEXT,
    content_type TEXT NOT NULL,
    size_bytes BIGINT NOT NULL,
    sha256 TEXT NOT NULL,
    disposition TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT communication_attachment_id_not_empty CHECK (length(trim(attachment_id)) > 0),
    CONSTRAINT communication_attachment_provider_id_not_empty CHECK (length(trim(provider_attachment_id)) > 0),
    CONSTRAINT communication_attachment_filename_not_empty CHECK (
        filename IS NULL OR length(trim(filename)) > 0
    ),
    CONSTRAINT communication_attachment_content_type_not_empty CHECK (length(trim(content_type)) > 0),
    CONSTRAINT communication_attachment_size_non_negative CHECK (size_bytes >= 0),
    CONSTRAINT communication_attachment_sha256_not_empty CHECK (length(trim(sha256)) > 0),
    CONSTRAINT communication_attachment_disposition CHECK (disposition IN ('attachment', 'inline', 'unknown')),
    CONSTRAINT communication_attachment_provider_identity_unique UNIQUE (message_id, provider_attachment_id)
);

CREATE INDEX IF NOT EXISTS communication_attachments_message_idx
    ON communication_attachments (message_id, created_at);

CREATE INDEX IF NOT EXISTS communication_attachments_raw_record_idx
    ON communication_attachments (raw_record_id);

CREATE INDEX IF NOT EXISTS communication_attachments_blob_idx
    ON communication_attachments (blob_id);
