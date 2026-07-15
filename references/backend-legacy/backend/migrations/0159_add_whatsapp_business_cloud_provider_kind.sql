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
            'whatsapp_web',
            'whatsapp_business_cloud'
        )
    );

ALTER TABLE whatsapp_web_sessions
    DROP CONSTRAINT IF EXISTS whatsapp_web_sessions_runtime;

ALTER TABLE whatsapp_web_sessions
    ADD CONSTRAINT whatsapp_web_sessions_runtime CHECK (
        companion_runtime IN ('fixture', 'manual_webview', 'blocked', 'api_credentials')
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
            'whatsapp_web_session_key',
            'whatsapp_business_cloud_access_token'
        )
    );
