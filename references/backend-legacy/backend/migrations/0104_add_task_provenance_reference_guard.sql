-- Enforce that task provenance references an existing record.
CREATE OR REPLACE FUNCTION enforce_task_provenance_target()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
    IF NEW.provenance_kind = 'observation' THEN
        IF NOT EXISTS (SELECT 1 FROM observations WHERE observation_id = NEW.provenance_id) THEN
            RAISE EXCEPTION 'tasks.provenance_id must reference an existing observation'
                USING ERRCODE = '23503';
        END IF;
    ELSIF NEW.provenance_kind = 'review_item' THEN
        IF NOT EXISTS (SELECT 1 FROM review_items WHERE review_item_id = NEW.provenance_id) THEN
            RAISE EXCEPTION 'tasks.provenance_id must reference an existing review item'
                USING ERRCODE = '23503';
        END IF;
    ELSIF NEW.provenance_kind = 'decision' THEN
        IF NOT EXISTS (SELECT 1 FROM decisions WHERE decision_id = NEW.provenance_id) THEN
            RAISE EXCEPTION 'tasks.provenance_id must reference an existing decision'
                USING ERRCODE = '23503';
        END IF;
    ELSIF NEW.provenance_kind = 'obligation' THEN
        IF NOT EXISTS (SELECT 1 FROM obligations WHERE obligation_id = NEW.provenance_id) THEN
            RAISE EXCEPTION 'tasks.provenance_id must reference an existing obligation'
                USING ERRCODE = '23503';
        END IF;
    ELSE
        RAISE EXCEPTION 'unsupported tasks.provenance_kind value'
            USING ERRCODE = '23514';
    END IF;

    RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS tasks_provenance_target_guard ON tasks;
CREATE TRIGGER tasks_provenance_target_guard
    BEFORE INSERT OR UPDATE OF provenance_kind, provenance_id ON tasks
    FOR EACH ROW
    EXECUTE FUNCTION enforce_task_provenance_target();
