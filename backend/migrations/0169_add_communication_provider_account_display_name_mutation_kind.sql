INSERT INTO observation_kind_definitions (
    kind_definition_id,
    code,
    name,
    version,
    category,
    description
)
VALUES (
    'observation_kind:v1:communication_provider_account_display_name_mutation',
    'COMMUNICATION_PROVIDER_ACCOUNT_DISPLAY_NAME_MUTATION',
    'Communication provider account display name mutation',
    1,
    'vault',
    'Canonical evidence describing an update of vault-owned communication provider account display metadata.'
)
ON CONFLICT (kind_definition_id) DO UPDATE SET
    code = EXCLUDED.code,
    name = EXCLUDED.name,
    version = EXCLUDED.version,
    category = EXCLUDED.category,
    description = EXCLUDED.description,
    updated_at = now();
