INSERT INTO observation_kind_definitions (
    kind_definition_id,
    code,
    name,
    version,
    category,
    description
)
VALUES (
    'observation_kind:v1:organization_record_mutation',
    'ORGANIZATION_RECORD_MUTATION',
    'Organization record mutation',
    1,
    'organizations',
    'Canonical evidence describing a manual mutation of subordinate organization records such as identities, aliases, departments, or organization contact links.'
)
ON CONFLICT (kind_definition_id) DO UPDATE SET
    code = EXCLUDED.code,
    name = EXCLUDED.name,
    version = EXCLUDED.version,
    category = EXCLUDED.category,
    description = EXCLUDED.description,
    updated_at = now();
