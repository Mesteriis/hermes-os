CREATE TABLE IF NOT EXISTS communication_ai_states (
    message_id TEXT PRIMARY KEY REFERENCES communication_messages(message_id) ON DELETE CASCADE,
    ai_state TEXT NOT NULL DEFAULT 'NEW',
    review_reason TEXT,
    last_error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT communication_ai_states_state CHECK (
        ai_state IN ('NEW', 'PROCESSING', 'PROCESSED', 'REVIEW_REQUIRED', 'FAILED', 'ARCHIVED')
    ),
    CONSTRAINT communication_ai_states_review_reason_not_blank CHECK (
        review_reason IS NULL OR length(trim(review_reason)) > 0
    ),
    CONSTRAINT communication_ai_states_last_error_not_blank CHECK (
        last_error IS NULL OR length(trim(last_error)) > 0
    )
);

CREATE INDEX IF NOT EXISTS communication_ai_states_state_updated_idx
    ON communication_ai_states (ai_state, updated_at DESC, message_id);

INSERT INTO communication_ai_states (message_id, ai_state, review_reason, last_error, created_at, updated_at)
SELECT
    message_id,
    ai_state,
    review_reason,
    last_error,
    created_at,
    updated_at
FROM mail_ai_states
ON CONFLICT (message_id) DO NOTHING;
