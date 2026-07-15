CREATE TABLE IF NOT EXISTS api_audit_log (
    audit_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    actor_kind TEXT NOT NULL,
    operation TEXT NOT NULL,
    method TEXT NOT NULL,
    path_template TEXT NOT NULL,
    target_kind TEXT NOT NULL,
    target_id TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,

    CONSTRAINT api_audit_log_actor_kind_not_empty CHECK (length(trim(actor_kind)) > 0),
    CONSTRAINT api_audit_log_operation_not_empty CHECK (length(trim(operation)) > 0),
    CONSTRAINT api_audit_log_method_not_empty CHECK (length(trim(method)) > 0),
    CONSTRAINT api_audit_log_path_template_not_empty CHECK (length(trim(path_template)) > 0),
    CONSTRAINT api_audit_log_target_kind_not_empty CHECK (length(trim(target_kind)) > 0),
    CONSTRAINT api_audit_log_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE INDEX IF NOT EXISTS api_audit_log_recorded_at_idx
    ON api_audit_log (recorded_at, audit_id);

CREATE INDEX IF NOT EXISTS api_audit_log_operation_idx
    ON api_audit_log (operation, recorded_at);

CREATE INDEX IF NOT EXISTS api_audit_log_target_idx
    ON api_audit_log (target_kind, target_id, recorded_at)
    WHERE target_id IS NOT NULL;

CREATE OR REPLACE FUNCTION prevent_api_audit_log_mutation()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
    RAISE EXCEPTION 'api_audit_log is append-only';
END;
$$;

DROP TRIGGER IF EXISTS api_audit_log_prevent_update ON api_audit_log;
CREATE TRIGGER api_audit_log_prevent_update
    BEFORE UPDATE ON api_audit_log
    FOR EACH ROW
    EXECUTE FUNCTION prevent_api_audit_log_mutation();

DROP TRIGGER IF EXISTS api_audit_log_prevent_delete ON api_audit_log;
CREATE TRIGGER api_audit_log_prevent_delete
    BEFORE DELETE ON api_audit_log
    FOR EACH ROW
    EXECUTE FUNCTION prevent_api_audit_log_mutation();
