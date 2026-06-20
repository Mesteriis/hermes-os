CREATE TABLE IF NOT EXISTS communication_read_receipts (
    receipt_id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL REFERENCES communication_accounts(account_id) ON DELETE CASCADE,
    outbox_id TEXT REFERENCES communication_outbox(outbox_id) ON DELETE SET NULL,
    provider_message_id TEXT NOT NULL,
    recipient TEXT NOT NULL,
    receipt_kind TEXT NOT NULL DEFAULT 'read',
    read_at TIMESTAMPTZ NOT NULL,
    source_kind TEXT NOT NULL DEFAULT 'mdn',
    provider_record_id TEXT,
    raw_record_id TEXT REFERENCES communication_raw_records(raw_record_id) ON DELETE SET NULL,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT communication_read_receipts_id_not_empty CHECK (length(trim(receipt_id)) > 0),
    CONSTRAINT communication_read_receipts_provider_message_not_empty CHECK (
        length(trim(provider_message_id)) > 0
    ),
    CONSTRAINT communication_read_receipts_recipient_not_empty CHECK (length(trim(recipient)) > 0),
    CONSTRAINT communication_read_receipts_kind CHECK (receipt_kind IN ('read')),
    CONSTRAINT communication_read_receipts_source_kind_not_empty CHECK (length(trim(source_kind)) > 0),
    CONSTRAINT communication_read_receipts_provider_record_not_empty CHECK (
        provider_record_id IS NULL OR length(trim(provider_record_id)) > 0
    ),
    CONSTRAINT communication_read_receipts_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE UNIQUE INDEX IF NOT EXISTS communication_read_receipts_provider_record_unique_idx
    ON communication_read_receipts (account_id, provider_record_id)
    WHERE provider_record_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS communication_read_receipts_outbox_read_at_idx
    ON communication_read_receipts (outbox_id, read_at DESC, receipt_id);

CREATE INDEX IF NOT EXISTS communication_read_receipts_provider_message_idx
    ON communication_read_receipts (account_id, provider_message_id, read_at DESC);

INSERT INTO communication_read_receipts (
    receipt_id,
    account_id,
    outbox_id,
    provider_message_id,
    recipient,
    receipt_kind,
    read_at,
    source_kind,
    provider_record_id,
    raw_record_id,
    metadata,
    created_at
)
SELECT
    receipt.receipt_id,
    receipt.account_id,
    receipt.outbox_id,
    receipt.provider_message_id,
    receipt.recipient,
    receipt.receipt_kind,
    receipt.read_at,
    receipt.source_kind,
    receipt.provider_record_id,
    receipt.raw_record_id,
    receipt.metadata || jsonb_build_object('source_table', 'mail_read_receipts'),
    receipt.created_at
FROM mail_read_receipts receipt
ON CONFLICT (receipt_id) DO NOTHING;
