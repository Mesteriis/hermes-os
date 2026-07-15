INSERT INTO observation_kind_definitions (
    kind_definition_id,
    code,
    name,
    version,
    category,
    description
)
VALUES (
    'okd_telegram_chat_v1',
    'TELEGRAM_CHAT',
    'Telegram Chat',
    1,
    'telegram',
    'Canonical evidence for Telegram chat state materialization.'
)
ON CONFLICT (code, version) DO NOTHING;
