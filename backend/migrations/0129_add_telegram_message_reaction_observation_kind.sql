INSERT INTO observation_kind_definitions (
    kind_definition_id,
    code,
    name,
    version,
    category,
    description
)
VALUES (
    'okd_telegram_message_reaction_v1',
    'TELEGRAM_MESSAGE_REACTION',
    'Telegram Message Reaction',
    1,
    'telegram',
    'Canonical evidence for Telegram message reaction state materialization.'
)
ON CONFLICT (code, version) DO NOTHING;
