INSERT INTO observation_kind_definitions (
    kind_definition_id,
    code,
    name,
    version,
    category,
    description
)
VALUES
(
    'okd_telegram_message_version_v1',
    'TELEGRAM_MESSAGE_VERSION',
    'Telegram Message Version',
    1,
    'telegram',
    'Canonical evidence for append-only Telegram message edit versions.'
),
(
    'okd_telegram_message_tombstone_v1',
    'TELEGRAM_MESSAGE_TOMBSTONE',
    'Telegram Message Tombstone',
    1,
    'telegram',
    'Canonical evidence for append-only Telegram message tombstones and visibility deletions.'
)
ON CONFLICT (code, version) DO NOTHING;
