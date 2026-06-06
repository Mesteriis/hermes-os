ALTER TABLE communication_provider_accounts
    DROP CONSTRAINT IF EXISTS communication_provider_account_kind;

ALTER TABLE communication_provider_accounts
    ADD CONSTRAINT communication_provider_account_kind CHECK (
        provider_kind IN (
            'gmail',
            'icloud',
            'imap',
            'telegram_user',
            'telegram_bot',
            'whatsapp_web'
        )
    );

ALTER TABLE communication_provider_account_secret_refs
    DROP CONSTRAINT IF EXISTS communication_provider_account_secret_purpose;

ALTER TABLE communication_provider_account_secret_refs
    ADD CONSTRAINT communication_provider_account_secret_purpose CHECK (
        secret_purpose IN (
            'oauth_token',
            'imap_password',
            'smtp_password',
            'telegram_api_hash',
            'telegram_session_key',
            'telegram_bot_token',
            'whatsapp_web_session_key'
        )
    );

ALTER TABLE communication_messages
    DROP CONSTRAINT IF EXISTS communication_messages_channel_kind;

ALTER TABLE communication_messages
    ADD CONSTRAINT communication_messages_channel_kind CHECK (
        channel_kind IN ('email', 'telegram_user', 'telegram_bot', 'whatsapp_web')
    );

CREATE TABLE IF NOT EXISTS whatsapp_web_sessions (
    session_id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL REFERENCES communication_provider_accounts(account_id) ON DELETE RESTRICT,
    device_name TEXT NOT NULL,
    companion_runtime TEXT NOT NULL DEFAULT 'fixture',
    link_state TEXT NOT NULL DEFAULT 'fixture',
    local_state_path TEXT NOT NULL,
    last_sync_at TIMESTAMPTZ,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT whatsapp_web_sessions_account_unique UNIQUE (account_id),
    CONSTRAINT whatsapp_web_sessions_device_name_not_empty CHECK (length(trim(device_name)) > 0),
    CONSTRAINT whatsapp_web_sessions_runtime CHECK (
        companion_runtime IN ('fixture', 'manual_webview', 'blocked')
    ),
    CONSTRAINT whatsapp_web_sessions_link_state CHECK (
        link_state IN ('fixture', 'qr_pending', 'linked', 'degraded', 'revoked', 'blocked')
    ),
    CONSTRAINT whatsapp_web_sessions_local_state_path_not_empty CHECK (length(trim(local_state_path)) > 0),
    CONSTRAINT whatsapp_web_sessions_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE INDEX IF NOT EXISTS whatsapp_web_sessions_state_idx
    ON whatsapp_web_sessions (link_state, updated_at DESC);
