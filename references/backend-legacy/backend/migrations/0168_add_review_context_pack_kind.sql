ALTER TABLE context_packs
    DROP CONSTRAINT IF EXISTS context_packs_kind;

ALTER TABLE context_packs
    ADD CONSTRAINT context_packs_kind CHECK (
        kind IN ('persona', 'meeting', 'task', 'calendar', 'project', 'review')
    );

ALTER TABLE context_pack_sources
    DROP CONSTRAINT IF EXISTS context_pack_sources_kind;

ALTER TABLE context_pack_sources
    ADD CONSTRAINT context_pack_sources_kind CHECK (
        source_kind IN (
            'observation',
            'domain_entity',
            'knowledge',
            'relationship',
            'decision',
            'task',
            'obligation',
            'document',
            'calendar_event',
            'project',
            'review_item'
        )
    );
