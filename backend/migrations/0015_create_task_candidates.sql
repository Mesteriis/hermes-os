CREATE TABLE IF NOT EXISTS task_candidates (
    task_candidate_id TEXT PRIMARY KEY,
    source_kind TEXT NOT NULL,
    source_id TEXT NOT NULL,
    project_id TEXT REFERENCES projects(project_id) ON DELETE SET NULL,
    title TEXT NOT NULL,
    due_text TEXT,
    assignee_label TEXT,
    confidence DOUBLE PRECISION NOT NULL,
    review_state TEXT NOT NULL DEFAULT 'suggested',
    evidence_excerpt TEXT NOT NULL,
    event_id TEXT,
    actor_id TEXT,
    generated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    reviewed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT task_candidates_source_kind_check
        CHECK (source_kind IN ('message', 'document')),
    CONSTRAINT task_candidates_review_state_check
        CHECK (review_state IN ('suggested', 'user_confirmed', 'user_rejected')),
    CONSTRAINT task_candidates_confidence_check
        CHECK (confidence >= 0.0 AND confidence <= 1.0),
    CONSTRAINT task_candidates_id_not_empty
        CHECK (length(trim(task_candidate_id)) > 0),
    CONSTRAINT task_candidates_source_id_not_empty
        CHECK (length(trim(source_id)) > 0),
    CONSTRAINT task_candidates_title_not_empty
        CHECK (length(trim(title)) > 0),
    CONSTRAINT task_candidates_evidence_excerpt_not_empty
        CHECK (length(trim(evidence_excerpt)) > 0)
);

CREATE UNIQUE INDEX IF NOT EXISTS task_candidates_source_title_idx
    ON task_candidates (source_kind, source_id, lower(title));

CREATE INDEX IF NOT EXISTS task_candidates_review_state_idx
    ON task_candidates (review_state, updated_at DESC);

CREATE INDEX IF NOT EXISTS task_candidates_project_idx
    ON task_candidates (project_id);

CREATE TABLE IF NOT EXISTS tasks (
    task_id TEXT PRIMARY KEY,
    task_candidate_id TEXT NOT NULL UNIQUE
        REFERENCES task_candidates(task_candidate_id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    source_kind TEXT NOT NULL,
    source_id TEXT NOT NULL,
    project_id TEXT REFERENCES projects(project_id) ON DELETE SET NULL,
    status TEXT NOT NULL DEFAULT 'active',
    created_from_event_id TEXT NOT NULL,
    created_by_actor_id TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT tasks_source_kind_check
        CHECK (source_kind IN ('message', 'document')),
    CONSTRAINT tasks_status_check
        CHECK (status IN ('active')),
    CONSTRAINT tasks_id_not_empty CHECK (length(trim(task_id)) > 0),
    CONSTRAINT tasks_title_not_empty CHECK (length(trim(title)) > 0)
);

CREATE INDEX IF NOT EXISTS tasks_project_idx ON tasks (project_id);
CREATE INDEX IF NOT EXISTS tasks_source_idx ON tasks (source_kind, source_id);
