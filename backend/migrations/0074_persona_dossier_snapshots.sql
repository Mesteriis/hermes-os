CREATE TABLE IF NOT EXISTS persona_dossier_snapshots (
    dossier_snapshot_id TEXT PRIMARY KEY,
    persona_id TEXT NOT NULL,
    dossier JSONB NOT NULL,
    source_refs JSONB NOT NULL DEFAULT '[]'::jsonb,
    review_state TEXT NOT NULL DEFAULT 'suggested',
    reviewed_by TEXT,
    reviewed_at TIMESTAMPTZ,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    generated_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT persona_dossier_snapshots_persona_id_not_empty CHECK (length(trim(persona_id)) > 0),
    CONSTRAINT persona_dossier_snapshots_dossier_is_object CHECK (jsonb_typeof(dossier) = 'object'),
    CONSTRAINT persona_dossier_snapshots_source_refs_is_array CHECK (jsonb_typeof(source_refs) = 'array'),
    CONSTRAINT persona_dossier_snapshots_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object'),
    CONSTRAINT persona_dossier_snapshots_review_state_check CHECK (
        review_state IN ('suggested', 'user_confirmed', 'user_rejected')
    ),
    CONSTRAINT persona_dossier_snapshots_reviewed_by_not_empty CHECK (
        reviewed_by IS NULL OR length(trim(reviewed_by)) > 0
    )
);

CREATE UNIQUE INDEX IF NOT EXISTS persona_dossier_snapshots_persona_latest_unique
    ON persona_dossier_snapshots (persona_id);

CREATE INDEX IF NOT EXISTS persona_dossier_snapshots_review_state_idx
    ON persona_dossier_snapshots (review_state, updated_at DESC);
