-- Migration 0206: Durable attachment references for Mail drafts and outbox items.
-- Attachment bytes remain in local blob storage through communication_attachment_imports.

CREATE TABLE IF NOT EXISTS communication_draft_attachments (
    draft_id TEXT NOT NULL REFERENCES communication_drafts(draft_id) ON DELETE CASCADE,
    attachment_id TEXT NOT NULL REFERENCES communication_attachment_imports(attachment_id) ON DELETE RESTRICT,
    disposition TEXT NOT NULL DEFAULT 'attachment',
    content_id TEXT,
    sort_order INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (draft_id, attachment_id),
    CONSTRAINT communication_draft_attachments_disposition
        CHECK (disposition IN ('attachment', 'inline')),
    CONSTRAINT communication_draft_attachments_content_id_not_empty
        CHECK (content_id IS NULL OR length(trim(content_id)) > 0),
    CONSTRAINT communication_draft_attachments_sort_order_non_negative
        CHECK (sort_order >= 0),
    CONSTRAINT communication_draft_attachments_sort_order_unique
        UNIQUE (draft_id, sort_order)
);

CREATE INDEX IF NOT EXISTS communication_draft_attachments_attachment_idx
    ON communication_draft_attachments (attachment_id);

CREATE TABLE IF NOT EXISTS communication_outbox_attachments (
    outbox_id TEXT NOT NULL REFERENCES communication_outbox(outbox_id) ON DELETE CASCADE,
    attachment_id TEXT NOT NULL REFERENCES communication_attachment_imports(attachment_id) ON DELETE RESTRICT,
    disposition TEXT NOT NULL DEFAULT 'attachment',
    content_id TEXT,
    sort_order INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (outbox_id, attachment_id),
    CONSTRAINT communication_outbox_attachments_disposition
        CHECK (disposition IN ('attachment', 'inline')),
    CONSTRAINT communication_outbox_attachments_content_id_not_empty
        CHECK (content_id IS NULL OR length(trim(content_id)) > 0),
    CONSTRAINT communication_outbox_attachments_sort_order_non_negative
        CHECK (sort_order >= 0),
    CONSTRAINT communication_outbox_attachments_sort_order_unique
        UNIQUE (outbox_id, sort_order)
);

CREATE INDEX IF NOT EXISTS communication_outbox_attachments_attachment_idx
    ON communication_outbox_attachments (attachment_id);
