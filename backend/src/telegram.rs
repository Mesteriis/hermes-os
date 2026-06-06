use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use thiserror::Error;

use crate::communications::{
    CommunicationIngestionError, CommunicationIngestionStore, CommunicationProviderKind,
    NewProviderAccount, NewRawCommunicationRecord,
};
use crate::messages::{MessageProjectionError, MessageProjectionStore, NewProjectedMessage};

const TELEGRAM_MESSAGE_RECORD_KIND: &str = "telegram_message";

#[derive(Clone)]
pub struct TelegramStore {
    pool: PgPool,
}

impl TelegramStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn setup_fixture_account(
        &self,
        request: &TelegramAccountSetupRequest,
    ) -> Result<TelegramAccountSetupResponse, TelegramError> {
        request.validate()?;
        let provider_kind = request.provider_kind;
        if !provider_kind.is_telegram() {
            return Err(TelegramError::InvalidRequest(
                "provider_kind must be telegram_user or telegram_bot".to_owned(),
            ));
        }

        let account = NewProviderAccount::new(
            &request.account_id,
            provider_kind,
            &request.display_name,
            &request.external_account_id,
        )
        .config(json!({
            "runtime": "fixture",
            "tdlib_data_path": request.tdlib_data_path,
            "transcription_enabled": request.transcription_enabled,
        }));
        let stored_account = CommunicationIngestionStore::new(self.pool.clone())
            .upsert_provider_account(&account)
            .await?;

        Ok(TelegramAccountSetupResponse {
            account_id: stored_account.account_id,
            provider_kind: stored_account.provider_kind.as_str().to_owned(),
            runtime: "fixture".to_owned(),
            transcription_enabled: request.transcription_enabled,
        })
    }

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
        let limit = validate_limit(limit)?;
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

    pub async fn ingest_fixture_message(
        &self,
        message: &NewTelegramMessage,
    ) -> Result<TelegramMessageIngestResult, TelegramError> {
        message.validate()?;
        let communication_store = CommunicationIngestionStore::new(self.pool.clone());
        let provider_account = communication_store
            .provider_account(&message.account_id)
            .await?
            .ok_or_else(|| {
                TelegramError::InvalidRequest(format!(
                    "Telegram account `{}` is not configured",
                    message.account_id
                ))
            })?;
        if !provider_account.provider_kind.is_telegram() {
            return Err(TelegramError::InvalidRequest(format!(
                "account `{}` is not a Telegram provider account",
                message.account_id
            )));
        }

        let chat = NewTelegramChat {
            account_id: message.account_id.clone(),
            provider_chat_id: message.provider_chat_id.clone(),
            chat_kind: message.chat_kind,
            title: message.chat_title.clone(),
            username: None,
            sync_state: TelegramSyncState::Synced,
            last_message_at: Some(message.occurred_at),
            metadata: json!({"runtime": "fixture"}),
        };
        self.upsert_chat(&chat).await?;

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
            json!({
                "provider_chat_id": message.provider_chat_id,
                "chat_title": message.chat_title,
                "chat_kind": message.chat_kind.as_str(),
                "sender_id": message.sender_id,
                "sender_display_name": message.sender_display_name,
                "text": message.text,
                "delivery_state": message.delivery_state.as_str(),
            }),
        )
        .occurred_at(message.occurred_at)
        .provenance(json!({
            "provider": "telegram",
            "provider_kind": provider_account.provider_kind.as_str(),
            "runtime": "fixture",
            "account_id": message.account_id,
            "provider_chat_id": message.provider_chat_id,
        }));
        let raw = communication_store.record_raw_source(&raw).await?;
        let projected =
            project_raw_telegram_message(&MessageProjectionStore::new(self.pool.clone()), &raw)
                .await?;

        Ok(TelegramMessageIngestResult {
            raw_record_id: raw.raw_record_id,
            message_id: projected.message_id,
        })
    }

    pub async fn recent_messages(
        &self,
        account_id: Option<&str>,
        provider_chat_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<TelegramMessage>, TelegramError> {
        let limit = validate_limit(limit)?;
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
}

pub async fn project_raw_telegram_message(
    store: &MessageProjectionStore,
    raw: &crate::communications::StoredRawCommunicationRecord,
) -> Result<crate::messages::ProjectedMessage, TelegramError> {
    if raw.record_kind != TELEGRAM_MESSAGE_RECORD_KIND {
        return Err(TelegramError::InvalidRequest(
            "raw record kind must be telegram_message".to_owned(),
        ));
    }

    let provider_chat_id = required_payload_string(&raw.payload, "provider_chat_id")?;
    let chat_title = required_payload_string(&raw.payload, "chat_title")?;
    let sender_display_name = required_payload_string(&raw.payload, "sender_display_name")?;
    let text = required_payload_string(&raw.payload, "text")?;
    let delivery_state =
        TelegramDeliveryState::try_from(required_payload_string(&raw.payload, "delivery_state")?)?;
    let channel_kind = raw
        .provenance
        .get("provider_kind")
        .and_then(Value::as_str)
        .unwrap_or("telegram_user");

    Ok(store
        .upsert_channel_message(&NewProjectedMessage {
            message_id: telegram_message_id(&raw.account_id, &raw.provider_record_id),
            raw_record_id: raw.raw_record_id.clone(),
            account_id: raw.account_id.clone(),
            provider_record_id: raw.provider_record_id.clone(),
            subject: chat_title,
            sender: sender_display_name.clone(),
            recipients: vec![provider_chat_id.clone()],
            body_text: text,
            occurred_at: raw.occurred_at,
            channel_kind: channel_kind.to_owned(),
            conversation_id: Some(provider_chat_id),
            sender_display_name: Some(sender_display_name),
            delivery_state: delivery_state.as_message_delivery_state().to_owned(),
            message_metadata: raw.payload.clone(),
        })
        .await?)
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct TelegramAccountSetupRequest {
    pub account_id: String,
    pub provider_kind: CommunicationProviderKind,
    pub display_name: String,
    pub external_account_id: String,
    pub tdlib_data_path: Option<String>,
    #[serde(default)]
    pub transcription_enabled: bool,
}

impl TelegramAccountSetupRequest {
    fn validate(&self) -> Result<(), TelegramError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("display_name", &self.display_name)?;
        validate_non_empty("external_account_id", &self.external_account_id)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramAccountSetupResponse {
    pub account_id: String,
    pub provider_kind: String,
    pub runtime: String,
    pub transcription_enabled: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramChat {
    pub telegram_chat_id: String,
    pub account_id: String,
    pub provider_chat_id: String,
    pub chat_kind: String,
    pub title: String,
    pub username: Option<String>,
    pub sync_state: String,
    pub last_message_at: Option<DateTime<Utc>>,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewTelegramChat {
    pub account_id: String,
    pub provider_chat_id: String,
    pub chat_kind: TelegramChatKind,
    pub title: String,
    pub username: Option<String>,
    pub sync_state: TelegramSyncState,
    pub last_message_at: Option<DateTime<Utc>>,
    pub metadata: Value,
}

impl NewTelegramChat {
    fn validate(&self) -> Result<(), TelegramError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("provider_chat_id", &self.provider_chat_id)?;
        validate_non_empty("title", &self.title)?;
        validate_object("metadata", &self.metadata)?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TelegramChatKind {
    Private,
    Group,
    Channel,
    Bot,
}

impl TelegramChatKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Private => "private",
            Self::Group => "group",
            Self::Channel => "channel",
            Self::Bot => "bot",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TelegramSyncState {
    Fixture,
    Syncing,
    Synced,
    Degraded,
    Error,
}

impl TelegramSyncState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Fixture => "fixture",
            Self::Syncing => "syncing",
            Self::Synced => "synced",
            Self::Degraded => "degraded",
            Self::Error => "error",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct NewTelegramMessage {
    pub account_id: String,
    pub provider_chat_id: String,
    pub provider_message_id: String,
    pub chat_kind: TelegramChatKind,
    pub chat_title: String,
    pub sender_id: String,
    pub sender_display_name: String,
    pub text: String,
    pub import_batch_id: String,
    pub occurred_at: DateTime<Utc>,
    pub delivery_state: TelegramDeliveryState,
}

impl NewTelegramMessage {
    fn validate(&self) -> Result<(), TelegramError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("provider_chat_id", &self.provider_chat_id)?;
        validate_non_empty("provider_message_id", &self.provider_message_id)?;
        validate_non_empty("chat_title", &self.chat_title)?;
        validate_non_empty("sender_id", &self.sender_id)?;
        validate_non_empty("sender_display_name", &self.sender_display_name)?;
        validate_non_empty("text", &self.text)?;
        validate_non_empty("import_batch_id", &self.import_batch_id)?;
        Ok(())
    }

    fn source_fingerprint(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.account_id.as_bytes());
        hasher.update(b"\0");
        hasher.update(self.provider_chat_id.as_bytes());
        hasher.update(b"\0");
        hasher.update(self.provider_message_id.as_bytes());
        format!("sha256:{:x}", hasher.finalize())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TelegramDeliveryState {
    Received,
    Sent,
    SendDryRun,
    SendBlocked,
}

impl TelegramDeliveryState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Received => "received",
            Self::Sent => "sent",
            Self::SendDryRun => "send_dry_run",
            Self::SendBlocked => "send_blocked",
        }
    }

    pub fn as_message_delivery_state(self) -> &'static str {
        self.as_str()
    }
}

impl TryFrom<String> for TelegramDeliveryState {
    type Error = TelegramError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "received" => Ok(Self::Received),
            "sent" => Ok(Self::Sent),
            "send_dry_run" => Ok(Self::SendDryRun),
            "send_blocked" => Ok(Self::SendBlocked),
            _ => Err(TelegramError::InvalidRequest(format!(
                "unsupported Telegram delivery_state `{value}`"
            ))),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramMessageIngestResult {
    pub raw_record_id: String,
    pub message_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramMessage {
    pub message_id: String,
    pub raw_record_id: String,
    pub account_id: String,
    pub provider_message_id: String,
    pub provider_chat_id: Option<String>,
    pub chat_title: String,
    pub sender: String,
    pub sender_display_name: Option<String>,
    pub text: String,
    pub occurred_at: Option<DateTime<Utc>>,
    pub projected_at: DateTime<Utc>,
    pub channel_kind: String,
    pub delivery_state: String,
    pub metadata: Value,
}

#[derive(Debug, Error)]
pub enum TelegramError {
    #[error("invalid Telegram request: {0}")]
    InvalidRequest(String),

    #[error(transparent)]
    Communication(#[from] CommunicationIngestionError),

    #[error(transparent)]
    MessageProjection(#[from] MessageProjectionError),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

fn row_to_telegram_chat(row: PgRow) -> Result<TelegramChat, TelegramError> {
    Ok(TelegramChat {
        telegram_chat_id: row.try_get("telegram_chat_id")?,
        account_id: row.try_get("account_id")?,
        provider_chat_id: row.try_get("provider_chat_id")?,
        chat_kind: row.try_get("chat_kind")?,
        title: row.try_get("title")?,
        username: row.try_get("username")?,
        sync_state: row.try_get("sync_state")?,
        last_message_at: row.try_get("last_message_at")?,
        metadata: row.try_get("metadata")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn row_to_telegram_message(row: PgRow) -> Result<TelegramMessage, TelegramError> {
    Ok(TelegramMessage {
        message_id: row.try_get("message_id")?,
        raw_record_id: row.try_get("raw_record_id")?,
        account_id: row.try_get("account_id")?,
        provider_message_id: row.try_get("provider_record_id")?,
        provider_chat_id: row.try_get("conversation_id")?,
        chat_title: row.try_get("subject")?,
        sender: row.try_get("sender")?,
        sender_display_name: row.try_get("sender_display_name")?,
        text: row.try_get("body_text")?,
        occurred_at: row.try_get("occurred_at")?,
        projected_at: row.try_get("projected_at")?,
        channel_kind: row.try_get("channel_kind")?,
        delivery_state: row.try_get("delivery_state")?,
        metadata: row.try_get("message_metadata")?,
    })
}

fn required_payload_string(payload: &Value, field: &'static str) -> Result<String, TelegramError> {
    payload
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| {
            TelegramError::InvalidRequest(format!("payload field `{field}` is required"))
        })
}

fn telegram_chat_id(account_id: &str, provider_chat_id: &str) -> String {
    format!(
        "telegram_chat:v4:{}",
        stable_hash([account_id, provider_chat_id].join("\0").as_bytes())
    )
}

fn telegram_message_id(account_id: &str, provider_message_id: &str) -> String {
    format!(
        "message:v4:telegram:{}",
        stable_hash([account_id, provider_message_id].join("\0").as_bytes())
    )
}

fn telegram_raw_record_id(account_id: &str, record_kind: &str, provider_record_id: &str) -> String {
    format!(
        "raw:v4:telegram:{}",
        stable_hash(
            [account_id, record_kind, provider_record_id]
                .join("\0")
                .as_bytes()
        )
    )
}

fn stable_hash(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn validate_limit(limit: i64) -> Result<i64, TelegramError> {
    if !(1..=100).contains(&limit) {
        return Err(TelegramError::InvalidRequest(
            "limit must be between 1 and 100".to_owned(),
        ));
    }
    Ok(limit)
}

fn validate_non_empty(field: &'static str, value: &str) -> Result<String, TelegramError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(TelegramError::InvalidRequest(format!(
            "{field} must not be empty"
        )));
    }
    Ok(trimmed.to_owned())
}

fn validate_object(field: &'static str, value: &Value) -> Result<(), TelegramError> {
    if !value.is_object() {
        return Err(TelegramError::InvalidRequest(format!(
            "{field} must be a JSON object"
        )));
    }
    Ok(())
}
