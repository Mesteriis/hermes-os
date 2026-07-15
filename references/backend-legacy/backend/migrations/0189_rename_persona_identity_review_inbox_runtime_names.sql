-- Rename the persona identity review inbox consumer/runtime while preserving
-- durable consumer cursor, retry markers, DLQ rows and runtime gate state.

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
    'persona_identity_review_inbox',
    last_processed_position,
    status,
    locked_by,
    locked_until,
    created_at,
    now()
FROM event_consumers
WHERE consumer_name = 'person_identity_review_inbox'
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
    'persona_identity_review_inbox',
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
WHERE consumer_name = 'person_identity_review_inbox'
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
    'persona_identity_review_inbox',
    event_position,
    event_id,
    event_type,
    processed_at
FROM event_consumer_processed_events
WHERE consumer_name = 'person_identity_review_inbox'
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
    'persona_identity_review_inbox',
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
WHERE consumer_name = 'person_identity_review_inbox'
ON CONFLICT DO NOTHING;

DELETE FROM event_consumer_failures
WHERE consumer_name = 'person_identity_review_inbox';

DELETE FROM event_consumer_processed_events
WHERE consumer_name = 'person_identity_review_inbox';

DELETE FROM event_dead_letters
WHERE consumer_name = 'person_identity_review_inbox';

DELETE FROM event_consumers
WHERE consumer_name = 'person_identity_review_inbox';

UPDATE signal_runtime_states
SET runtime_kind = 'persona_identity_review_inbox',
    updated_at = now()
WHERE runtime_kind = 'person_identity_review_inbox';
