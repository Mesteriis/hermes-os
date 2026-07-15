INSERT INTO observation_kind_definitions (
    kind_definition_id,
    code,
    name,
    version,
    category,
    description
)
VALUES (
    'observation_kind:v1:communication_provider_account_config_mutation',
    'COMMUNICATION_PROVIDER_ACCOUNT_CONFIG_MUTATION',
    'Communication provider account config mutation',
    1,
    'vault',
    'Canonical evidence describing an update of vault-owned communication provider account config metadata.'
)
ON CONFLICT (kind_definition_id) DO UPDATE SET
    code = EXCLUDED.code,
    name = EXCLUDED.name,
    version = EXCLUDED.version,
    category = EXCLUDED.category,
    description = EXCLUDED.description,
    updated_at = now();
