CREATE INDEX IF NOT EXISTS event_log_trace_position_idx
    ON event_log (correlation_id, position)
    WHERE correlation_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS event_log_causation_id_position_idx
    ON event_log (causation_id, position)
    WHERE causation_id IS NOT NULL;
