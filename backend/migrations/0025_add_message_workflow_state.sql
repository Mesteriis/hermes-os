ALTER TABLE communication_messages
    ADD COLUMN IF NOT EXISTS workflow_state TEXT NOT NULL DEFAULT 'new',
    ADD COLUMN IF NOT EXISTS importance_score SMALLINT,
    ADD COLUMN IF NOT EXISTS ai_category TEXT,
    ADD COLUMN IF NOT EXISTS ai_summary TEXT,
    ADD COLUMN IF NOT EXISTS ai_summary_generated_at TIMESTAMPTZ;

ALTER TABLE communication_messages
    ADD CONSTRAINT communication_messages_workflow_state CHECK (
        workflow_state IN (
            'new',
            'reviewed',
            'needs_action',
            'waiting',
            'done',
            'archived',
            'muted',
            'spam'
        )
    );

ALTER TABLE communication_messages
    ADD CONSTRAINT communication_messages_importance_score_range CHECK (
        importance_score IS NULL OR (importance_score >= 0 AND importance_score <= 100)
    );

CREATE INDEX IF NOT EXISTS communication_messages_workflow_state_idx
    ON communication_messages (workflow_state, COALESCE(occurred_at, projected_at) DESC);

CREATE INDEX IF NOT EXISTS communication_messages_importance_idx
    ON communication_messages (importance_score DESC NULLS LAST)
    WHERE importance_score IS NOT NULL;
