INSERT INTO observation_kind_definitions (
    kind_definition_id,
    code,
    name,
    version,
    category,
    description
)
VALUES (
    'observation_kind:v1:realtime_conversation_radar_signal',
    'REALTIME_CONVERSATION_RADAR_SIGNAL',
    'Realtime conversation radar signal',
    1,
    'meeting',
    'Provider-neutral realtime conversation radar candidate captured from a local or provider runtime before owner review and promotion.'
)
ON CONFLICT (kind_definition_id) DO UPDATE SET
    code = EXCLUDED.code,
    name = EXCLUDED.name,
    version = EXCLUDED.version,
    category = EXCLUDED.category,
    description = EXCLUDED.description,
    updated_at = now();
