ALTER TABLE api_audit_log
    ADD COLUMN actor_id TEXT;

ALTER TABLE api_audit_log
    ADD CONSTRAINT api_audit_log_actor_id_not_empty
    CHECK (actor_id IS NULL OR length(trim(actor_id)) > 0);

CREATE INDEX api_audit_log_actor_idx
    ON api_audit_log (actor_kind, actor_id, recorded_at)
    WHERE actor_id IS NOT NULL;
