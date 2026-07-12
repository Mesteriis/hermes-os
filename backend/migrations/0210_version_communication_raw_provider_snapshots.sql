-- Raw provider snapshots remain immutable. A provider message can legitimately
-- change labels, flags or folders, so identity is no longer unique per record.
ALTER TABLE communication_raw_records
    DROP CONSTRAINT IF EXISTS communication_raw_provider_identity_unique;

CREATE INDEX IF NOT EXISTS communication_raw_records_provider_snapshot_idx
    ON communication_raw_records (
        account_id, record_kind, provider_record_id, captured_at DESC, raw_record_id DESC
    );
