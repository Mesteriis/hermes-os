CREATE TABLE IF NOT EXISTS mail_saved_searches (
    saved_search_id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    account_id TEXT REFERENCES communication_provider_accounts(account_id) ON DELETE CASCADE,
    query_text TEXT NOT NULL DEFAULT '',
    workflow_state TEXT,
    local_state TEXT NOT NULL DEFAULT 'active',
    channel_kind TEXT,
    is_smart_folder BOOLEAN NOT NULL DEFAULT false,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT mail_saved_searches_id_not_empty CHECK (length(trim(saved_search_id)) > 0),
    CONSTRAINT mail_saved_searches_name_not_empty CHECK (length(trim(name)) > 0),
    CONSTRAINT mail_saved_searches_query_not_blank_or_filters CHECK (
        length(trim(query_text)) > 0
        OR workflow_state IS NOT NULL
        OR channel_kind IS NOT NULL
        OR account_id IS NOT NULL
        OR local_state <> 'active'
    ),
    CONSTRAINT mail_saved_searches_workflow_state CHECK (
        workflow_state IS NULL
        OR workflow_state IN ('new', 'reviewed', 'needs_action', 'waiting', 'done', 'archived', 'muted', 'spam')
    ),
    CONSTRAINT mail_saved_searches_local_state CHECK (local_state IN ('active', 'trash', 'all')),
    CONSTRAINT mail_saved_searches_description_not_blank CHECK (
        description IS NULL OR length(trim(description)) > 0
    ),
    CONSTRAINT mail_saved_searches_channel_kind_not_blank CHECK (
        channel_kind IS NULL OR length(trim(channel_kind)) > 0
    )
);

CREATE INDEX IF NOT EXISTS mail_saved_searches_account_smart_idx
    ON mail_saved_searches (account_id, is_smart_folder, sort_order, lower(name));

CREATE INDEX IF NOT EXISTS mail_saved_searches_smart_idx
    ON mail_saved_searches (is_smart_folder, sort_order, lower(name));
