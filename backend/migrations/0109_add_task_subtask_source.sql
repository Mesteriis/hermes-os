ALTER TABLE task_subtasks
    ADD COLUMN IF NOT EXISTS source TEXT NOT NULL DEFAULT 'manual';

UPDATE task_subtasks
SET source = 'manual'
WHERE source IS NULL OR btrim(source) = '';

