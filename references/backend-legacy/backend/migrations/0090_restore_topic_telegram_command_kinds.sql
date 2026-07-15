-- Migration 0090: restore topic provider-write command kinds after 0085 narrowed the allowlist

ALTER TABLE telegram_provider_write_commands
    DROP CONSTRAINT IF EXISTS telegram_provider_write_commands_command_kind;

ALTER TABLE telegram_provider_write_commands
    ADD CONSTRAINT telegram_provider_write_commands_command_kind
        CHECK (command_kind IN (
            'send_text',
            'send_media',
            'edit',
            'delete',
            'restore_visibility',
            'mark_read',
            'mark_unread',
            'pin',
            'unpin',
            'archive',
            'unarchive',
            'mute',
            'unmute',
            'react',
            'unreact',
            'reply',
            'forward',
            'join',
            'leave',
            'topic_create',
            'topic_close',
            'topic_reopen',
            'admin_action'
        ));
