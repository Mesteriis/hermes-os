INSERT INTO observation_kind_definitions (
    kind_definition_id,
    code,
    name,
    version,
    category,
    description
)
VALUES (
    'observation_kind:v1:task_mutation',
    'TASK_MUTATION',
    'Task mutation',
    1,
    'tasks',
    'Canonical evidence describing a manual or local-runtime task mutation, task-local record change, or compatibility task materialization.'
)
ON CONFLICT (kind_definition_id) DO UPDATE SET
    code = EXCLUDED.code,
    name = EXCLUDED.name,
    version = EXCLUDED.version,
    category = EXCLUDED.category,
    description = EXCLUDED.description,
    updated_at = now();
