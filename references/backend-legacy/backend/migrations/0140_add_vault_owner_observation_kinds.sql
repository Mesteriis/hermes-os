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
    'observation_kind:v1:calendar_account_link',
    'CALENDAR_ACCOUNT_LINK',
    'Calendar account link',
    1,
    'vault',
    'Canonical evidence describing a linked provider calendar account materialized through the vault owner boundary.'
),
(
    'observation_kind:v1:task_provider_account',
    'TASK_PROVIDER_ACCOUNT',
    'Task provider account',
    1,
    'vault',
    'Canonical evidence describing creation of a vault-owned task provider account.'
),
(
    'observation_kind:v1:communication_provider_account',
    'COMMUNICATION_PROVIDER_ACCOUNT',
    'Communication provider account',
    1,
    'vault',
    'Canonical evidence describing an upsert of a vault-owned communication provider account.'
),
(
    'observation_kind:v1:communication_provider_secret_binding',
    'COMMUNICATION_PROVIDER_SECRET_BINDING',
    'Communication provider secret binding',
    1,
    'vault',
    'Canonical evidence describing a vault-owned communication provider account secret binding mutation.'
)
ON CONFLICT (kind_definition_id) DO UPDATE SET
    code = EXCLUDED.code,
    name = EXCLUDED.name,
    version = EXCLUDED.version,
    category = EXCLUDED.category,
    description = EXCLUDED.description,
    updated_at = now();
