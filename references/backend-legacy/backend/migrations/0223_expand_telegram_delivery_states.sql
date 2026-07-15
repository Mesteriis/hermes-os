-- TDLib reports a local outgoing message as queued before the server assigns
-- its final provider id, and may later report a send failure.
ALTER TABLE communication_messages
    DROP CONSTRAINT IF EXISTS communication_messages_delivery_state;

ALTER TABLE communication_messages
    ADD CONSTRAINT communication_messages_delivery_state CHECK (
        delivery_state IN (
            'received',
            'queued',
            'sent',
            'send_failed',
            'delivered',
            'read',
            'played',
            'send_dry_run',
            'send_blocked'
        )
    );
