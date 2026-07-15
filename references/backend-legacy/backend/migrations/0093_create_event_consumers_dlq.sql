CREATE TABLE IF NOT EXISTS event_consumers (
    consumer_name TEXT PRIMARY KEY,
    last_processed_position BIGINT NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'active',
    locked_by TEXT,
    locked_until TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT event_consumers_name_not_empty CHECK (length(trim(consumer_name)) > 0),
    CONSTRAINT event_consumers_position_non_negative CHECK (last_processed_position >= 0),
    CONSTRAINT event_consumers_status CHECK (status IN ('active', 'paused', 'disabled')),
    CONSTRAINT event_consumers_locked_by_not_empty CHECK (
        locked_by IS NULL OR length(trim(locked_by)) > 0
    )
);

CREATE INDEX IF NOT EXISTS event_consumers_updated_at_idx
    ON event_consumers (updated_at);

CREATE TABLE IF NOT EXISTS event_consumer_failures (
    consumer_name TEXT NOT NULL REFERENCES event_consumers(consumer_name) ON DELETE CASCADE,
    event_position BIGINT NOT NULL,
    event_id TEXT NOT NULL,
    event_type TEXT NOT NULL,
    attempt_count INTEGER NOT NULL DEFAULT 1,
    next_attempt_at TIMESTAMPTZ NOT NULL,
    last_attempt_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_error TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    PRIMARY KEY (consumer_name, event_position),

    CONSTRAINT event_consumer_failures_position_positive CHECK (event_position > 0),
    CONSTRAINT event_consumer_failures_event_id_not_empty CHECK (length(trim(event_id)) > 0),
    CONSTRAINT event_consumer_failures_event_type_not_empty CHECK (length(trim(event_type)) > 0),
    CONSTRAINT event_consumer_failures_attempt_count_positive CHECK (attempt_count > 0),
    CONSTRAINT event_consumer_failures_last_error_not_empty CHECK (length(trim(last_error)) > 0)
);

CREATE INDEX IF NOT EXISTS event_consumer_failures_due_idx
    ON event_consumer_failures (consumer_name, next_attempt_at, event_position);

CREATE TABLE IF NOT EXISTS event_consumer_processed_events (
    consumer_name TEXT NOT NULL REFERENCES event_consumers(consumer_name) ON DELETE CASCADE,
    event_position BIGINT NOT NULL,
    event_id TEXT NOT NULL,
    event_type TEXT NOT NULL,
    processed_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    PRIMARY KEY (consumer_name, event_position),

    CONSTRAINT event_consumer_processed_events_position_positive CHECK (event_position > 0),
    CONSTRAINT event_consumer_processed_events_event_id_not_empty CHECK (length(trim(event_id)) > 0),
    CONSTRAINT event_consumer_processed_events_event_type_not_empty CHECK (length(trim(event_type)) > 0)
);

CREATE UNIQUE INDEX IF NOT EXISTS event_consumer_processed_events_event_id_idx
    ON event_consumer_processed_events (consumer_name, event_id);

CREATE TABLE IF NOT EXISTS event_dead_letters (
    dead_letter_id TEXT PRIMARY KEY,
    consumer_name TEXT NOT NULL REFERENCES event_consumers(consumer_name) ON DELETE CASCADE,
    event_position BIGINT NOT NULL,
    event_id TEXT NOT NULL,
    event_type TEXT NOT NULL,
    attempts INTEGER NOT NULL,
    last_error TEXT NOT NULL,
    event_payload JSONB NOT NULL,
    review_state TEXT NOT NULL DEFAULT 'open',
    replay_requested_at TIMESTAMPTZ,
    replayed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT event_dead_letters_consumer_event_unique UNIQUE (consumer_name, event_position),
    CONSTRAINT event_dead_letters_id_not_empty CHECK (length(trim(dead_letter_id)) > 0),
    CONSTRAINT event_dead_letters_position_positive CHECK (event_position > 0),
    CONSTRAINT event_dead_letters_event_id_not_empty CHECK (length(trim(event_id)) > 0),
    CONSTRAINT event_dead_letters_event_type_not_empty CHECK (length(trim(event_type)) > 0),
    CONSTRAINT event_dead_letters_attempts_positive CHECK (attempts > 0),
    CONSTRAINT event_dead_letters_last_error_not_empty CHECK (length(trim(last_error)) > 0),
    CONSTRAINT event_dead_letters_payload_is_object CHECK (jsonb_typeof(event_payload) = 'object'),
    CONSTRAINT event_dead_letters_review_state CHECK (
        review_state IN ('open', 'replay_requested', 'replayed', 'dismissed')
    )
);

CREATE INDEX IF NOT EXISTS event_dead_letters_review_idx
    ON event_dead_letters (review_state, created_at DESC);

CREATE INDEX IF NOT EXISTS event_dead_letters_consumer_idx
    ON event_dead_letters (consumer_name, event_position);
