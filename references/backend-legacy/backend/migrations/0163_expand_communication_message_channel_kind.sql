ALTER TABLE communication_messages
    DROP CONSTRAINT IF EXISTS communication_messages_channel_kind;

ALTER TABLE communication_messages
    ADD CONSTRAINT communication_messages_channel_kind CHECK (
        channel_kind IN (
            'email',
            'telegram_user',
            'telegram_bot',
            'whatsapp_web',
            'whatsapp_business_cloud'
        )
    );
