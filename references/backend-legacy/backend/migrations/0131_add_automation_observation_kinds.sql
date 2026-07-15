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
        'observation_kind:v1:automation_template',
        'AUTOMATION_TEMPLATE',
        'Automation template',
        1,
        'automation',
        'Automation template configuration captured as canonical evidence.'
    ),
    (
        'observation_kind:v1:automation_policy',
        'AUTOMATION_POLICY',
        'Automation policy',
        1,
        'automation',
        'Automation policy configuration captured as canonical evidence.'
    ),
    (
        'observation_kind:v1:telegram_outbound_message',
        'TELEGRAM_OUTBOUND_MESSAGE',
        'Telegram outbound message',
        1,
        'automation',
        'Automation dry-run or live outbound Telegram message materialization captured as canonical evidence.'
    )
ON CONFLICT (kind_definition_id) DO UPDATE SET
    code = EXCLUDED.code,
    name = EXCLUDED.name,
    version = EXCLUDED.version,
    category = EXCLUDED.category,
    description = EXCLUDED.description,
    updated_at = now();
