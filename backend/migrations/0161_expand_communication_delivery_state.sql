ALTER TABLE communication_messages
    DROP CONSTRAINT IF EXISTS communication_messages_delivery_state;

ALTER TABLE communication_messages
    ADD CONSTRAINT communication_messages_delivery_state CHECK (
        delivery_state IN (
            'received',
            'sent',
            'delivered',
            'read',
            'played',
            'send_dry_run',
            'send_blocked'
        )
    );
