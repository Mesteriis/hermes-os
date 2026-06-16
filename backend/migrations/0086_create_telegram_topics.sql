-- Forum topics for supergroup/channel chats with forum mode enabled.
-- Each topic maps to a TDLib forumTopic with a stable provider_topic_id (BIGINT).
-- Messages belong to a topic via metadata->>'forum_topic_id'; see ADR-0091.

CREATE TABLE IF NOT EXISTS telegram_topics (
    topic_id            TEXT PRIMARY KEY,
    telegram_chat_id    TEXT NOT NULL REFERENCES telegram_chats(telegram_chat_id) ON DELETE CASCADE,
    account_id          TEXT NOT NULL,
    provider_topic_id   BIGINT NOT NULL,
    provider_chat_id    TEXT NOT NULL,
    title               TEXT NOT NULL,
    icon_emoji          TEXT,
    is_pinned           BOOLEAN NOT NULL DEFAULT FALSE,
    is_closed           BOOLEAN NOT NULL DEFAULT FALSE,
    unread_count        INT NOT NULL DEFAULT 0,
    last_message_at     TIMESTAMPTZ,
    metadata            JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),

    UNIQUE (telegram_chat_id, provider_topic_id)
);

CREATE INDEX IF NOT EXISTS idx_telegram_topics_chat
    ON telegram_topics (telegram_chat_id, updated_at DESC);

CREATE INDEX IF NOT EXISTS idx_telegram_topics_account
    ON telegram_topics (account_id, updated_at DESC);

-- Index for message-per-topic queries via message_metadata JSONB
CREATE INDEX IF NOT EXISTS idx_comm_messages_forum_topic_id
    ON communication_messages ((message_metadata->>'forum_topic_id'))
    WHERE message_metadata->>'forum_topic_id' IS NOT NULL;
