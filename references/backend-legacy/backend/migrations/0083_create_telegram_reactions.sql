-- Migration 0083: Telegram message reactions
-- ADR-0091: reaction add/remove/sync with source-backed projection

CREATE TABLE IF NOT EXISTS telegram_message_reactions (
    reaction_id             TEXT PRIMARY KEY,
    message_id              TEXT NOT NULL,
    account_id              TEXT NOT NULL
        REFERENCES communication_provider_accounts(account_id) ON DELETE RESTRICT,
    provider_message_id     TEXT NOT NULL,
    provider_chat_id        TEXT NOT NULL,
    sender_id               TEXT NOT NULL,
    sender_display_name     TEXT,
    reaction_emoji          TEXT NOT NULL,
    is_active               BOOLEAN NOT NULL DEFAULT true,
    observed_at             TIMESTAMPTZ NOT NULL,
    source_event            TEXT,
    provider_actor_id       TEXT,
    metadata                JSONB NOT NULL DEFAULT '{}'::jsonb,
    provenance              JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT telegram_message_reactions_message_id_not_empty
        CHECK (length(trim(message_id)) > 0),
    CONSTRAINT telegram_message_reactions_provider_message_id_not_empty
        CHECK (length(trim(provider_message_id)) > 0),
    CONSTRAINT telegram_message_reactions_provider_chat_id_not_empty
        CHECK (length(trim(provider_chat_id)) > 0),
    CONSTRAINT telegram_message_reactions_sender_id_not_empty
        CHECK (length(trim(sender_id)) > 0),
    CONSTRAINT telegram_message_reactions_emoji_not_empty
        CHECK (length(trim(reaction_emoji)) > 0),
    CONSTRAINT telegram_message_reactions_metadata_is_object
        CHECK (jsonb_typeof(metadata) = 'object'),
    CONSTRAINT telegram_message_reactions_provenance_is_object
        CHECK (jsonb_typeof(provenance) = 'object'),
    CONSTRAINT telegram_message_reactions_unique_active
        UNIQUE (message_id, sender_id, reaction_emoji)
);

CREATE INDEX IF NOT EXISTS telegram_message_reactions_message_idx
    ON telegram_message_reactions (message_id, is_active, created_at DESC);

CREATE INDEX IF NOT EXISTS telegram_message_reactions_account_idx
    ON telegram_message_reactions (account_id, provider_chat_id, provider_message_id);
