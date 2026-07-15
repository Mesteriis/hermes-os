CREATE INDEX IF NOT EXISTS communication_attachments_search_order_idx
    ON communication_attachments (created_at DESC, attachment_id ASC);

CREATE INDEX IF NOT EXISTS communication_attachments_scan_status_idx
    ON communication_attachments (scan_status, created_at DESC);

CREATE INDEX IF NOT EXISTS communication_attachments_content_type_idx
    ON communication_attachments (content_type, created_at DESC);
