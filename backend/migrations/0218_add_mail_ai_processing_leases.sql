ALTER TABLE communication_ai_states
    ADD COLUMN IF NOT EXISTS retry_count INTEGER NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS next_attempt_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS processing_lease_expires_at TIMESTAMPTZ;

ALTER TABLE communication_ai_states
    DROP CONSTRAINT IF EXISTS communication_ai_states_retry_count_nonnegative;

ALTER TABLE communication_ai_states
    ADD CONSTRAINT communication_ai_states_retry_count_nonnegative
    CHECK (retry_count >= 0);

CREATE INDEX IF NOT EXISTS communication_ai_states_retry_due_idx
    ON communication_ai_states (next_attempt_at, message_id)
    WHERE ai_state = 'FAILED' AND next_attempt_at IS NOT NULL;

CREATE INDEX IF NOT EXISTS communication_ai_states_processing_lease_idx
    ON communication_ai_states (processing_lease_expires_at, message_id)
    WHERE ai_state = 'PROCESSING' AND processing_lease_expires_at IS NOT NULL;

-- The percentage/decimal confidence defect is fixed by this release. Requeue
-- only those known deterministic failures; other failures remain owner-visible.
UPDATE communication_ai_states
SET next_attempt_at = now()
WHERE ai_state = 'FAILED'
  AND retry_count = 0
  AND last_error LIKE 'confidence must be between 0.0 and 1.0:%';

-- States written before leases existed are recoverable on the first worker tick.
UPDATE communication_ai_states
SET processing_lease_expires_at = now() - INTERVAL '1 second'
WHERE ai_state = 'PROCESSING'
  AND processing_lease_expires_at IS NULL;
