-- Migration 0089: Telegram provider participant projection
-- Telegram remains a Communication Channel: participant rows are provider
-- communication projection state, not Persona/Organization lifecycle records.

CREATE TABLE IF NOT EXISTS telegram_chat_participants (
    participant_id       TEXT PRIMARY KEY,
    telegram_chat_id     TEXT NOT NULL
        REFERENCES telegram_chats(telegram_chat_id) ON DELETE CASCADE,
    account_id           TEXT NOT NULL
        REFERENCES communication_provider_accounts(account_id) ON DELETE RESTRICT,
    provider_chat_id     TEXT NOT NULL,
    provider_member_id   TEXT NOT NULL,
    display_name         TEXT,
    username             TEXT,
    role                 TEXT NOT NULL,
    status               TEXT NOT NULL,
    is_admin             BOOLEAN NOT NULL DEFAULT false,
    is_owner             BOOLEAN NOT NULL DEFAULT false,
    permissions          JSONB NOT NULL DEFAULT '{}'::jsonb,
    raw_payload          JSONB NOT NULL DEFAULT '{}'::jsonb,
    source               TEXT NOT NULL DEFAULT 'tdlib',
    observed_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at           TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at           TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT telegram_chat_participants_participant_id_not_empty
        CHECK (length(trim(participant_id)) > 0),
    CONSTRAINT telegram_chat_participants_provider_chat_id_not_empty
        CHECK (length(trim(provider_chat_id)) > 0),
    CONSTRAINT telegram_chat_participants_provider_member_id_not_empty
        CHECK (length(trim(provider_member_id)) > 0),
    CONSTRAINT telegram_chat_participants_role_not_empty
        CHECK (length(trim(role)) > 0),
    CONSTRAINT telegram_chat_participants_status_not_empty
        CHECK (length(trim(status)) > 0),
    CONSTRAINT telegram_chat_participants_permissions_is_object
        CHECK (jsonb_typeof(permissions) = 'object'),
    CONSTRAINT telegram_chat_participants_raw_payload_is_object
        CHECK (jsonb_typeof(raw_payload) = 'object'),
    CONSTRAINT telegram_chat_participants_source
        CHECK (source IN ('tdlib', 'bot_api')),
    CONSTRAINT telegram_chat_participants_unique_provider_member
        UNIQUE (telegram_chat_id, provider_member_id)
);

CREATE INDEX IF NOT EXISTS telegram_chat_participants_chat_idx
    ON telegram_chat_participants (telegram_chat_id, role, updated_at DESC);

CREATE INDEX IF NOT EXISTS telegram_chat_participants_account_chat_idx
    ON telegram_chat_participants (account_id, provider_chat_id, updated_at DESC);

CREATE INDEX IF NOT EXISTS telegram_chat_participants_search_idx
    ON telegram_chat_participants (
        telegram_chat_id,
        lower(coalesce(display_name, '')),
        lower(coalesce(username, '')),
        lower(provider_member_id)
    );
