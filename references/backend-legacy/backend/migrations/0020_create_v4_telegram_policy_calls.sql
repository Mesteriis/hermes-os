ALTER TABLE communication_provider_accounts
    DROP CONSTRAINT IF EXISTS communication_provider_account_kind;

ALTER TABLE communication_provider_accounts
    ADD CONSTRAINT communication_provider_account_kind CHECK (
        provider_kind IN ('gmail', 'icloud', 'imap', 'telegram_user', 'telegram_bot')
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
            'telegram_bot_token'
        )
    );

ALTER TABLE communication_messages
    ADD COLUMN IF NOT EXISTS channel_kind TEXT NOT NULL DEFAULT 'email',
    ADD COLUMN IF NOT EXISTS conversation_id TEXT,
    ADD COLUMN IF NOT EXISTS sender_display_name TEXT,
    ADD COLUMN IF NOT EXISTS delivery_state TEXT NOT NULL DEFAULT 'received',
    ADD COLUMN IF NOT EXISTS message_metadata JSONB NOT NULL DEFAULT '{}'::jsonb;

ALTER TABLE communication_messages
    ADD CONSTRAINT communication_messages_channel_kind CHECK (
        channel_kind IN ('email', 'telegram_user', 'telegram_bot')
    ),
    ADD CONSTRAINT communication_messages_delivery_state CHECK (
        delivery_state IN ('received', 'sent', 'send_dry_run', 'send_blocked')
    ),
    ADD CONSTRAINT communication_messages_metadata_is_object CHECK (
        jsonb_typeof(message_metadata) = 'object'
    );

CREATE TABLE IF NOT EXISTS telegram_chats (
    telegram_chat_id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL REFERENCES communication_provider_accounts(account_id) ON DELETE RESTRICT,
    provider_chat_id TEXT NOT NULL,
    chat_kind TEXT NOT NULL,
    title TEXT NOT NULL,
    username TEXT,
    sync_state TEXT NOT NULL DEFAULT 'fixture',
    last_message_at TIMESTAMPTZ,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT telegram_chats_provider_chat_id_not_empty CHECK (length(trim(provider_chat_id)) > 0),
    CONSTRAINT telegram_chats_title_not_empty CHECK (length(trim(title)) > 0),
    CONSTRAINT telegram_chats_kind CHECK (chat_kind IN ('private', 'group', 'channel', 'bot')),
    CONSTRAINT telegram_chats_sync_state CHECK (sync_state IN ('fixture', 'syncing', 'synced', 'degraded', 'error')),
    CONSTRAINT telegram_chats_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object'),
    CONSTRAINT telegram_chats_account_provider_unique UNIQUE (account_id, provider_chat_id)
);

CREATE INDEX IF NOT EXISTS telegram_chats_account_idx
    ON telegram_chats (account_id, updated_at DESC);

CREATE TABLE IF NOT EXISTS automation_templates (
    template_id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    body_template TEXT NOT NULL,
    required_variables JSONB NOT NULL DEFAULT '[]'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT automation_templates_name_not_empty CHECK (length(trim(name)) > 0),
    CONSTRAINT automation_templates_body_not_empty CHECK (length(trim(body_template)) > 0),
    CONSTRAINT automation_templates_required_variables_is_array CHECK (jsonb_typeof(required_variables) = 'array')
);

CREATE TABLE IF NOT EXISTS automation_policies (
    policy_id TEXT PRIMARY KEY,
    template_id TEXT NOT NULL REFERENCES automation_templates(template_id) ON DELETE RESTRICT,
    name TEXT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT false,
    account_id TEXT NOT NULL REFERENCES communication_provider_accounts(account_id) ON DELETE RESTRICT,
    allowed_chat_ids JSONB NOT NULL DEFAULT '[]'::jsonb,
    trigger_kind TEXT NOT NULL,
    max_sends_per_hour INTEGER NOT NULL,
    quiet_hours JSONB NOT NULL DEFAULT '{}'::jsonb,
    expires_at TIMESTAMPTZ,
    conditions JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT automation_policies_name_not_empty CHECK (length(trim(name)) > 0),
    CONSTRAINT automation_policies_trigger_kind_not_empty CHECK (length(trim(trigger_kind)) > 0),
    CONSTRAINT automation_policies_allowed_chat_ids_is_array CHECK (jsonb_typeof(allowed_chat_ids) = 'array'),
    CONSTRAINT automation_policies_quiet_hours_is_object CHECK (jsonb_typeof(quiet_hours) = 'object'),
    CONSTRAINT automation_policies_conditions_is_object CHECK (jsonb_typeof(conditions) = 'object'),
    CONSTRAINT automation_policies_max_sends_positive CHECK (max_sends_per_hour > 0)
);

CREATE INDEX IF NOT EXISTS automation_policies_account_idx
    ON automation_policies (account_id, enabled, updated_at DESC);

CREATE TABLE IF NOT EXISTS telegram_outbound_messages (
    outbound_message_id TEXT PRIMARY KEY,
    policy_id TEXT REFERENCES automation_policies(policy_id) ON DELETE RESTRICT,
    template_id TEXT REFERENCES automation_templates(template_id) ON DELETE RESTRICT,
    account_id TEXT NOT NULL REFERENCES communication_provider_accounts(account_id) ON DELETE RESTRICT,
    provider_chat_id TEXT NOT NULL,
    send_mode TEXT NOT NULL,
    status TEXT NOT NULL,
    rendered_preview_hash TEXT NOT NULL,
    variables JSONB NOT NULL DEFAULT '{}'::jsonb,
    source_context JSONB NOT NULL DEFAULT '{}'::jsonb,
    actor_id TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT telegram_outbound_provider_chat_not_empty CHECK (length(trim(provider_chat_id)) > 0),
    CONSTRAINT telegram_outbound_send_mode CHECK (send_mode IN ('dry_run', 'live')),
    CONSTRAINT telegram_outbound_status CHECK (status IN ('allowed', 'blocked', 'sent', 'failed')),
    CONSTRAINT telegram_outbound_preview_hash_not_empty CHECK (length(trim(rendered_preview_hash)) > 0),
    CONSTRAINT telegram_outbound_variables_is_object CHECK (jsonb_typeof(variables) = 'object'),
    CONSTRAINT telegram_outbound_source_context_is_object CHECK (jsonb_typeof(source_context) = 'object'),
    CONSTRAINT telegram_outbound_actor_not_empty CHECK (length(trim(actor_id)) > 0)
);

CREATE INDEX IF NOT EXISTS telegram_outbound_policy_idx
    ON telegram_outbound_messages (policy_id, created_at DESC);

CREATE TABLE IF NOT EXISTS telegram_calls (
    call_id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL REFERENCES communication_provider_accounts(account_id) ON DELETE RESTRICT,
    provider_call_id TEXT NOT NULL,
    provider_chat_id TEXT NOT NULL,
    direction TEXT NOT NULL,
    call_state TEXT NOT NULL,
    started_at TIMESTAMPTZ,
    ended_at TIMESTAMPTZ,
    transcription_policy_id TEXT REFERENCES automation_policies(policy_id) ON DELETE SET NULL,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT telegram_calls_provider_call_id_not_empty CHECK (length(trim(provider_call_id)) > 0),
    CONSTRAINT telegram_calls_provider_chat_id_not_empty CHECK (length(trim(provider_chat_id)) > 0),
    CONSTRAINT telegram_calls_direction CHECK (direction IN ('incoming', 'outgoing')),
    CONSTRAINT telegram_calls_state CHECK (call_state IN ('ringing', 'active', 'ended', 'missed', 'declined', 'failed')),
    CONSTRAINT telegram_calls_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object'),
    CONSTRAINT telegram_calls_account_provider_unique UNIQUE (account_id, provider_call_id)
);

CREATE INDEX IF NOT EXISTS telegram_calls_account_idx
    ON telegram_calls (account_id, created_at DESC);

CREATE TABLE IF NOT EXISTS call_transcripts (
    transcript_id TEXT PRIMARY KEY,
    call_id TEXT NOT NULL REFERENCES telegram_calls(call_id) ON DELETE RESTRICT,
    account_id TEXT NOT NULL REFERENCES communication_provider_accounts(account_id) ON DELETE RESTRICT,
    provider_chat_id TEXT NOT NULL,
    transcript_status TEXT NOT NULL,
    stt_provider TEXT NOT NULL,
    source_audio_ref TEXT,
    language_code TEXT,
    transcript_text TEXT NOT NULL,
    segments JSONB NOT NULL DEFAULT '[]'::jsonb,
    provenance JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT call_transcripts_provider_chat_id_not_empty CHECK (length(trim(provider_chat_id)) > 0),
    CONSTRAINT call_transcripts_status CHECK (transcript_status IN ('queued', 'running', 'succeeded', 'failed')),
    CONSTRAINT call_transcripts_stt_provider_not_empty CHECK (length(trim(stt_provider)) > 0),
    CONSTRAINT call_transcripts_segments_is_array CHECK (jsonb_typeof(segments) = 'array'),
    CONSTRAINT call_transcripts_provenance_is_object CHECK (jsonb_typeof(provenance) = 'object')
);

CREATE INDEX IF NOT EXISTS call_transcripts_call_idx
    ON call_transcripts (call_id, created_at DESC);
