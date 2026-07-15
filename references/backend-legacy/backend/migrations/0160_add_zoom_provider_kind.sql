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
            'whatsapp_business_cloud',
            'zoom_user',
            'zoom_server_to_server'
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
            'whatsapp_web_session_key',
            'whatsapp_business_cloud_access_token',
            'whatsapp_business_cloud_app_secret',
            'whatsapp_business_cloud_webhook_verify_token',
            'zoom_oauth_token',
            'zoom_client_secret',
            'zoom_webhook_secret'
        )
    );

CREATE INDEX IF NOT EXISTS telegram_calls_zoom_provider_idx
    ON telegram_calls (account_id, provider_call_id, created_at DESC)
    WHERE metadata->>'provider' = 'zoom';
