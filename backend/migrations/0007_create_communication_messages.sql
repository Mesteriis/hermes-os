CREATE TABLE IF NOT EXISTS communication_messages (
    message_id TEXT PRIMARY KEY,
    raw_record_id TEXT NOT NULL REFERENCES communication_raw_records(raw_record_id) ON DELETE RESTRICT,
    account_id TEXT NOT NULL REFERENCES communication_provider_accounts(account_id) ON DELETE RESTRICT,
    provider_record_id TEXT NOT NULL,
    subject TEXT NOT NULL,
    sender TEXT NOT NULL,
    recipients JSONB NOT NULL,
    body_text TEXT NOT NULL,
    occurred_at TIMESTAMPTZ,
    projected_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT communication_messages_subject_not_empty CHECK (length(trim(subject)) > 0),
    CONSTRAINT communication_messages_sender_not_empty CHECK (length(trim(sender)) > 0),
    CONSTRAINT communication_messages_recipients_is_array CHECK (jsonb_typeof(recipients) = 'array'),
    CONSTRAINT communication_messages_body_not_empty CHECK (length(trim(body_text)) > 0),
    UNIQUE (account_id, provider_record_id)
);
