ALTER TABLE tasks
    DROP CONSTRAINT IF EXISTS tasks_source_type_check;

ALTER TABLE tasks
    ADD CONSTRAINT tasks_source_type_check CHECK (
        source_type IN (
            'manual',
            'observation',
            'communication',
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
