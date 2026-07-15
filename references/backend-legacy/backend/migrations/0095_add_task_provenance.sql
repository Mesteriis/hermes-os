ALTER TABLE tasks
    ADD COLUMN IF NOT EXISTS provenance_kind TEXT,
    ADD COLUMN IF NOT EXISTS provenance_id TEXT;

UPDATE tasks
SET
    provenance_kind = COALESCE(
        provenance_kind,
        CASE
            WHEN source_kind IN ('manual', 'message', 'email', 'telegram', 'whatsapp', 'calendar', 'meeting', 'document', 'note', 'jira', 'youtrack', 'github', 'gitlab', 'linear', 'todoist', 'apple_reminders', 'ms_todo', 'ai_rule', 'workflow', 'import')
                THEN 'observation'
            ELSE 'review_item'
        END
    ),
    provenance_id = COALESCE(provenance_id, source_id)
WHERE provenance_kind IS NULL
   OR provenance_id IS NULL;

ALTER TABLE tasks
    ALTER COLUMN provenance_kind SET NOT NULL,
    ALTER COLUMN provenance_id SET NOT NULL;

ALTER TABLE tasks
    DROP CONSTRAINT IF EXISTS tasks_provenance_kind_check;

ALTER TABLE tasks
    ADD CONSTRAINT tasks_provenance_kind_check CHECK (
        provenance_kind IN ('observation', 'review_item', 'decision', 'obligation')
    );

ALTER TABLE tasks
    ADD CONSTRAINT tasks_provenance_id_not_empty CHECK (length(trim(provenance_id)) > 0);

CREATE INDEX IF NOT EXISTS tasks_provenance_idx
    ON tasks (provenance_kind, provenance_id);
