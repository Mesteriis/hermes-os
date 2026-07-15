-- Rename identity-candidate observation kind to Persona terminology.

INSERT INTO observation_kind_definitions (
    kind_definition_id,
    code,
    name,
    version,
    category,
    description
)
VALUES (
    'observation_kind:v1:persona_identity_candidate',
    'PERSONA_IDENTITY_CANDIDATE',
    'Persona identity candidate',
    1,
    'identity',
    'Synthetic but canonical evidence describing a Persona identity candidate generated for review and promotion workflows.'
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
SET kind_definition_id = 'observation_kind:v1:persona_identity_candidate'
WHERE kind_definition_id = 'observation_kind:v1:person_identity_candidate';

ALTER TABLE observations ENABLE TRIGGER observations_append_only_update;
