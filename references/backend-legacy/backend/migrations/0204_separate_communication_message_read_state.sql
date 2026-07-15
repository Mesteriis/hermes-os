-- Read state is independent from the owner's workflow triage state.
ALTER TABLE communication_messages
    ADD COLUMN IF NOT EXISTS is_read BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS read_changed_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS read_origin TEXT NOT NULL DEFAULT 'migration_inferred';

UPDATE communication_messages
SET
    is_read = CASE
        WHEN COALESCE(message_metadata->'label_ids', '[]'::jsonb) ? 'UNREAD' THEN false
        WHEN workflow_state = 'new' THEN false
        ELSE true
    END,
    read_changed_at = COALESCE(read_changed_at, projected_at),
    read_origin = CASE
        WHEN COALESCE(message_metadata->'label_ids', '[]'::jsonb) ? 'UNREAD'
            THEN 'provider_observed'
        ELSE 'migration_inferred'
    END
WHERE read_changed_at IS NULL;

CREATE INDEX IF NOT EXISTS communication_messages_account_read_idx
    ON communication_messages (account_id, is_read, projected_at DESC);
