INSERT INTO observation_kind_definitions (
    kind_definition_id,
    code,
    name,
    version,
    category,
    description
)
VALUES (
    'observation_kind:v1:person_record_mutation',
    'PERSON_RECORD_MUTATION',
    'Person record mutation',
    1,
    'persons',
    'Canonical evidence describing a manual mutation of subordinate person records such as identity traces, identities, compatibility roles, compatibility personas, facts, preferences, or relationship timeline events.'
)
ON CONFLICT (kind_definition_id) DO UPDATE SET
    code = EXCLUDED.code,
    name = EXCLUDED.name,
    version = EXCLUDED.version,
    category = EXCLUDED.category,
    description = EXCLUDED.description,
    updated_at = now();
