use chrono::Utc;
use serde_json::{Value, json};
use sqlx::Row;

use crate::domains::decisions::DecisionStore;
use crate::domains::mail::core::{
    CommunicationIngestionStore, NewRawCommunicationRecord, ProviderAccount,
};
use crate::domains::mail::messages::MessageProjectionStore;
use crate::domains::tasks::candidates::TaskCandidateStore;
use crate::integrations::telegram::tdjson::TelegramTdlibMessageSnapshot;

use super::TELEGRAM_MESSAGE_RECORD_KIND;
use super::errors::TelegramError;
use super::identifiers::{
    telegram_account_runtime, telegram_raw_record_id, telegram_text_preview_hash,
};
use super::models::{
    NewTelegramChat, NewTelegramMessage, TelegramAttachmentAnchor, TelegramChat, TelegramChatKind,
    TelegramDeliveryState, TelegramManualSendRequest, TelegramManualSendResponse, TelegramMessage,
    TelegramMessageIngestResult, TelegramSyncState,
};
use super::projection::project_raw_telegram_message;
use super::rows::{row_to_telegram_chat, row_to_telegram_message};
use super::store::TelegramStore;
use super::validation::validate_message_list_limit;

impl TelegramStore {
    pub async fn ingest_fixture_message(
        &self,
        message: &NewTelegramMessage,
    ) -> Result<TelegramMessageIngestResult, TelegramError> {
        self.ingest_message_with_runtime(message, "fixture", None)
            .await
    }

    pub(crate) async fn ingest_tdlib_message_snapshot(
        &self,
        account_id: &str,
        snapshot: &TelegramTdlibMessageSnapshot,
        import_batch_id: &str,
    ) -> Result<TelegramMessageIngestResult, TelegramError> {
        let communication_store = CommunicationIngestionStore::new(self.pool.clone());
        let provider_account = self
            .telegram_provider_account(&communication_store, account_id)
            .await?;
        let existing_chat = self
            .telegram_chat(&provider_account.account_id, &snapshot.provider_chat_id)
            .await?;
        let (chat_kind, chat_title) = match existing_chat {
            Some(chat) => (
                TelegramChatKind::try_from(chat.chat_kind.as_str())?,
                chat.title,
            ),
            None => (
                TelegramChatKind::Private,
                format!("Telegram Chat {}", snapshot.provider_chat_id),
            ),
        };
        let provider_message_id = format!(
            "{}:{}",
            snapshot.provider_chat_id, snapshot.provider_message_id
        );
        let message = NewTelegramMessage {
            account_id: provider_account.account_id,
            provider_chat_id: snapshot.provider_chat_id.clone(),
            provider_message_id,
            chat_kind,
            chat_title,
            sender_id: snapshot.sender_id.clone(),
            sender_display_name: snapshot.sender_display_name.clone(),
            text: snapshot.text.clone(),
            import_batch_id: import_batch_id.trim().to_owned(),
            occurred_at: snapshot.occurred_at,
            delivery_state: snapshot.delivery_state,
        };

        self.ingest_message_with_runtime(&message, "tdlib", Some(snapshot.raw.clone()))
            .await
    }

    async fn ingest_message_with_runtime(
        &self,
        message: &NewTelegramMessage,
        runtime_kind: &str,
        tdlib_raw: Option<Value>,
    ) -> Result<TelegramMessageIngestResult, TelegramError> {
        message.validate_for_runtime(runtime_kind)?;
        let communication_store = CommunicationIngestionStore::new(self.pool.clone());
        let provider_account = self
            .telegram_provider_account(&communication_store, &message.account_id)
            .await?;

        let chat = NewTelegramChat {
            account_id: message.account_id.clone(),
            provider_chat_id: message.provider_chat_id.clone(),
            chat_kind: message.chat_kind,
            title: message.chat_title.clone(),
            username: None,
            sync_state: TelegramSyncState::Synced,
            last_message_at: Some(message.occurred_at),
            metadata: json!({"runtime": runtime_kind}),
        };
        self.upsert_chat(&chat).await?;

        let mut payload = json!({
            "provider_chat_id": message.provider_chat_id,
            "chat_title": message.chat_title,
            "chat_kind": message.chat_kind.as_str(),
            "sender_id": message.sender_id,
            "sender_display_name": message.sender_display_name,
            "text": message.text,
            "delivery_state": message.delivery_state.as_str(),
        });
        if let (Some(payload), Some(tdlib_raw)) = (payload.as_object_mut(), tdlib_raw) {
            payload.insert("tdlib_raw".to_owned(), tdlib_raw);
        }
        let raw_record_id = telegram_raw_record_id(
            &message.account_id,
            TELEGRAM_MESSAGE_RECORD_KIND,
            &message.provider_message_id,
        );
        let raw = NewRawCommunicationRecord::new(
            &raw_record_id,
            &message.account_id,
            TELEGRAM_MESSAGE_RECORD_KIND,
            &message.provider_message_id,
            message.source_fingerprint(),
            &message.import_batch_id,
            payload,
        )
        .occurred_at(message.occurred_at)
        .provenance(json!({
            "provider": "telegram",
            "provider_kind": provider_account.provider_kind.as_str(),
            "runtime": runtime_kind,
            "account_id": message.account_id,
            "provider_chat_id": message.provider_chat_id,
        }));
        let raw = communication_store.record_raw_source(&raw).await?;
        let projected =
            project_raw_telegram_message(&MessageProjectionStore::new(self.pool.clone()), &raw)
                .await?;
        self.refresh_message_intelligence_candidates(&projected.message_id)
            .await?;

        Ok(TelegramMessageIngestResult {
            raw_record_id: raw.raw_record_id,
            message_id: projected.message_id,
        })
    }

    async fn refresh_message_intelligence_candidates(
        &self,
        message_id: &str,
    ) -> Result<(), TelegramError> {
        let message_ids = vec![message_id.to_owned()];
        DecisionStore::new(self.pool.clone())
            .refresh_message_candidates_for_ids(&message_ids)
            .await?;
        TaskCandidateStore::new(self.pool.clone())
            .refresh_message_candidates_for_ids(&message_ids)
            .await?;
        Ok(())
    }

    pub(super) async fn telegram_provider_account(
        &self,
        communication_store: &CommunicationIngestionStore,
        account_id: &str,
    ) -> Result<ProviderAccount, TelegramError> {
        let provider_account = communication_store
            .provider_account(account_id)
            .await?
            .ok_or_else(|| {
                TelegramError::InvalidRequest(format!(
                    "Telegram account `{account_id}` is not configured"
                ))
            })?;
        if !provider_account.provider_kind.is_telegram() {
            return Err(TelegramError::InvalidRequest(format!(
                "account `{}` is not a Telegram provider account",
                provider_account.account_id
            )));
        }
        Ok(provider_account)
    }

    pub async fn manual_send_message(
        &self,
        request: &TelegramManualSendRequest,
    ) -> Result<TelegramManualSendResponse, TelegramError> {
        request.validate()?;
        let communication_store = CommunicationIngestionStore::new(self.pool.clone());
        let provider_account = communication_store
            .provider_account(&request.account_id)
            .await?
            .ok_or_else(|| {
                TelegramError::InvalidRequest(format!(
                    "Telegram account `{}` is not configured",
                    request.account_id
                ))
            })?;
        if !provider_account.provider_kind.is_telegram() {
            return Err(TelegramError::InvalidRequest(format!(
                "account `{}` is not a Telegram provider account",
                request.account_id
            )));
        }

        let runtime_kind = telegram_account_runtime(&provider_account);
        if runtime_kind != "fixture" {
            return Err(TelegramError::InvalidRequest(
                "manual live Telegram sends require an enabled TDLib actor".to_owned(),
            ));
        }

        let chat = self
            .telegram_chat(&request.account_id, &request.provider_chat_id)
            .await?
            .ok_or_else(|| {
                TelegramError::InvalidRequest(format!(
                    "Telegram chat `{}` is not synced for account `{}`",
                    request.provider_chat_id, request.account_id
                ))
            })?;
        let provider_message_id = format!("manual:{}", request.command_id.trim());
        let rendered_preview_hash = telegram_text_preview_hash(&request.text);
        let message = NewTelegramMessage {
            account_id: request.account_id.trim().to_owned(),
            provider_chat_id: request.provider_chat_id.trim().to_owned(),
            provider_message_id,
            chat_kind: TelegramChatKind::try_from(chat.chat_kind.as_str())?,
            chat_title: chat.title,
            sender_id: "hermes".to_owned(),
            sender_display_name: "Hermes".to_owned(),
            text: request.text.trim().to_owned(),
            import_batch_id: format!("telegram-manual-send:{}", request.command_id.trim()),
            occurred_at: Utc::now(),
            delivery_state: TelegramDeliveryState::Sent,
        };
        let result = self.ingest_fixture_message(&message).await?;

        Ok(TelegramManualSendResponse {
            raw_record_id: result.raw_record_id,
            message_id: result.message_id,
            account_id: request.account_id.trim().to_owned(),
            provider_chat_id: request.provider_chat_id.trim().to_owned(),
            delivery_state: TelegramDeliveryState::Sent.as_str().to_owned(),
            status: "sent".to_owned(),
            runtime_kind,
            rendered_preview_hash,
        })
    }

    pub async fn recent_messages(
        &self,
        account_id: Option<&str>,
        provider_chat_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<TelegramMessage>, TelegramError> {
        let limit = validate_message_list_limit(limit)?;
        let account_id = account_id.map(str::trim).filter(|value| !value.is_empty());
        let provider_chat_id = provider_chat_id
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let rows = sqlx::query(
            r#"
            SELECT
                message_id,
                raw_record_id,
                account_id,
                provider_record_id,
                subject,
                sender,
                body_text,
                occurred_at,
                projected_at,
                channel_kind,
                conversation_id,
                sender_display_name,
                delivery_state,
                message_metadata
            FROM communication_messages
            WHERE channel_kind IN ('telegram_user', 'telegram_bot')
              AND ($1::text IS NULL OR account_id = $1)
              AND ($2::text IS NULL OR conversation_id = $2)
            ORDER BY COALESCE(occurred_at, projected_at) DESC, message_id ASC
            LIMIT $3
            "#,
        )
        .bind(account_id)
        .bind(provider_chat_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_telegram_message).collect()
    }

    pub(crate) async fn attachment_anchor_for_message(
        &self,
        account_id: &str,
        provider_chat_id: &str,
        provider_message_id: &str,
    ) -> Result<TelegramAttachmentAnchor, TelegramError> {
        let row = sqlx::query(
            r#"
            SELECT message_id, raw_record_id
            FROM communication_messages
            WHERE account_id = $1
              AND conversation_id = $2
              AND provider_record_id = $3
              AND channel_kind IN ('telegram_user', 'telegram_bot')
            ORDER BY COALESCE(occurred_at, projected_at) DESC, message_id ASC
            LIMIT 1
            "#,
        )
        .bind(account_id.trim())
        .bind(provider_chat_id.trim())
        .bind(provider_message_id.trim())
        .fetch_optional(&self.pool)
        .await?;

        let row = row.ok_or_else(|| {
            TelegramError::InvalidRequest(format!(
                "Telegram message `{}` is not projected for chat `{}` and account `{}`",
                provider_message_id.trim(),
                provider_chat_id.trim(),
                account_id.trim()
            ))
        })?;

        Ok(TelegramAttachmentAnchor {
            message_id: row.try_get("message_id")?,
            raw_record_id: row.try_get("raw_record_id")?,
        })
    }

    pub(crate) async fn telegram_chat(
        &self,
        account_id: &str,
        provider_chat_id: &str,
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
            WHERE account_id = $1 AND provider_chat_id = $2
            "#,
        )
        .bind(account_id.trim())
        .bind(provider_chat_id.trim())
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_telegram_chat).transpose()
    }
}
