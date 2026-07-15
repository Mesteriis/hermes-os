-- Persona observation kind naming alignment.
--
-- Keep old kind definitions for historical compatibility, but move existing
-- observations and all new writers to Persona-native kind ids/codes.

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
        'observation_kind:v1:persona_mutation',
        'PERSONA_MUTATION',
        'Persona mutation',
        1,
        'personas',
        'Canonical evidence describing a Persona root mutation.'
    ),
    (
        'observation_kind:v1:persona_record_mutation',
        'PERSONA_RECORD_MUTATION',
        'Persona record mutation',
        1,
        'personas',
        'Canonical evidence describing a manual mutation of Persona subordinate records such as identity traces, identities, compatibility roles, interaction contexts, facts, preferences, or relationship timeline events.'
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
SET kind_definition_id = 'observation_kind:v1:persona_mutation'
WHERE kind_definition_id = 'observation_kind:v1:person_mutation';

UPDATE observations
SET kind_definition_id = 'observation_kind:v1:persona_record_mutation'
WHERE kind_definition_id = 'observation_kind:v1:person_record_mutation';

ALTER TABLE observations ENABLE TRIGGER observations_append_only_update;
