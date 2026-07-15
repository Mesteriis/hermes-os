CREATE TABLE IF NOT EXISTS email_outbox_tracking (
    outbox_id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL REFERENCES communication_provider_accounts(account_id) ON DELETE CASCADE,
    draft_id TEXT REFERENCES email_drafts(draft_id) ON DELETE SET NULL,
    to_recipients JSONB NOT NULL DEFAULT '[]'::jsonb,
    cc_recipients JSONB NOT NULL DEFAULT '[]'::jsonb,
    bcc_recipients JSONB NOT NULL DEFAULT '[]'::jsonb,
    subject TEXT NOT NULL DEFAULT '',
    body_text TEXT NOT NULL DEFAULT '',
    body_html TEXT,
    status TEXT NOT NULL,
    scheduled_send_at TIMESTAMPTZ,
    undo_deadline_at TIMESTAMPTZ,
    send_attempts INTEGER NOT NULL DEFAULT 0,
    claimed_at TIMESTAMPTZ,
    sent_at TIMESTAMPTZ,
    provider_message_id TEXT,
    last_error TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT email_outbox_tracking_status CHECK (
        status IN ('queued', 'scheduled', 'sending', 'sent', 'failed', 'canceled')
    ),
    CONSTRAINT email_outbox_tracking_to_is_array CHECK (jsonb_typeof(to_recipients) = 'array'),
    CONSTRAINT email_outbox_tracking_cc_is_array CHECK (jsonb_typeof(cc_recipients) = 'array'),
    CONSTRAINT email_outbox_tracking_bcc_is_array CHECK (jsonb_typeof(bcc_recipients) = 'array'),
    CONSTRAINT email_outbox_tracking_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE INDEX IF NOT EXISTS email_outbox_tracking_account_status_idx
    ON email_outbox_tracking (account_id, status, created_at DESC);

CREATE INDEX IF NOT EXISTS email_outbox_tracking_due_idx
    ON email_outbox_tracking (status, scheduled_send_at, undo_deadline_at, created_at);
