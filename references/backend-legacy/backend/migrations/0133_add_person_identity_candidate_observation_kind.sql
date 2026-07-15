INSERT INTO observation_kind_definitions (
    kind_definition_id,
    code,
    name,
    version,
    category,
    description
)
VALUES (
    'observation_kind:v1:person_identity_candidate',
    'PERSON_IDENTITY_CANDIDATE',
    'Person identity candidate',
    1,
    'identity',
    'Synthetic but canonical evidence describing a person identity candidate generated for review and promotion workflows.'
)
ON CONFLICT (kind_definition_id) DO UPDATE SET
    code = EXCLUDED.code,
    name = EXCLUDED.name,
    version = EXCLUDED.version,
    category = EXCLUDED.category,
    description = EXCLUDED.description,
    updated_at = now();
