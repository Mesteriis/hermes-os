INSERT INTO observation_kind_definitions (
    kind_definition_id,
    code,
    name,
    version,
    category,
    description
)
VALUES (
    'observation_kind:v1:calendar_account_mutation',
    'CALENDAR_ACCOUNT_MUTATION',
    'Calendar account mutation',
    1,
    'calendar',
    'Canonical evidence describing a manual mutation of a calendar account aggregate such as create, update, delete, or sync trigger.'
)
ON CONFLICT (kind_definition_id) DO UPDATE SET
    code = EXCLUDED.code,
    name = EXCLUDED.name,
    version = EXCLUDED.version,
    category = EXCLUDED.category,
    description = EXCLUDED.description,
    updated_at = now();
