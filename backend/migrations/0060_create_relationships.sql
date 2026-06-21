CREATE TABLE IF NOT EXISTS relationships (
    relationship_id TEXT PRIMARY KEY,
    source_entity_kind TEXT NOT NULL,
    source_entity_id TEXT NOT NULL,
    target_entity_kind TEXT NOT NULL,
    target_entity_id TEXT NOT NULL,
    relationship_type TEXT NOT NULL,
    trust_score NUMERIC(5,4) NOT NULL DEFAULT 0.5000,
    strength_score NUMERIC(5,4) NOT NULL DEFAULT 0.5000,
    confidence NUMERIC(5,4) NOT NULL DEFAULT 1.0000,
    review_state TEXT NOT NULL DEFAULT 'suggested',
    valid_from TIMESTAMPTZ,
    valid_to TIMESTAMPTZ,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT relationships_entity_kind_check CHECK (
        source_entity_kind IN (
            'persona',
            'organization',
            'project',
            'communication',
            'document',
            'task',
            'event',
            'decision',
            'obligation',
            'knowledge'
        )
        AND target_entity_kind IN (
            'persona',
            'organization',
            'project',
            'communication',
            'document',
            'task',
            'event',
            'decision',
            'obligation',
            'knowledge'
        )
    ),
    CONSTRAINT relationships_source_entity_id_not_empty CHECK (length(trim(source_entity_id)) > 0),
    CONSTRAINT relationships_target_entity_id_not_empty CHECK (length(trim(target_entity_id)) > 0),
    CONSTRAINT relationships_type_not_empty CHECK (length(trim(relationship_type)) > 0),
    CONSTRAINT relationships_distinct_endpoints CHECK (
        source_entity_kind != target_entity_kind
        OR source_entity_id != target_entity_id
    ),
    CONSTRAINT relationships_trust_score_range CHECK (trust_score >= 0.0 AND trust_score <= 1.0),
    CONSTRAINT relationships_strength_score_range CHECK (strength_score >= 0.0 AND strength_score <= 1.0),
    CONSTRAINT relationships_confidence_range CHECK (confidence >= 0.0 AND confidence <= 1.0),
    CONSTRAINT relationships_review_state_check CHECK (
        review_state IN ('suggested', 'system_accepted', 'user_confirmed', 'user_rejected')
    ),
    CONSTRAINT relationships_temporal_range_check CHECK (
        valid_to IS NULL OR valid_from IS NULL OR valid_to >= valid_from
    ),
    CONSTRAINT relationships_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE UNIQUE INDEX IF NOT EXISTS relationships_active_unique
    ON relationships (
        source_entity_kind,
        source_entity_id,
        target_entity_kind,
        target_entity_id,
        relationship_type
    )
    WHERE valid_to IS NULL;

CREATE INDEX IF NOT EXISTS relationships_source_idx
    ON relationships (source_entity_kind, source_entity_id);
CREATE INDEX IF NOT EXISTS relationships_target_idx
    ON relationships (target_entity_kind, target_entity_id);
CREATE INDEX IF NOT EXISTS relationships_type_idx
    ON relationships (relationship_type);
CREATE INDEX IF NOT EXISTS relationships_review_state_idx
    ON relationships (review_state, updated_at);

CREATE TABLE IF NOT EXISTS relationship_evidence (
    evidence_id TEXT PRIMARY KEY,
    relationship_id TEXT NOT NULL REFERENCES relationships(relationship_id) ON DELETE CASCADE,
    source_kind TEXT NOT NULL,
    source_id TEXT NOT NULL,
    excerpt TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT relationship_evidence_source_kind_check CHECK (
        source_kind IN (
            'communication',
            'document',
            'event',
            'memory',
            'knowledge',
            'decision',
            'obligation',
            'task',
            'project',
            'organization',
            'persona',
            'raw_record'
        )
    ),
    CONSTRAINT relationship_evidence_source_id_not_empty CHECK (length(trim(source_id)) > 0),
    CONSTRAINT relationship_evidence_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object'),
    UNIQUE (relationship_id, source_kind, source_id)
);

CREATE INDEX IF NOT EXISTS relationship_evidence_relationship_idx
    ON relationship_evidence (relationship_id);
CREATE INDEX IF NOT EXISTS relationship_evidence_source_idx
    ON relationship_evidence (source_kind, source_id);
