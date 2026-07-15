-- Persona domain naming alignment.
--
-- Evidence/review domain links should use the product domain name before and
-- after the physical Persona table rename.

INSERT INTO observation_links (
    observation_id,
    domain,
    entity_kind,
    entity_id,
    relationship_kind,
    confidence,
    metadata,
    created_at
)
SELECT
    observation_id,
    'personas',
    entity_kind,
    entity_id,
    relationship_kind,
    confidence,
    metadata,
    created_at
FROM observation_links
WHERE domain = 'persons'
ON CONFLICT (observation_id, domain, entity_kind, entity_id, relationship_kind)
DO UPDATE SET
    confidence = GREATEST(observation_links.confidence, EXCLUDED.confidence),
    metadata = observation_links.metadata || EXCLUDED.metadata;

DELETE FROM observation_links
WHERE domain = 'persons';

UPDATE review_items
SET target_domain = 'personas'
WHERE target_domain = 'persons';
