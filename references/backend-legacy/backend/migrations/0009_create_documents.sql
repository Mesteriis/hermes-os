CREATE TABLE IF NOT EXISTS documents (
    document_id TEXT PRIMARY KEY,
    document_kind TEXT NOT NULL,
    title TEXT NOT NULL,
    source_fingerprint TEXT NOT NULL,
    extracted_text TEXT NOT NULL,
    imported_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT documents_kind CHECK (document_kind IN ('markdown', 'pdf')),
    CONSTRAINT documents_title_not_empty CHECK (length(trim(title)) > 0),
    CONSTRAINT documents_fingerprint_not_empty CHECK (length(trim(source_fingerprint)) > 0)
);
