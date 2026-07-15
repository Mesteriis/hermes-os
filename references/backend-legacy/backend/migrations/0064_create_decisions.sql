CREATE TABLE IF NOT EXISTS decisions (
    decision_id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    rationale TEXT NOT NULL,
    alternatives JSONB NOT NULL DEFAULT '[]'::jsonb,
    decided_by_entity_kind TEXT,
    decided_by_entity_id TEXT,
    decided_at TIMESTAMPTZ,
    review_state TEXT NOT NULL DEFAULT 'suggested',
    confidence NUMERIC(5,4) NOT NULL DEFAULT 1.0000,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT decisions_title_not_empty CHECK (length(trim(title)) > 0),
    CONSTRAINT decisions_rationale_not_empty CHECK (length(trim(rationale)) > 0),
    CONSTRAINT decisions_status_check CHECK (
        status IN ('active', 'superseded', 'reversed', 'deprecated')
    ),
    CONSTRAINT decisions_decider_pair_check CHECK (
        (decided_by_entity_kind IS NULL AND decided_by_entity_id IS NULL)
        OR (
            decided_by_entity_kind IS NOT NULL
            AND decided_by_entity_id IS NOT NULL
            AND decided_by_entity_kind IN (
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
            AND length(trim(decided_by_entity_id)) > 0
        )
    ),
    CONSTRAINT decisions_review_state_check CHECK (
        review_state IN ('suggested', 'user_confirmed', 'user_rejected')
    ),
    CONSTRAINT decisions_confidence_range CHECK (confidence >= 0.0 AND confidence <= 1.0),
    CONSTRAINT decisions_alternatives_is_array CHECK (jsonb_typeof(alternatives) = 'array'),
    CONSTRAINT decisions_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE INDEX IF NOT EXISTS decisions_status_idx
    ON decisions (status, updated_at DESC);
CREATE INDEX IF NOT EXISTS decisions_review_state_idx
    ON decisions (review_state, updated_at DESC);
CREATE INDEX IF NOT EXISTS decisions_decider_idx
    ON decisions (decided_by_entity_kind, decided_by_entity_id, decided_at DESC)
    WHERE decided_by_entity_kind IS NOT NULL;
CREATE INDEX IF NOT EXISTS decisions_decided_at_idx
    ON decisions (decided_at DESC)
    WHERE decided_at IS NOT NULL;

CREATE TABLE IF NOT EXISTS decision_evidence (
    evidence_id TEXT PRIMARY KEY,
    decision_id TEXT NOT NULL REFERENCES decisions(decision_id) ON DELETE CASCADE,
    source_kind TEXT NOT NULL,
    source_id TEXT NOT NULL,
    quote TEXT,
    confidence NUMERIC(5,4) NOT NULL DEFAULT 1.0000,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT decision_evidence_source_kind_check CHECK (
        source_kind IN (
            'communication',
            'document',
            'event',
            'memory',
            'knowledge',
            'decision',
            'obligation',
            'task',
            'relationship',
            'project',
            'organization',
            'persona',
            'raw_record'
        )
    ),
    CONSTRAINT decision_evidence_source_id_not_empty CHECK (length(trim(source_id)) > 0),
    CONSTRAINT decision_evidence_confidence_range CHECK (confidence >= 0.0 AND confidence <= 1.0),
    CONSTRAINT decision_evidence_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object'),
    UNIQUE (decision_id, source_kind, source_id)
);

CREATE INDEX IF NOT EXISTS decision_evidence_decision_idx
    ON decision_evidence (decision_id);
CREATE INDEX IF NOT EXISTS decision_evidence_source_idx
    ON decision_evidence (source_kind, source_id);

CREATE TABLE IF NOT EXISTS decision_impacted_entities (
    decision_id TEXT NOT NULL REFERENCES decisions(decision_id) ON DELETE CASCADE,
    entity_kind TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    impact_type TEXT NOT NULL DEFAULT 'related',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT decision_impacted_entities_entity_kind_check CHECK (
        entity_kind IN (
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
    CONSTRAINT decision_impacted_entities_entity_id_not_empty CHECK (length(trim(entity_id)) > 0),
    CONSTRAINT decision_impacted_entities_impact_type_not_empty CHECK (length(trim(impact_type)) > 0),
    CONSTRAINT decision_impacted_entities_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object'),
    PRIMARY KEY (decision_id, entity_kind, entity_id)
);

CREATE INDEX IF NOT EXISTS decision_impacted_entities_entity_idx
    ON decision_impacted_entities (entity_kind, entity_id);
