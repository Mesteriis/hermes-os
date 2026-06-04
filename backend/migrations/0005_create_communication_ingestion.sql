CREATE TABLE IF NOT EXISTS communication_provider_accounts (
    account_id TEXT PRIMARY KEY,
    provider_kind TEXT NOT NULL,
    display_name TEXT NOT NULL,
    external_account_id TEXT NOT NULL,
    config JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT communication_provider_account_kind CHECK (provider_kind IN ('gmail', 'icloud', 'imap')),
    CONSTRAINT communication_provider_account_id_not_empty CHECK (length(trim(account_id)) > 0),
    CONSTRAINT communication_provider_display_name_not_empty CHECK (length(trim(display_name)) > 0),
    CONSTRAINT communication_provider_external_id_not_empty CHECK (length(trim(external_account_id)) > 0),
    CONSTRAINT communication_provider_config_is_object CHECK (jsonb_typeof(config) = 'object'),
    CONSTRAINT communication_provider_external_identity_unique UNIQUE (provider_kind, external_account_id)
);

CREATE INDEX IF NOT EXISTS communication_provider_accounts_kind_idx
    ON communication_provider_accounts (provider_kind, created_at);

CREATE TABLE IF NOT EXISTS communication_raw_records (
    raw_record_id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL REFERENCES communication_provider_accounts(account_id) ON DELETE RESTRICT,
    record_kind TEXT NOT NULL,
    provider_record_id TEXT NOT NULL,
    source_fingerprint TEXT NOT NULL,
    import_batch_id TEXT NOT NULL,
    occurred_at TIMESTAMPTZ,
    captured_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    payload JSONB NOT NULL,
    provenance JSONB NOT NULL DEFAULT '{}'::jsonb,

    CONSTRAINT communication_raw_record_id_not_empty CHECK (length(trim(raw_record_id)) > 0),
    CONSTRAINT communication_raw_record_kind_not_empty CHECK (length(trim(record_kind)) > 0),
    CONSTRAINT communication_raw_provider_record_id_not_empty CHECK (length(trim(provider_record_id)) > 0),
    CONSTRAINT communication_raw_source_fingerprint_not_empty CHECK (length(trim(source_fingerprint)) > 0),
    CONSTRAINT communication_raw_import_batch_id_not_empty CHECK (length(trim(import_batch_id)) > 0),
    CONSTRAINT communication_raw_payload_is_object CHECK (jsonb_typeof(payload) = 'object'),
    CONSTRAINT communication_raw_provenance_is_object CHECK (jsonb_typeof(provenance) = 'object'),
    CONSTRAINT communication_raw_provider_identity_unique UNIQUE (account_id, record_kind, provider_record_id)
);

CREATE INDEX IF NOT EXISTS communication_raw_records_account_idx
    ON communication_raw_records (account_id, captured_at, raw_record_id);

CREATE INDEX IF NOT EXISTS communication_raw_records_import_batch_idx
    ON communication_raw_records (import_batch_id, captured_at);

CREATE TABLE IF NOT EXISTS communication_ingestion_checkpoints (
    account_id TEXT NOT NULL REFERENCES communication_provider_accounts(account_id) ON DELETE RESTRICT,
    stream_id TEXT NOT NULL,
    checkpoint JSONB NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT communication_checkpoint_stream_id_not_empty CHECK (length(trim(stream_id)) > 0),
    CONSTRAINT communication_checkpoint_is_object CHECK (jsonb_typeof(checkpoint) = 'object'),
    PRIMARY KEY (account_id, stream_id)
);

CREATE INDEX IF NOT EXISTS communication_ingestion_checkpoints_updated_at_idx
    ON communication_ingestion_checkpoints (updated_at);

CREATE OR REPLACE FUNCTION prevent_communication_raw_records_mutation()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
    RAISE EXCEPTION 'communication_raw_records is append-only';
END;
$$;

DROP TRIGGER IF EXISTS communication_raw_records_prevent_update ON communication_raw_records;
CREATE TRIGGER communication_raw_records_prevent_update
    BEFORE UPDATE ON communication_raw_records
    FOR EACH ROW
    EXECUTE FUNCTION prevent_communication_raw_records_mutation();

DROP TRIGGER IF EXISTS communication_raw_records_prevent_delete ON communication_raw_records;
CREATE TRIGGER communication_raw_records_prevent_delete
    BEFORE DELETE ON communication_raw_records
    FOR EACH ROW
    EXECUTE FUNCTION prevent_communication_raw_records_mutation();
