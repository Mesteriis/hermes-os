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
        'observation_kind:v1:telegram_provider_write_command',
        'TELEGRAM_PROVIDER_WRITE_COMMAND',
        'Telegram provider write command',
        1,
        'telegram',
        'Telegram provider write command queued as durable action evidence.'
    ),
    (
        'observation_kind:v1:telegram_provider_write_command_status',
        'TELEGRAM_PROVIDER_WRITE_COMMAND_STATUS',
        'Telegram provider write command status',
        1,
        'telegram',
        'Telegram provider write command lifecycle or reconciliation state captured as evidence.'
    )
ON CONFLICT (kind_definition_id) DO UPDATE SET
    code = EXCLUDED.code,
    name = EXCLUDED.name,
    version = EXCLUDED.version,
    category = EXCLUDED.category,
    description = EXCLUDED.description,
    updated_at = now();
