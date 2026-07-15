CREATE TABLE IF NOT EXISTS communication_sender_reputation (
    sender_key TEXT NOT NULL,
    sender_domain TEXT NOT NULL,
    score SMALLINT NOT NULL DEFAULT 100,
    spam_count INTEGER NOT NULL DEFAULT 0,
    non_spam_count INTEGER NOT NULL DEFAULT 0,
    suppressed_until TIMESTAMPTZ,
    last_reason TEXT,
    last_message_id TEXT REFERENCES communication_messages(message_id) ON DELETE SET NULL,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    first_seen_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    PRIMARY KEY (sender_key, sender_domain),
    CONSTRAINT communication_sender_reputation_sender_key_not_empty CHECK (length(trim(sender_key)) > 0),
    CONSTRAINT communication_sender_reputation_sender_domain_not_empty CHECK (length(trim(sender_domain)) > 0),
    CONSTRAINT communication_sender_reputation_score_range CHECK (score >= 0 AND score <= 100),
    CONSTRAINT communication_sender_reputation_spam_count_non_negative CHECK (spam_count >= 0),
    CONSTRAINT communication_sender_reputation_non_spam_count_non_negative CHECK (non_spam_count >= 0),
    CONSTRAINT communication_sender_reputation_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE INDEX IF NOT EXISTS communication_sender_reputation_score_idx
    ON communication_sender_reputation (score, suppressed_until, last_seen_at DESC);

CREATE INDEX IF NOT EXISTS communication_sender_reputation_domain_idx
    ON communication_sender_reputation (sender_domain, score, last_seen_at DESC);
