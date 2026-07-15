-- Rename legacy contact-record evidence to Persona terminology.
--
-- Address-book/provider contact payloads are evidence for Persona identity and
-- address-book membership. They are not a durable Contact domain entity.

INSERT INTO observation_kind_definitions (
    kind_definition_id,
    code,
    name,
    version,
    category,
    description
)
VALUES (
    'observation_kind:v1:persona_record',
    'PERSONA_RECORD',
    'Persona record',
    1,
    'identity',
    'Provider address-book or identity evidence captured for Persona review and projection workflows.'
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
SET kind_definition_id = 'observation_kind:v1:persona_record'
WHERE kind_definition_id = 'observation_kind:v1:contact_record';

ALTER TABLE observations ENABLE TRIGGER observations_append_only_update;
