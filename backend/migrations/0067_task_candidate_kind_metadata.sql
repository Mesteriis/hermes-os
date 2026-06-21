ALTER TABLE task_candidates
    ADD COLUMN IF NOT EXISTS candidate_kind TEXT NOT NULL DEFAULT 'task';

ALTER TABLE task_candidates
    ADD COLUMN IF NOT EXISTS candidate_metadata JSONB NOT NULL DEFAULT '{}'::jsonb;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'task_candidates_candidate_kind_check'
    ) THEN
        ALTER TABLE task_candidates
            ADD CONSTRAINT task_candidates_candidate_kind_check
            CHECK (candidate_kind IN ('task', 'obligation_task'));
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'task_candidates_candidate_metadata_is_object'
    ) THEN
        ALTER TABLE task_candidates
            ADD CONSTRAINT task_candidates_candidate_metadata_is_object
            CHECK (jsonb_typeof(candidate_metadata) = 'object');
    END IF;
END $$;

CREATE INDEX IF NOT EXISTS task_candidates_candidate_kind_idx
    ON task_candidates (candidate_kind, review_state, updated_at DESC);
