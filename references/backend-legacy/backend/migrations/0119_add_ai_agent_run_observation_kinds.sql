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
        'observation_kind:v1:ai_agent_run',
        'AI_AGENT_RUN',
        'AI agent run',
        1,
        'ai',
        'AI agent run request captured as durable execution evidence.'
    ),
    (
        'observation_kind:v1:ai_agent_run_status',
        'AI_AGENT_RUN_STATUS',
        'AI agent run status',
        1,
        'ai',
        'AI agent run lifecycle state captured as durable execution evidence.'
    )
ON CONFLICT (kind_definition_id) DO UPDATE SET
    code = EXCLUDED.code,
    name = EXCLUDED.name,
    version = EXCLUDED.version,
    category = EXCLUDED.category,
    description = EXCLUDED.description,
    updated_at = now();
