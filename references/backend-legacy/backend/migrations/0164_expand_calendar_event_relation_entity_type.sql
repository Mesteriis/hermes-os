ALTER TABLE event_relations
DROP CONSTRAINT IF EXISTS event_relations_entity_type_check;

ALTER TABLE event_relations
ADD CONSTRAINT event_relations_entity_type_check CHECK (
    entity_type IN (
        'person',
        'organization',
        'project',
        'document',
        'task',
        'email',
        'note',
        'decision',
        'obligation',
        'recording',
        'call'
    )
);
