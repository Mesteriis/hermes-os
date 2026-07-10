-- Rename the derived persona-evidence consumer/projection runtime names while
-- preserving durable cursor, retry, DLQ and runtime gate state.

INSERT INTO event_consumers (
    consumer_name,
    last_processed_position,
    status,
    locked_by,
    locked_until,
    created_at,
    updated_at
)
SELECT
    'persona_derived_evidence',
    last_processed_position,
    status,
    locked_by,
    locked_until,
    created_at,
    now()
FROM event_consumers
WHERE consumer_name = 'person_derived_evidence'
ON CONFLICT (consumer_name) DO UPDATE
SET
    last_processed_position = GREATEST(
        event_consumers.last_processed_position,
        EXCLUDED.last_processed_position
    ),
    status = EXCLUDED.status,
    locked_by = EXCLUDED.locked_by,
    locked_until = EXCLUDED.locked_until,
    updated_at = now();

INSERT INTO event_consumer_failures (
    consumer_name,
    event_position,
    event_id,
    event_type,
    attempt_count,
    next_attempt_at,
    last_attempt_at,
    last_error,
    created_at,
    updated_at
)
SELECT
    'persona_derived_evidence',
    event_position,
    event_id,
    event_type,
    attempt_count,
    next_attempt_at,
    last_attempt_at,
    last_error,
    created_at,
    now()
FROM event_consumer_failures
WHERE consumer_name = 'person_derived_evidence'
ON CONFLICT (consumer_name, event_position) DO UPDATE
SET
    attempt_count = GREATEST(
        event_consumer_failures.attempt_count,
        EXCLUDED.attempt_count
    ),
    next_attempt_at = LEAST(
        event_consumer_failures.next_attempt_at,
        EXCLUDED.next_attempt_at
    ),
    last_attempt_at = GREATEST(
        event_consumer_failures.last_attempt_at,
        EXCLUDED.last_attempt_at
    ),
    last_error = EXCLUDED.last_error,
    updated_at = now();

INSERT INTO event_consumer_processed_events (
    consumer_name,
    event_position,
    event_id,
    event_type,
    processed_at
)
SELECT
    'persona_derived_evidence',
    event_position,
    event_id,
    event_type,
    processed_at
FROM event_consumer_processed_events
WHERE consumer_name = 'person_derived_evidence'
ON CONFLICT DO NOTHING;

INSERT INTO event_dead_letters (
    dead_letter_id,
    consumer_name,
    event_position,
    event_id,
    event_type,
    attempts,
    last_error,
    event_payload,
    review_state,
    replay_requested_at,
    replayed_at,
    created_at,
    updated_at
)
SELECT
    dead_letter_id,
    'persona_derived_evidence',
    event_position,
    event_id,
    event_type,
    attempts,
    last_error,
    event_payload,
    review_state,
    replay_requested_at,
    replayed_at,
    created_at,
    now()
FROM event_dead_letters
WHERE consumer_name = 'person_derived_evidence'
ON CONFLICT DO NOTHING;

DELETE FROM event_consumer_failures
WHERE consumer_name = 'person_derived_evidence';

DELETE FROM event_consumer_processed_events
WHERE consumer_name = 'person_derived_evidence';

DELETE FROM event_dead_letters
WHERE consumer_name = 'person_derived_evidence';

DELETE FROM event_consumers
WHERE consumer_name = 'person_derived_evidence';

INSERT INTO projection_cursors (
    projection_name,
    last_processed_position,
    updated_at
)
SELECT
    'persona_derived_evidence',
    last_processed_position,
    now()
FROM projection_cursors
WHERE projection_name = 'person_derived_evidence'
ON CONFLICT (projection_name) DO UPDATE
SET
    last_processed_position = GREATEST(
        projection_cursors.last_processed_position,
        EXCLUDED.last_processed_position
    ),
    updated_at = now();

DELETE FROM projection_cursors
WHERE projection_name = 'person_derived_evidence';

UPDATE signal_runtime_states
SET runtime_kind = 'persona_derived_evidence',
    updated_at = now()
WHERE runtime_kind = 'person_derived_evidence';

UPDATE signal_replay_requests
SET metadata = jsonb_set(
        metadata,
        '{target_projection}',
        to_jsonb('persona_derived_evidence'::text),
        false
    )
WHERE metadata ->> 'target_projection' = 'person_derived_evidence';
