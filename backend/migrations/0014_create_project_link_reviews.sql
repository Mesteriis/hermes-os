CREATE TABLE IF NOT EXISTS project_link_reviews (
    project_id TEXT NOT NULL REFERENCES projects(project_id) ON DELETE CASCADE,
    target_kind TEXT NOT NULL,
    target_id TEXT NOT NULL,
    review_state TEXT NOT NULL,
    event_id TEXT NOT NULL REFERENCES event_log(event_id),
    actor_id TEXT NOT NULL,
    reviewed_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT project_link_reviews_pk PRIMARY KEY (project_id, target_kind, target_id),
    CONSTRAINT project_link_reviews_target_kind_check
        CHECK (target_kind IN ('message', 'document')),
    CONSTRAINT project_link_reviews_review_state_check
        CHECK (review_state IN ('user_confirmed', 'user_rejected')),
    CONSTRAINT project_link_reviews_actor_id_not_empty
        CHECK (length(trim(actor_id)) > 0),
    CONSTRAINT project_link_reviews_project_id_not_empty
        CHECK (length(trim(project_id)) > 0),
    CONSTRAINT project_link_reviews_target_id_not_empty
        CHECK (length(trim(target_id)) > 0)
);

CREATE INDEX IF NOT EXISTS project_link_reviews_event_id_idx
    ON project_link_reviews (event_id);

CREATE INDEX IF NOT EXISTS project_link_reviews_review_state_idx
    ON project_link_reviews (review_state, updated_at);
