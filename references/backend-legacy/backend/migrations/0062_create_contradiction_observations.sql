CREATE TABLE IF NOT EXISTS contradiction_observations (
    observation_id TEXT PRIMARY KEY,
    old_source_kind TEXT NOT NULL,
    old_source_id TEXT NOT NULL,
    new_source_kind TEXT NOT NULL,
    new_source_id TEXT NOT NULL,
    affected_entities JSONB NOT NULL DEFAULT '[]'::jsonb,
    conflict_type TEXT NOT NULL,
    old_claim TEXT NOT NULL,
    new_claim TEXT NOT NULL,
    confidence NUMERIC(5,4) NOT NULL,
    severity TEXT NOT NULL,
    review_state TEXT NOT NULL DEFAULT 'suggested',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    reviewed_by TEXT,
    reviewed_at TIMESTAMPTZ,
    resolution TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT contradiction_observations_source_kind_check CHECK (
        old_source_kind IN (
            'communication',
            'document',
            'event',
            'memory',
            'knowledge',
            'decision',
            'obligation',
            'task',
            'relationship',
            'raw_record'
        )
        AND new_source_kind IN (
            'communication',
            'document',
            'event',
            'memory',
            'knowledge',
            'decision',
            'obligation',
            'task',
            'relationship',
            'raw_record'
        )
    ),
    CONSTRAINT contradiction_observations_old_source_id_not_empty CHECK (length(trim(old_source_id)) > 0),
    CONSTRAINT contradiction_observations_new_source_id_not_empty CHECK (length(trim(new_source_id)) > 0),
    CONSTRAINT contradiction_observations_conflict_type_not_empty CHECK (length(trim(conflict_type)) > 0),
    CONSTRAINT contradiction_observations_old_claim_not_empty CHECK (length(trim(old_claim)) > 0),
    CONSTRAINT contradiction_observations_new_claim_not_empty CHECK (length(trim(new_claim)) > 0),
    CONSTRAINT contradiction_observations_confidence_range CHECK (confidence >= 0.0 AND confidence <= 1.0),
    CONSTRAINT contradiction_observations_severity_check CHECK (
        severity IN ('low', 'medium', 'high', 'critical')
    ),
    CONSTRAINT contradiction_observations_review_state_check CHECK (
        review_state IN ('suggested', 'user_confirmed', 'user_rejected')
    ),
    CONSTRAINT contradiction_observations_affected_entities_json_check CHECK (
        jsonb_typeof(affected_entities) IN ('array', 'object')
    ),
    CONSTRAINT contradiction_observations_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object'),
    CONSTRAINT contradiction_observations_reviewed_by_not_empty CHECK (
        reviewed_by IS NULL OR length(trim(reviewed_by)) > 0
    )
);

CREATE UNIQUE INDEX IF NOT EXISTS contradiction_observations_source_unique
    ON contradiction_observations (
        old_source_kind,
        old_source_id,
        new_source_kind,
        new_source_id,
        conflict_type
    );

CREATE INDEX IF NOT EXISTS contradiction_observations_review_state_idx
    ON contradiction_observations (review_state, updated_at DESC);

CREATE INDEX IF NOT EXISTS contradiction_observations_new_source_idx
    ON contradiction_observations (new_source_kind, new_source_id);

CREATE INDEX IF NOT EXISTS contradiction_observations_old_source_idx
    ON contradiction_observations (old_source_kind, old_source_id);
