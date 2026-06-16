use chrono::{DateTime, Utc};
use serde_json::json;

use crate::domains::mail::core::{CommunicationIngestionStore, NewRawCommunicationRecord};
use crate::integrations::telegram::tdjson::TelegramTdlibChatSnapshot;

use super::TELEGRAM_CHAT_RECORD_KIND;
use super::errors::TelegramError;
use super::identifiers::{stable_hash, telegram_chat_id, telegram_raw_record_id};
use super::models::{
    NewTelegramChat, TelegramChat, TelegramChatGroupFilter, TelegramChatMember, TelegramSyncState,
};
use super::rows::row_to_telegram_chat;
use super::store::TelegramStore;
use super::validation::validate_chat_list_limit;

impl TelegramStore {
    pub async fn upsert_chat(&self, chat: &NewTelegramChat) -> Result<TelegramChat, TelegramError> {
        chat.validate()?;
        let telegram_chat_id = telegram_chat_id(&chat.account_id, &chat.provider_chat_id);
        let row = sqlx::query(
            r#"
            INSERT INTO telegram_chats (
                telegram_chat_id,
                account_id,
                provider_chat_id,
                chat_kind,
                title,
                username,
                sync_state,
                last_message_at,
                metadata,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, now())
            ON CONFLICT (account_id, provider_chat_id)
            DO UPDATE SET
                chat_kind = EXCLUDED.chat_kind,
                title = EXCLUDED.title,
                username = EXCLUDED.username,
                sync_state = EXCLUDED.sync_state,
                last_message_at = EXCLUDED.last_message_at,
                metadata = COALESCE(telegram_chats.metadata, '{}'::jsonb) || EXCLUDED.metadata,
                updated_at = now()
            RETURNING
                telegram_chat_id,
                account_id,
                provider_chat_id,
                chat_kind,
                title,
                username,
                sync_state,
                last_message_at,
                metadata,
                created_at,
                updated_at
            "#,
        )
        .bind(&telegram_chat_id)
        .bind(chat.account_id.trim())
        .bind(chat.provider_chat_id.trim())
        .bind(chat.chat_kind.as_str())
        .bind(chat.title.trim())
        .bind(
            chat.username
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty()),
        )
        .bind(chat.sync_state.as_str())
        .bind(chat.last_message_at)
        .bind(&chat.metadata)
        .fetch_one(&self.pool)
        .await?;

        row_to_telegram_chat(row)
    }

    pub async fn list_chats(
        &self,
        account_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<TelegramChat>, TelegramError> {
        let limit = validate_chat_list_limit(limit)?;
        let account_id = account_id.map(str::trim).filter(|value| !value.is_empty());
        let rows = sqlx::query(
            r#"
            SELECT
                telegram_chat_id,
                account_id,
                provider_chat_id,
                chat_kind,
                title,
                username,
                sync_state,
                last_message_at,
                metadata,
                created_at,
                updated_at
            FROM telegram_chats
            WHERE ($1::text IS NULL OR account_id = $1)
            ORDER BY COALESCE(last_message_at, updated_at) DESC, telegram_chat_id ASC
            LIMIT $2
            "#,
        )
        .bind(account_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_telegram_chat).collect()
    }

    pub async fn list_chat_group_filters(
        &self,
        account_id: Option<&str>,
    ) -> Result<Vec<TelegramChatGroupFilter>, TelegramError> {
        let account_id = account_id.map(str::trim).filter(|value| !value.is_empty());
        let rows = sqlx::query_as::<_, (String, String, String, i64, String)>(
            r#"
            SELECT id, label, source, count, icon
            FROM (
                SELECT
                    'local:all'::text AS id,
                    'All'::text AS label,
                    'local'::text AS source,
                    COUNT(*)::bigint AS count,
                    'tabler:message'::text AS icon
                FROM telegram_chats
                WHERE ($1::text IS NULL OR account_id = $1)

                UNION ALL

                SELECT
                    'folder:' || BTRIM(metadata->>'folder_name') AS id,
                    BTRIM(metadata->>'folder_name') AS label,
                    'telegram'::text AS source,
                    COUNT(*)::bigint AS count,
                    'tabler:folder'::text AS icon
                FROM telegram_chats
                WHERE ($1::text IS NULL OR account_id = $1)
                  AND NULLIF(BTRIM(metadata->>'folder_name'), '') IS NOT NULL
                GROUP BY BTRIM(metadata->>'folder_name')
            ) filters
            ORDER BY
                CASE WHEN source = 'local' THEN 0 ELSE 1 END,
                label ASC
            "#,
        )
        .bind(account_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(id, label, source, count, icon)| TelegramChatGroupFilter {
                id,
                label,
                source,
                count,
                icon,
            })
            .collect())
    }

    pub async fn set_chat_metadata_bool(
        &self,
        telegram_chat_id: &str,
        key: &str,
        value: bool,
    ) -> Result<serde_json::Value, TelegramError> {
        let mut metadata = self.chat_metadata_map(telegram_chat_id).await?;
        metadata.insert(key.to_owned(), serde_json::Value::Bool(value));
        self.persist_chat_metadata(telegram_chat_id, metadata).await
    }

    pub async fn set_chat_metadata_number(
        &self,
        telegram_chat_id: &str,
        key: &str,
        value: i64,
    ) -> Result<serde_json::Value, TelegramError> {
        let mut metadata = self.chat_metadata_map(telegram_chat_id).await?;
        metadata.insert(
            key.to_owned(),
            serde_json::Value::Number(serde_json::Number::from(value.max(0))),
        );
        self.persist_chat_metadata(telegram_chat_id, metadata).await
    }

    pub async fn set_chat_last_read_at(
        &self,
        telegram_chat_id: &str,
        last_read_at: Option<DateTime<Utc>>,
    ) -> Result<serde_json::Value, TelegramError> {
        let mut metadata = self.chat_metadata_map(telegram_chat_id).await?;
        match last_read_at {
            Some(value) => {
                metadata.insert(
                    "last_read_at".to_owned(),
                    serde_json::Value::String(value.to_rfc3339()),
                );
            }
            None => {
                metadata.remove("last_read_at");
            }
        }
        self.persist_chat_metadata(telegram_chat_id, metadata).await
    }

    pub async fn recompute_chat_unread_count(
        &self,
        telegram_chat_id: &str,
    ) -> Result<serde_json::Value, TelegramError> {
        let chat = self
            .telegram_chat_by_id(telegram_chat_id)
            .await?
            .ok_or_else(|| {
                TelegramError::InvalidRequest(format!(
                    "Telegram chat `{telegram_chat_id}` was not found"
                ))
            })?;
        let mut metadata = self.chat_metadata_map(telegram_chat_id).await?;
        let last_read_at = metadata
            .get("last_read_at")
            .and_then(serde_json::Value::as_str)
            .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
            .map(|value| value.with_timezone(&Utc));
        let unread_count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)::bigint
            FROM communication_messages
            WHERE account_id = $1
              AND conversation_id = $2
              AND channel_kind IN ('telegram_user', 'telegram_bot')
              AND delivery_state = 'received'
              AND ($3::timestamptz IS NULL OR COALESCE(occurred_at, projected_at) > $3)
            "#,
        )
        .bind(&chat.account_id)
        .bind(&chat.provider_chat_id)
        .bind(last_read_at)
        .fetch_one(&self.pool)
        .await?;
        let mention_count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COALESCE(SUM(
                CASE
                    WHEN jsonb_typeof(message_metadata->'mention_count') = 'number'
                        THEN (message_metadata->>'mention_count')::bigint
                    ELSE 0
                END
            ), 0)::bigint
            FROM communication_messages
            WHERE account_id = $1
              AND conversation_id = $2
              AND channel_kind IN ('telegram_user', 'telegram_bot')
              AND delivery_state = 'received'
              AND ($3::timestamptz IS NULL OR COALESCE(occurred_at, projected_at) > $3)
            "#,
        )
        .bind(&chat.account_id)
        .bind(&chat.provider_chat_id)
        .bind(last_read_at)
        .fetch_one(&self.pool)
        .await?;
        metadata.insert(
            "unread_count".to_owned(),
            serde_json::Value::Number(serde_json::Number::from(unread_count.max(0))),
        );
        metadata.insert(
            "mention_count".to_owned(),
            serde_json::Value::Number(serde_json::Number::from(mention_count.max(0))),
        );
        self.persist_chat_metadata(telegram_chat_id, metadata).await
    }

    pub async fn telegram_chat_by_id(
        &self,
        telegram_chat_id: &str,
    ) -> Result<Option<TelegramChat>, TelegramError> {
        let row = sqlx::query(
            r#"
            SELECT
                telegram_chat_id,
                account_id,
                provider_chat_id,
                chat_kind,
                title,
                username,
                sync_state,
                last_message_at,
                metadata,
                created_at,
                updated_at
            FROM telegram_chats
            WHERE telegram_chat_id = $1
            "#,
        )
        .bind(telegram_chat_id.trim())
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_telegram_chat).transpose()
    }

    async fn chat_metadata_map(
        &self,
        telegram_chat_id: &str,
    ) -> Result<serde_json::Map<String, serde_json::Value>, TelegramError> {
        let row: Option<(serde_json::Value,)> =
            sqlx::query_as("SELECT metadata FROM telegram_chats WHERE telegram_chat_id = $1")
                .bind(telegram_chat_id)
                .fetch_optional(&self.pool)
                .await?;
        let metadata = row
            .ok_or_else(|| {
                TelegramError::InvalidRequest(format!(
                    "Telegram chat `{telegram_chat_id}` was not found"
                ))
            })?
            .0;
        Ok(metadata.as_object().cloned().unwrap_or_default())
    }

    async fn persist_chat_metadata(
        &self,
        telegram_chat_id: &str,
        metadata: serde_json::Map<String, serde_json::Value>,
    ) -> Result<serde_json::Value, TelegramError> {
        let metadata = serde_json::Value::Object(metadata);
        sqlx::query(
            "UPDATE telegram_chats SET metadata = $2, updated_at = now() WHERE telegram_chat_id = $1",
        )
        .bind(telegram_chat_id)
        .bind(&metadata)
        .execute(&self.pool)
        .await?;

        Ok(metadata)
    }

    pub async fn list_chat_members(
        &self,
        telegram_chat_id: &str,
        limit: i64,
    ) -> Result<Vec<TelegramChatMember>, TelegramError> {
        let limit = validate_chat_list_limit(limit)?;
        let chat = self
            .telegram_chat_by_id(telegram_chat_id)
            .await?
            .ok_or_else(|| {
                TelegramError::InvalidRequest(format!(
                    "Telegram chat `{telegram_chat_id}` was not found"
                ))
            })?;

        let rows =
            sqlx::query_as::<_, (String, Option<String>, i64, Option<chrono::DateTime<Utc>>)>(
                r#"
            SELECT
                sender,
                MAX(NULLIF(BTRIM(sender_display_name), '')) AS sender_display_name,
                COUNT(*)::bigint AS message_count,
                MAX(COALESCE(occurred_at, projected_at)) AS last_message_at
            FROM communication_messages
            WHERE account_id = $1
              AND conversation_id = $2
              AND channel_kind IN ('telegram_user', 'telegram_bot')
            GROUP BY sender
            ORDER BY message_count DESC, last_message_at DESC NULLS LAST, sender ASC
            LIMIT $3
            "#,
            )
            .bind(&chat.account_id)
            .bind(&chat.provider_chat_id)
            .bind(limit)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows
            .into_iter()
            .map(
                |(sender_id, sender_display_name, message_count, last_message_at)| {
                    TelegramChatMember {
                        sender_id,
                        sender_display_name,
                        message_count,
                        last_message_at,
                    }
                },
            )
            .collect())
    }

    pub(crate) async fn ingest_tdlib_chat_snapshot(
        &self,
        account_id: &str,
        snapshot: &TelegramTdlibChatSnapshot,
    ) -> Result<TelegramChat, TelegramError> {
        let communication_store = CommunicationIngestionStore::new(self.pool.clone());
        let provider_account = self
            .telegram_provider_account(&communication_store, account_id)
            .await?;
        let raw_record_id = telegram_raw_record_id(
            &provider_account.account_id,
            TELEGRAM_CHAT_RECORD_KIND,
            &snapshot.provider_chat_id,
        );
        let import_batch_id = format!("telegram-tdlib-chat-sync:{}", provider_account.account_id);
        let raw = NewRawCommunicationRecord::new(
            &raw_record_id,
            &provider_account.account_id,
            TELEGRAM_CHAT_RECORD_KIND,
            &snapshot.provider_chat_id,
            format!(
                "sha256:{}",
                stable_hash(snapshot.raw.to_string().as_bytes())
            ),
            &import_batch_id,
            snapshot.raw.clone(),
        )
        .occurred_at(snapshot.last_message_at.unwrap_or_else(Utc::now))
        .provenance(json!({
            "provider": "telegram",
            "provider_kind": provider_account.provider_kind.as_str(),
            "runtime": "tdlib",
            "account_id": provider_account.account_id,
            "provider_chat_id": snapshot.provider_chat_id,
        }));
        communication_store.record_raw_source(&raw).await?;

        self.upsert_chat(&NewTelegramChat {
            account_id: provider_account.account_id,
            provider_chat_id: snapshot.provider_chat_id.clone(),
            chat_kind: snapshot.chat_kind,
            title: snapshot.title.clone(),
            username: snapshot.username.clone(),
            sync_state: TelegramSyncState::Synced,
            last_message_at: snapshot.last_message_at,
            metadata: json!({
                "runtime": "tdlib",
                "raw_record_id": raw_record_id,
            }),
        })
        .await
    }
}
