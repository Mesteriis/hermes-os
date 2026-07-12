-- Replace historical mailbox-name guesses with the explicit IMAP/iCloud
-- provider-resource mapping seeded by configuration or SPECIAL-USE discovery.
WITH reconciled AS (
    SELECT
        message.message_id,
        CASE
            WHEN EXISTS (
                SELECT 1
                FROM communication_mail_provider_resources AS resource
                WHERE resource.account_id = message.account_id
                  AND resource.resource_kind = 'folder'
                  AND resource.provider_resource_id = message.message_metadata->>'mailbox'
                  AND resource.semantic_role = 'sent'
            )
            THEN 'sent'
            ELSE 'received'
        END AS delivery_state
    FROM communication_messages AS message
    WHERE message.channel_kind = 'email'
      AND message.message_metadata->>'transport' = 'imap'
)
UPDATE communication_messages AS message
SET delivery_state = reconciled.delivery_state,
    projected_at = now()
FROM reconciled
WHERE message.message_id = reconciled.message_id
  AND message.delivery_state IS DISTINCT FROM reconciled.delivery_state;
