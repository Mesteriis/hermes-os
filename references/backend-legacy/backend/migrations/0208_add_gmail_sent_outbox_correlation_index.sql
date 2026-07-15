-- Gmail returns the same provider message ID on send and subsequent sync.
-- Keep sent-message correlation bounded to the account and efficient during projection.
CREATE INDEX IF NOT EXISTS communication_outbox_gmail_sent_correlation_idx
    ON communication_outbox (account_id, provider_message_id, sent_at DESC)
    WHERE status = 'sent' AND provider_message_id IS NOT NULL;
