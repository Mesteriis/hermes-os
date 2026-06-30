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
            'zulip_bot',
            'zoom_user',
            'zoom_server_to_server',
            'yandex_telemost_user'
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
            'zulip_api_key',
            'zoom_oauth_token',
            'zoom_client_secret',
            'zoom_webhook_secret',
            'yandex_telemost_oauth_token'
        )
    );

ALTER TABLE communication_messages
    DROP CONSTRAINT IF EXISTS communication_messages_channel_kind;

ALTER TABLE communication_messages
    ADD CONSTRAINT communication_messages_channel_kind CHECK (
        channel_kind IN (
            'email',
            'telegram_user',
            'telegram_bot',
            'whatsapp_web',
            'whatsapp_business_cloud',
            'zulip'
        )
    );

ALTER TABLE person_identities
    DROP CONSTRAINT IF EXISTS person_identities_type_check;

ALTER TABLE person_identities
    ADD CONSTRAINT person_identities_type_check CHECK (identity_type IN (
        'email', 'telegram', 'whatsapp', 'zulip', 'phone',
        'github', 'linkedin', 'website',
        'mastodon', 'x', 'stackoverflow', 'habr',
        'medium', 'orcid', 'google_scholar',
        'document_mention', 'message_participant'
    ));
