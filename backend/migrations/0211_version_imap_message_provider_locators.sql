-- IMAP UIDs are only unique within a mailbox UIDVALIDITY epoch. Keep existing
-- canonical message IDs stable while moving their current provider locators to
-- the versioned representation used by new IMAP syncs.

UPDATE communication_messages AS message
SET provider_record_id = format(
    'imap:v2:imap:%s:%s:%s',
    replace(replace(message.message_metadata->>'mailbox', '%', '%25'), ':', '%3A'),
    message.message_metadata->>'uid_validity',
    message.message_metadata->>'uid'
)
WHERE message.message_metadata->>'transport' = 'imap'
  AND message.message_metadata->>'mailbox' IS NOT NULL
  AND message.message_metadata->>'mailbox' <> ''
  AND message.message_metadata->>'uid_validity' ~ '^[1-9][0-9]*$'
  AND message.message_metadata->>'uid' ~ '^[1-9][0-9]*$'
  AND message.provider_record_id NOT LIKE 'imap:v2:%';
