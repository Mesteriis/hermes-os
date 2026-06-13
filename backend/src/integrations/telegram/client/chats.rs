use chrono::Utc;
use serde_json::json;

use crate::domains::mail::core::{CommunicationIngestionStore, NewRawCommunicationRecord};
use crate::integrations::telegram::tdjson::TelegramTdlibChatSnapshot;

use super::TELEGRAM_CHAT_RECORD_KIND;
use super::errors::TelegramError;
use super::identifiers::{stable_hash, telegram_chat_id, telegram_raw_record_id};
use super::models::{NewTelegramChat, TelegramChat, TelegramSyncState};
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
                metadata = EXCLUDED.metadata,
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
