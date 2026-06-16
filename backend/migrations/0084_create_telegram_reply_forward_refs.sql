-- Migration 0084: Telegram reply and forward reference tracking
-- ADR-0091: reply targets, reply chains, forward attribution, forward chains

-- ---------------------------------------------------------------------------
-- telegram_message_reply_refs — reply target and reply chain
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS telegram_message_reply_refs (
    reply_ref_id        TEXT PRIMARY KEY,
    source_message_id   TEXT NOT NULL,
    target_message_id   TEXT NOT NULL,
    account_id          TEXT NOT NULL
        REFERENCES communication_provider_accounts(account_id) ON DELETE RESTRICT,
    provider_chat_id    TEXT NOT NULL,
    source_provider_id  TEXT NOT NULL,
    target_provider_id  TEXT NOT NULL,
    reply_depth         INTEGER NOT NULL DEFAULT 1,
    is_topic_reply      BOOLEAN NOT NULL DEFAULT false,
    topic_id            TEXT,
    metadata            JSONB NOT NULL DEFAULT '{}'::jsonb,
    provenance          JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT telegram_message_reply_refs_source_not_empty
        CHECK (length(trim(source_message_id)) > 0),
    CONSTRAINT telegram_message_reply_refs_target_not_empty
        CHECK (length(trim(target_message_id)) > 0),
    CONSTRAINT telegram_message_reply_refs_reply_depth_positive
        CHECK (reply_depth > 0),
    CONSTRAINT telegram_message_reply_refs_metadata_is_object
        CHECK (jsonb_typeof(metadata) = 'object'),
    CONSTRAINT telegram_message_reply_refs_provenance_is_object
        CHECK (jsonb_typeof(provenance) = 'object'),
    CONSTRAINT telegram_message_reply_refs_unique
        UNIQUE (source_message_id, target_message_id)
);

CREATE INDEX IF NOT EXISTS telegram_message_reply_refs_target_idx
    ON telegram_message_reply_refs (target_message_id, created_at DESC);

CREATE INDEX IF NOT EXISTS telegram_message_reply_refs_source_idx
    ON telegram_message_reply_refs (source_message_id, created_at DESC);

CREATE INDEX IF NOT EXISTS telegram_message_reply_refs_chat_idx
    ON telegram_message_reply_refs (account_id, provider_chat_id);

-- ---------------------------------------------------------------------------
-- telegram_message_forward_refs — forward attribution and chains
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS telegram_message_forward_refs (
    forward_ref_id          TEXT PRIMARY KEY,
    source_message_id       TEXT NOT NULL,
    account_id              TEXT NOT NULL
        REFERENCES communication_provider_accounts(account_id) ON DELETE RESTRICT,
    provider_chat_id        TEXT NOT NULL,
    source_provider_id      TEXT NOT NULL,
    forward_origin_chat_id  TEXT,
    forward_origin_message_id TEXT,
    forward_origin_sender_id TEXT,
    forward_origin_sender_name TEXT,
    forward_date            TIMESTAMPTZ,
    forward_depth           INTEGER NOT NULL DEFAULT 1,
    metadata                JSONB NOT NULL DEFAULT '{}'::jsonb,
    provenance              JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT telegram_message_forward_refs_source_not_empty
        CHECK (length(trim(source_message_id)) > 0),
    CONSTRAINT telegram_message_forward_refs_forward_depth_positive
        CHECK (forward_depth > 0),
    CONSTRAINT telegram_message_forward_refs_metadata_is_object
        CHECK (jsonb_typeof(metadata) = 'object'),
    CONSTRAINT telegram_message_forward_refs_provenance_is_object
        CHECK (jsonb_typeof(provenance) = 'object'),
    CONSTRAINT telegram_message_forward_refs_unique
        UNIQUE (source_message_id, account_id)
);

CREATE INDEX IF NOT EXISTS telegram_message_forward_refs_source_idx
    ON telegram_message_forward_refs (source_message_id);

CREATE INDEX IF NOT EXISTS telegram_message_forward_refs_chat_idx
    ON telegram_message_forward_refs (account_id, provider_chat_id);
