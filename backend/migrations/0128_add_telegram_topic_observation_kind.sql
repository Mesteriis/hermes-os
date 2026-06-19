INSERT INTO observation_kind_definitions (
    kind_definition_id,
    code,
    name,
    version,
    category,
    description
)
VALUES (
    'okd_telegram_topic_v1',
    'TELEGRAM_TOPIC',
    'Telegram Topic',
    1,
    'telegram',
    'Canonical evidence for Telegram forum topic materialization.'
)
ON CONFLICT (code, version) DO NOTHING;
