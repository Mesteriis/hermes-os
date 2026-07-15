CREATE TABLE IF NOT EXISTS email_drafts (
    draft_id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL REFERENCES communication_provider_accounts(account_id) ON DELETE CASCADE,
    persona_id TEXT,
    to_recipients JSONB NOT NULL DEFAULT '[]'::jsonb,
    cc_recipients JSONB NOT NULL DEFAULT '[]'::jsonb,
    bcc_recipients JSONB NOT NULL DEFAULT '[]'::jsonb,
    subject TEXT NOT NULL,
    body_text TEXT NOT NULL DEFAULT '',
    body_html TEXT,
    in_reply_to TEXT,
    message_references JSONB NOT NULL DEFAULT '[]'::jsonb,
    status TEXT NOT NULL DEFAULT 'draft',
    scheduled_send_at TIMESTAMPTZ,
    send_attempts INTEGER NOT NULL DEFAULT 0,
    last_error TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT email_drafts_subject_not_empty CHECK (length(trim(subject)) > 0),
    CONSTRAINT email_drafts_status CHECK (status IN ('draft', 'scheduled', 'sending', 'sent', 'failed')),
    CONSTRAINT email_drafts_to_is_array CHECK (jsonb_typeof(to_recipients) = 'array'),
    CONSTRAINT email_drafts_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);
CREATE INDEX IF NOT EXISTS email_drafts_account_status_idx ON email_drafts (account_id, status, updated_at DESC);
