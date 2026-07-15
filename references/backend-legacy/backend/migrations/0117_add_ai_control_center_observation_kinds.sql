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
        'observation_kind:v1:ai_provider_account',
        'AI_PROVIDER_ACCOUNT',
        'AI provider account',
        1,
        'ai',
        'AI control center provider account configuration captured as evidence.'
    ),
    (
        'observation_kind:v1:ai_provider_secret_binding',
        'AI_PROVIDER_SECRET_BINDING',
        'AI provider secret binding',
        1,
        'ai',
        'AI control center provider secret binding captured as evidence.'
    ),
    (
        'observation_kind:v1:ai_model_route',
        'AI_MODEL_ROUTE',
        'AI model route',
        1,
        'ai',
        'AI control center capability-slot routing captured as evidence.'
    )
ON CONFLICT (kind_definition_id) DO UPDATE SET
    code = EXCLUDED.code,
    name = EXCLUDED.name,
    version = EXCLUDED.version,
    category = EXCLUDED.category,
    description = EXCLUDED.description,
    updated_at = now();
