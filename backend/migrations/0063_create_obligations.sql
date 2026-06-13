CREATE TABLE IF NOT EXISTS obligations (
    obligation_id TEXT PRIMARY KEY,
    obligated_entity_kind TEXT NOT NULL,
    obligated_entity_id TEXT NOT NULL,
    beneficiary_entity_kind TEXT,
    beneficiary_entity_id TEXT,
    statement TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'open',
    review_state TEXT NOT NULL DEFAULT 'suggested',
    due_at TIMESTAMPTZ,
    condition TEXT,
    risk_state TEXT NOT NULL DEFAULT 'none',
    confidence NUMERIC(5,4) NOT NULL DEFAULT 1.0000,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT obligations_entity_kind_check CHECK (
        obligated_entity_kind IN (
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
        AND (
            beneficiary_entity_kind IS NULL
            OR beneficiary_entity_kind IN (
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
        )
    ),
    CONSTRAINT obligations_obligated_entity_id_not_empty CHECK (length(trim(obligated_entity_id)) > 0),
    CONSTRAINT obligations_beneficiary_pair_check CHECK (
        (beneficiary_entity_kind IS NULL AND beneficiary_entity_id IS NULL)
        OR (
            beneficiary_entity_kind IS NOT NULL
            AND beneficiary_entity_id IS NOT NULL
            AND length(trim(beneficiary_entity_id)) > 0
        )
    ),
    CONSTRAINT obligations_statement_not_empty CHECK (length(trim(statement)) > 0),
    CONSTRAINT obligations_status_check CHECK (
        status IN ('open', 'fulfilled', 'waived', 'disputed', 'canceled')
    ),
    CONSTRAINT obligations_review_state_check CHECK (
        review_state IN ('suggested', 'user_confirmed', 'user_rejected')
    ),
    CONSTRAINT obligations_risk_state_check CHECK (
        risk_state IN ('none', 'watch', 'at_risk', 'breached')
    ),
    CONSTRAINT obligations_confidence_range CHECK (confidence >= 0.0 AND confidence <= 1.0),
    CONSTRAINT obligations_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE UNIQUE INDEX IF NOT EXISTS obligations_active_unique
    ON obligations (
        obligated_entity_kind,
        obligated_entity_id,
        COALESCE(beneficiary_entity_kind, ''),
        COALESCE(beneficiary_entity_id, ''),
        lower(statement)
    )
    WHERE status IN ('open', 'disputed');

CREATE INDEX IF NOT EXISTS obligations_obligated_entity_idx
    ON obligations (obligated_entity_kind, obligated_entity_id, updated_at DESC);
CREATE INDEX IF NOT EXISTS obligations_beneficiary_entity_idx
    ON obligations (beneficiary_entity_kind, beneficiary_entity_id, updated_at DESC)
    WHERE beneficiary_entity_kind IS NOT NULL;
CREATE INDEX IF NOT EXISTS obligations_status_idx
    ON obligations (status, updated_at DESC);
CREATE INDEX IF NOT EXISTS obligations_review_state_idx
    ON obligations (review_state, updated_at DESC);
CREATE INDEX IF NOT EXISTS obligations_risk_state_idx
    ON obligations (risk_state, updated_at DESC);

CREATE TABLE IF NOT EXISTS obligation_evidence (
    evidence_id TEXT PRIMARY KEY,
    obligation_id TEXT NOT NULL REFERENCES obligations(obligation_id) ON DELETE CASCADE,
    source_kind TEXT NOT NULL,
    source_id TEXT NOT NULL,
    quote TEXT,
    confidence NUMERIC(5,4) NOT NULL DEFAULT 1.0000,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT obligation_evidence_source_kind_check CHECK (
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
    CONSTRAINT obligation_evidence_source_id_not_empty CHECK (length(trim(source_id)) > 0),
    CONSTRAINT obligation_evidence_confidence_range CHECK (confidence >= 0.0 AND confidence <= 1.0),
    CONSTRAINT obligation_evidence_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object'),
    UNIQUE (obligation_id, source_kind, source_id)
);

CREATE INDEX IF NOT EXISTS obligation_evidence_obligation_idx
    ON obligation_evidence (obligation_id);
CREATE INDEX IF NOT EXISTS obligation_evidence_source_idx
    ON obligation_evidence (source_kind, source_id);

CREATE TABLE IF NOT EXISTS obligation_task_links (
    obligation_id TEXT NOT NULL REFERENCES obligations(obligation_id) ON DELETE CASCADE,
    task_id TEXT NOT NULL REFERENCES tasks(task_id) ON DELETE CASCADE,
    link_kind TEXT NOT NULL DEFAULT 'related',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT obligation_task_links_kind_check CHECK (
        link_kind IN ('related', 'fulfillment_task', 'follow_up_task', 'evidence_task')
    ),
    CONSTRAINT obligation_task_links_task_id_not_empty CHECK (length(trim(task_id)) > 0),
    PRIMARY KEY (obligation_id, task_id, link_kind)
);

CREATE INDEX IF NOT EXISTS obligation_task_links_task_idx
    ON obligation_task_links (task_id);
