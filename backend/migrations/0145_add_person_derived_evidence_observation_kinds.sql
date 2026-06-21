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
    'observation_kind:v1:person_role',
    'PERSON_ROLE',
    'Person role',
    1,
    'persons',
    'Canonical evidence describing a person role assignment or removal materialized as compatibility knowledge and relationship evidence.'
),
(
    'observation_kind:v1:person_trust_signal',
    'PERSON_TRUST_SIGNAL',
    'Person trust signal',
    1,
    'persons',
    'Canonical evidence describing a derived trust signal for a persona relationship materialized from person enrichment.'
),
(
    'observation_kind:v1:person_promise',
    'PERSON_PROMISE',
    'Person promise',
    1,
    'persons',
    'Canonical evidence describing a persona promise that is projected into an obligation.'
)
ON CONFLICT (kind_definition_id) DO UPDATE SET
    code = EXCLUDED.code,
    name = EXCLUDED.name,
    version = EXCLUDED.version,
    category = EXCLUDED.category,
    description = EXCLUDED.description,
    updated_at = now();
