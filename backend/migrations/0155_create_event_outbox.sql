CREATE TABLE IF NOT EXISTS event_outbox (
    event_id TEXT PRIMARY KEY REFERENCES event_log(event_id) ON DELETE RESTRICT,
    subject TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    attempts INTEGER NOT NULL DEFAULT 0,
    next_attempt_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_error_redacted TEXT,
    published_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT event_outbox_subject_not_empty CHECK (length(trim(subject)) > 0),
    CONSTRAINT event_outbox_status_not_empty CHECK (length(trim(status)) > 0),
    CONSTRAINT event_outbox_attempts_non_negative CHECK (attempts >= 0)
);

CREATE INDEX IF NOT EXISTS event_outbox_pending_idx
    ON event_outbox (next_attempt_at, created_at)
    WHERE status = 'pending';

CREATE INDEX IF NOT EXISTS event_log_source_code_idx
    ON event_log ((source ->> 'source_code'), occurred_at, position)
    WHERE source ? 'source_code';

CREATE INDEX IF NOT EXISTS event_log_subject_identity_idx
    ON event_log ((subject ->> 'kind'), (subject ->> 'entity_id'), occurred_at, position)
    WHERE subject ? 'kind';

CREATE INDEX IF NOT EXISTS event_log_source_gin_idx
    ON event_log USING GIN (source);

CREATE INDEX IF NOT EXISTS event_log_subject_gin_idx
    ON event_log USING GIN (subject);
