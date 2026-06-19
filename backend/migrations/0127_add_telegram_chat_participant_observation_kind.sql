INSERT INTO observation_kind_definitions (
    kind_definition_id,
    code,
    name,
    version,
    category,
    description
)
VALUES (
    'okd_telegram_chat_participant_v1',
    'TELEGRAM_CHAT_PARTICIPANT',
    'Telegram Chat Participant',
    1,
    'telegram',
    'Canonical evidence for Telegram chat participant roster materialization.'
)
ON CONFLICT (code, version) DO NOTHING;
