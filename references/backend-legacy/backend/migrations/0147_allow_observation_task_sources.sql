ALTER TABLE tasks
    DROP CONSTRAINT IF EXISTS tasks_source_kind_check;

ALTER TABLE tasks
    ADD CONSTRAINT tasks_source_kind_check CHECK (
        source_kind IN (
            'manual',
            'observation',
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
