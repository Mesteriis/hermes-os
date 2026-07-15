CREATE TABLE IF NOT EXISTS email_legal_documents (
    document_id TEXT PRIMARY KEY,
    message_id TEXT,
    document_type TEXT NOT NULL DEFAULT 'other',
    title TEXT NOT NULL,
    parties JSONB NOT NULL DEFAULT '[]'::jsonb,
    effective_date TIMESTAMPTZ,
    expiry_date TIMESTAMPTZ,
    amount DOUBLE PRECISION,
    currency TEXT,
    status TEXT NOT NULL DEFAULT 'draft',
    linked_project_id TEXT,
    risks JSONB NOT NULL DEFAULT '[]'::jsonb,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT email_legal_docs_title_not_empty CHECK (length(trim(title)) > 0),
    CONSTRAINT email_legal_docs_type CHECK (document_type IN ('contract','nda','msa','dpa','agreement','legal_notice','claim','court_document','tax_notice','government_doc','other')),
    CONSTRAINT email_legal_docs_status CHECK (status IN ('active','expired','pending_review','signed','terminated','draft')),
    CONSTRAINT email_legal_docs_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);
