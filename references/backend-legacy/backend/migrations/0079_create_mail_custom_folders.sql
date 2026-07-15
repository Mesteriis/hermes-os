CREATE TABLE IF NOT EXISTS mail_folders (
    folder_id TEXT PRIMARY KEY,
    account_id TEXT REFERENCES communication_provider_accounts(account_id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    color TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT mail_folders_id_not_empty CHECK (length(trim(folder_id)) > 0),
    CONSTRAINT mail_folders_name_not_empty CHECK (length(trim(name)) > 0),
    CONSTRAINT mail_folders_description_not_empty CHECK (
        description IS NULL OR length(trim(description)) > 0
    ),
    CONSTRAINT mail_folders_color_not_empty CHECK (
        color IS NULL OR length(trim(color)) > 0
    )
);

CREATE UNIQUE INDEX IF NOT EXISTS mail_folders_account_name_unique_idx
    ON mail_folders (COALESCE(account_id, ''), lower(name));

CREATE INDEX IF NOT EXISTS mail_folders_account_order_idx
    ON mail_folders (account_id, sort_order, lower(name), folder_id);

CREATE TABLE IF NOT EXISTS mail_folder_messages (
    folder_id TEXT NOT NULL REFERENCES mail_folders(folder_id) ON DELETE CASCADE,
    message_id TEXT NOT NULL REFERENCES communication_messages(message_id) ON DELETE CASCADE,
    added_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_operation TEXT NOT NULL DEFAULT 'copy',

    PRIMARY KEY (folder_id, message_id),
    CONSTRAINT mail_folder_messages_operation CHECK (last_operation IN ('copy', 'move'))
);

CREATE INDEX IF NOT EXISTS mail_folder_messages_message_idx
    ON mail_folder_messages (message_id, added_at DESC);

CREATE INDEX IF NOT EXISTS mail_folder_messages_folder_order_idx
    ON mail_folder_messages (folder_id, added_at DESC, message_id);
