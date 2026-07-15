ALTER TABLE communication_messages
    DROP CONSTRAINT IF EXISTS communication_messages_body_not_empty;

ALTER TABLE communication_messages
    ADD CONSTRAINT communication_messages_body_not_empty CHECK (
        length(trim(body_text)) > 0
        OR (
            channel_kind IN ('telegram_user', 'telegram_bot')
            AND jsonb_typeof(message_metadata) = 'object'
            AND message_metadata ? 'tdlib_raw'
        )
    );
