INSERT INTO observation_kind_definitions (
    kind_definition_id,
    code,
    name,
    version,
    category,
    description
)
VALUES (
    'observation_kind:v1:person_mutation',
    'PERSON_MUTATION',
    'Person mutation',
    1,
    'persons',
    'Canonical evidence describing a manual mutation of a persona or person-centric profile state such as owner assignment, persona update, favorite toggle, or watchlist toggle.'
)
ON CONFLICT (kind_definition_id) DO UPDATE SET
    code = EXCLUDED.code,
    name = EXCLUDED.name,
    version = EXCLUDED.version,
    category = EXCLUDED.category,
    description = EXCLUDED.description,
    updated_at = now();
