ALTER TABLE communication_attachments
    ADD COLUMN scan_status TEXT NOT NULL DEFAULT 'not_scanned',
    ADD COLUMN scan_engine TEXT,
    ADD COLUMN scan_checked_at TIMESTAMPTZ,
    ADD COLUMN scan_summary TEXT,
    ADD COLUMN scan_metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    ADD CONSTRAINT communication_attachment_scan_status CHECK (
        scan_status IN ('not_scanned', 'clean', 'suspicious', 'malicious', 'failed')
    ),
    ADD CONSTRAINT communication_attachment_scan_engine_not_empty CHECK (
        scan_engine IS NULL OR length(trim(scan_engine)) > 0
    ),
    ADD CONSTRAINT communication_attachment_scan_summary_not_empty CHECK (
        scan_summary IS NULL OR length(trim(scan_summary)) > 0
    ),
    ADD CONSTRAINT communication_attachment_scan_metadata_object CHECK (
        jsonb_typeof(scan_metadata) = 'object'
    );
