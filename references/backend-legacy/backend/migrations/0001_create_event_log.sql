CREATE TABLE IF NOT EXISTS event_log (
    position BIGINT GENERATED ALWAYS AS IDENTITY UNIQUE,
    event_id TEXT PRIMARY KEY,
    event_type TEXT NOT NULL,
    schema_version INTEGER NOT NULL,
    occurred_at TIMESTAMPTZ NOT NULL,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    source JSONB NOT NULL,
    actor JSONB,
    subject JSONB NOT NULL,
    payload JSONB NOT NULL DEFAULT '{}'::jsonb,
    provenance JSONB NOT NULL DEFAULT '{}'::jsonb,
    causation_id TEXT,
    correlation_id TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT event_log_event_id_not_empty CHECK (length(trim(event_id)) > 0),
    CONSTRAINT event_log_event_type_not_empty CHECK (length(trim(event_type)) > 0),
    CONSTRAINT event_log_schema_version_positive CHECK (schema_version > 0),
    CONSTRAINT event_log_source_is_object CHECK (jsonb_typeof(source) = 'object'),
    CONSTRAINT event_log_actor_is_object CHECK (actor IS NULL OR jsonb_typeof(actor) = 'object'),
    CONSTRAINT event_log_subject_is_object CHECK (jsonb_typeof(subject) = 'object'),
    CONSTRAINT event_log_payload_is_object CHECK (jsonb_typeof(payload) = 'object'),
    CONSTRAINT event_log_provenance_is_object CHECK (jsonb_typeof(provenance) = 'object')
);

CREATE INDEX IF NOT EXISTS event_log_recorded_at_idx
    ON event_log (recorded_at, position);

CREATE INDEX IF NOT EXISTS event_log_occurred_at_idx
    ON event_log (occurred_at, position);

CREATE INDEX IF NOT EXISTS event_log_event_type_idx
    ON event_log (event_type, recorded_at);

CREATE INDEX IF NOT EXISTS event_log_correlation_id_idx
    ON event_log (correlation_id)
    WHERE correlation_id IS NOT NULL;

CREATE UNIQUE INDEX IF NOT EXISTS event_log_source_idempotency_idx
    ON event_log (
        event_type,
        (source ->> 'kind'),
        COALESCE(source ->> 'provider', ''),
        (source ->> 'source_id')
    )
    WHERE source ? 'source_id';

CREATE OR REPLACE FUNCTION prevent_event_log_mutation()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
    RAISE EXCEPTION 'event_log is append-only';
END;
$$;

DROP TRIGGER IF EXISTS event_log_prevent_update ON event_log;
CREATE TRIGGER event_log_prevent_update
    BEFORE UPDATE ON event_log
    FOR EACH ROW
    EXECUTE FUNCTION prevent_event_log_mutation();

DROP TRIGGER IF EXISTS event_log_prevent_delete ON event_log;
CREATE TRIGGER event_log_prevent_delete
    BEFORE DELETE ON event_log
    FOR EACH ROW
    EXECUTE FUNCTION prevent_event_log_mutation();

