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
    'observation_kind:v1:communication_provider_account_deleted',
    'COMMUNICATION_PROVIDER_ACCOUNT_DELETED',
    'Communication provider account deleted',
    1,
    'vault',
    'Canonical evidence describing deletion of vault-owned communication provider account metadata.'
),
(
    'observation_kind:v1:communication_provider_secret_binding_removed',
    'COMMUNICATION_PROVIDER_SECRET_BINDING_REMOVED',
    'Communication provider secret binding removed',
    1,
    'vault',
    'Canonical evidence describing removal of a vault-owned communication provider secret binding during metadata cleanup.'
)
ON CONFLICT (kind_definition_id) DO UPDATE SET
    code = EXCLUDED.code,
    name = EXCLUDED.name,
    version = EXCLUDED.version,
    category = EXCLUDED.category,
    description = EXCLUDED.description,
    updated_at = now();
