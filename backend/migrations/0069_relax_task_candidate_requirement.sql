ALTER TABLE tasks
    ALTER COLUMN task_candidate_id DROP NOT NULL;

ALTER TABLE tasks
    ALTER COLUMN created_from_event_id DROP NOT NULL;

ALTER TABLE tasks
    ALTER COLUMN created_by_actor_id DROP NOT NULL;

ALTER TABLE tasks
    DROP CONSTRAINT IF EXISTS tasks_source_kind_check;

ALTER TABLE tasks
    ADD CONSTRAINT tasks_source_kind_check CHECK (
        source_kind IN (
            'manual',
            'message',
            'email',
            'telegram',
            'whatsapp',
            'calendar',
            'meeting',
            'document',
            'note',
            'jira',
            'youtrack',
            'github',
            'gitlab',
            'linear',
            'todoist',
            'apple_reminders',
            'ms_todo',
            'ai_rule',
            'workflow',
            'import'
        )
    );
