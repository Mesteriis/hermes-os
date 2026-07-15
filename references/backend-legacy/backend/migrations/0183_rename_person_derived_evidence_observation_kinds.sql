-- Persona derived evidence observation kind naming alignment.
--
-- Old Person-named definitions stay for historical compatibility, but existing
-- observations and new writers use Persona-native kind ids/codes.

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
        'observation_kind:v1:persona_memory_card',
        'PERSONA_MEMORY_CARD',
        'Persona memory card',
        1,
        'personas',
        'Canonical evidence describing a manual Persona memory note or memory card captured into Persona memory.'
    ),
    (
        'observation_kind:v1:persona_role',
        'PERSONA_ROLE',
        'Persona role',
        1,
        'personas',
        'Canonical evidence describing a Persona role compatibility assignment or removal materialized as relationship evidence.'
    ),
    (
        'observation_kind:v1:persona_trust_signal',
        'PERSONA_TRUST_SIGNAL',
        'Persona trust signal',
        1,
        'personas',
        'Canonical evidence describing a derived trust signal for a Persona relationship materialized from Persona enrichment.'
    ),
    (
        'observation_kind:v1:persona_promise',
        'PERSONA_PROMISE',
        'Persona promise',
        1,
        'personas',
        'Canonical evidence describing a Persona promise projected into an obligation.'
    )
ON CONFLICT (kind_definition_id) DO UPDATE SET
    code = EXCLUDED.code,
    name = EXCLUDED.name,
    version = EXCLUDED.version,
    category = EXCLUDED.category,
    description = EXCLUDED.description,
    updated_at = now();

ALTER TABLE observations DISABLE TRIGGER observations_append_only_update;

UPDATE observations
SET kind_definition_id = 'observation_kind:v1:persona_memory_card'
WHERE kind_definition_id = 'observation_kind:v1:person_memory_card';

UPDATE observations
SET kind_definition_id = 'observation_kind:v1:persona_role'
WHERE kind_definition_id = 'observation_kind:v1:person_role';

UPDATE observations
SET kind_definition_id = 'observation_kind:v1:persona_trust_signal'
WHERE kind_definition_id = 'observation_kind:v1:person_trust_signal';

UPDATE observations
SET kind_definition_id = 'observation_kind:v1:persona_promise'
WHERE kind_definition_id = 'observation_kind:v1:person_promise';

ALTER TABLE observations ENABLE TRIGGER observations_append_only_update;
