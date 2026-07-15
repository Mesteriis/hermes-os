CREATE TABLE IF NOT EXISTS email_invoices (
    invoice_id TEXT PRIMARY KEY,
    message_id TEXT,
    amount DOUBLE PRECISION,
    currency TEXT,
    invoice_number TEXT,
    issue_date TIMESTAMPTZ,
    due_date TIMESTAMPTZ,
    counterparty TEXT,
    tax_id TEXT,
    status TEXT NOT NULL DEFAULT 'received',
    linked_project_id TEXT,
    linked_contact_id TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT email_invoices_status CHECK (status IN ('received','recognized','needs_review','approved','paid','closed','rejected')),
    CONSTRAINT email_invoices_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);
CREATE INDEX IF NOT EXISTS email_invoices_status_idx ON email_invoices (status, due_date);
