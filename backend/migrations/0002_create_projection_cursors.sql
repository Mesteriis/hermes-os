CREATE TABLE IF NOT EXISTS projection_cursors (
    projection_name TEXT PRIMARY KEY,
    last_processed_position BIGINT NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT projection_cursors_name_not_empty CHECK (length(trim(projection_name)) > 0),
    CONSTRAINT projection_cursors_position_non_negative CHECK (last_processed_position >= 0)
);

CREATE INDEX IF NOT EXISTS projection_cursors_updated_at_idx
    ON projection_cursors (updated_at);
