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

const WHATSAPP_WEB_MESSAGE_RECORD_KIND: &str = "whatsapp_web_message";

#[derive(Clone)]
pub struct WhatsappWebStore {
    pool: PgPool,
}

impl WhatsappWebStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn setup_fixture_account(
        &self,
        request: &WhatsappWebAccountSetupRequest,
    ) -> Result<WhatsappWebAccountSetupResponse, WhatsappWebError> {
        request.validate()?;
        if request.provider_kind != CommunicationProviderKind::WhatsappWeb {
            return Err(WhatsappWebError::InvalidRequest(
                "provider_kind must be whatsapp_web".to_owned(),
            ));
        }

        let account = NewProviderAccount::new(
            &request.account_id,
            request.provider_kind,
            &request.display_name,
            &request.external_account_id,
        )
        .config(json!({
            "runtime": "fixture",
            "local_state_path": request.local_state_path,
            "device_name": request.device_name,
        }));
        let stored_account = CommunicationIngestionStore::new(self.pool.clone())
            .upsert_provider_account(&account)
            .await?;

        let session = self
            .upsert_session(&NewWhatsappWebSession {
                session_id: whatsapp_web_session_id(&request.account_id),
                account_id: stored_account.account_id.clone(),
                device_name: request.device_name.clone(),
                companion_runtime: WhatsappWebCompanionRuntime::Fixture,
                link_state: WhatsappWebLinkState::Fixture,
                local_state_path: request.local_state_path.clone(),
                last_sync_at: None,
                metadata: json!({"runtime": "fixture"}),
            })
            .await?;

        Ok(WhatsappWebAccountSetupResponse {
            account_id: stored_account.account_id,
            provider_kind: stored_account.provider_kind.as_str().to_owned(),
            runtime: "fixture".to_owned(),
            session,
        })
    }

    pub async fn upsert_session(
        &self,
        session: &NewWhatsappWebSession,
    ) -> Result<WhatsappWebSession, WhatsappWebError> {
        session.validate()?;
        let row = sqlx::query(
            r#"
            INSERT INTO whatsapp_web_sessions (
                session_id,
                account_id,
                device_name,
                companion_runtime,
                link_state,
                local_state_path,
                last_sync_at,
                metadata,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, now())
            ON CONFLICT (account_id)
            DO UPDATE SET
                device_name = EXCLUDED.device_name,
                companion_runtime = EXCLUDED.companion_runtime,
                link_state = EXCLUDED.link_state,
                local_state_path = EXCLUDED.local_state_path,
                last_sync_at = EXCLUDED.last_sync_at,
                metadata = EXCLUDED.metadata,
                updated_at = now()
            RETURNING
                session_id,
                account_id,
                device_name,
                companion_runtime,
                link_state,
                local_state_path,
                last_sync_at,
                metadata,
                created_at,
                updated_at
            "#,
        )
        .bind(session.session_id.trim())
        .bind(session.account_id.trim())
        .bind(session.device_name.trim())
        .bind(session.companion_runtime.as_str())
        .bind(session.link_state.as_str())
        .bind(session.local_state_path.trim())
        .bind(session.last_sync_at)
        .bind(&session.metadata)
        .fetch_one(&self.pool)
        .await?;

        row_to_whatsapp_web_session(row)
    }

    pub async fn list_sessions(
        &self,
        account_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<WhatsappWebSession>, WhatsappWebError> {
        let limit = validate_limit(limit)?;
        let account_id = account_id.map(str::trim).filter(|value| !value.is_empty());
        let rows = sqlx::query(
            r#"
            SELECT
                session_id,
                account_id,
                device_name,
                companion_runtime,
                link_state,
                local_state_path,
                last_sync_at,
                metadata,
                created_at,
                updated_at
            FROM whatsapp_web_sessions
            WHERE ($1::text IS NULL OR account_id = $1)
            ORDER BY updated_at DESC, session_id ASC
            LIMIT $2
            "#,
        )
        .bind(account_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_whatsapp_web_session).collect()
    }

    pub async fn ingest_fixture_message(
        &self,
        message: &NewWhatsappWebMessage,
    ) -> Result<WhatsappWebMessageIngestResult, WhatsappWebError> {
        message.validate()?;
        let communication_store = CommunicationIngestionStore::new(self.pool.clone());
        let provider_account = communication_store
            .provider_account(&message.account_id)
            .await?
            .ok_or_else(|| {
                WhatsappWebError::InvalidRequest(format!(
                    "WhatsApp Web account `{}` is not configured",
                    message.account_id
                ))
            })?;
        if !provider_account.provider_kind.is_whatsapp() {
            return Err(WhatsappWebError::InvalidRequest(format!(
                "account `{}` is not a WhatsApp Web provider account",
                message.account_id
            )));
        }

        let session = self
            .list_sessions(Some(&message.account_id), 1)
            .await?
            .into_iter()
            .next()
            .ok_or_else(|| {
                WhatsappWebError::InvalidRequest(format!(
                    "WhatsApp Web account `{}` has no session metadata",
                    message.account_id
                ))
            })?;
        if session.link_state == WhatsappWebLinkState::Blocked.as_str() {
            return Err(WhatsappWebError::InvalidRequest(
                "blocked WhatsApp Web sessions cannot ingest fixture messages".to_owned(),
            ));
        }

        let raw_record_id = whatsapp_web_raw_record_id(
            &message.account_id,
            WHATSAPP_WEB_MESSAGE_RECORD_KIND,
            &message.provider_message_id,
        );
        let raw = NewRawCommunicationRecord::new(
            &raw_record_id,
            &message.account_id,
            WHATSAPP_WEB_MESSAGE_RECORD_KIND,
            &message.provider_message_id,
            message.source_fingerprint(),
            &message.import_batch_id,
            json!({
                "provider_chat_id": message.provider_chat_id,
                "chat_title": message.chat_title,
                "sender_id": message.sender_id,
                "sender_display_name": message.sender_display_name,
                "text": message.text,
                "delivery_state": message.delivery_state.as_str(),
            }),
        )
        .occurred_at(message.occurred_at)
        .provenance(json!({
            "provider": "whatsapp_web",
            "provider_kind": provider_account.provider_kind.as_str(),
            "runtime": session.companion_runtime,
            "account_id": message.account_id,
            "provider_chat_id": message.provider_chat_id,
        }));
        let raw = communication_store.record_raw_source(&raw).await?;
        let projected =
            project_raw_whatsapp_web_message(&MessageProjectionStore::new(self.pool.clone()), &raw)
                .await?;

        self.update_session_last_sync(&message.account_id, message.occurred_at)
            .await?;

        Ok(WhatsappWebMessageIngestResult {
            raw_record_id: raw.raw_record_id,
            message_id: projected.message_id,
        })
    }

    pub async fn recent_messages(
        &self,
        account_id: Option<&str>,
        provider_chat_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<WhatsappWebMessage>, WhatsappWebError> {
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
            WHERE channel_kind = 'whatsapp_web'
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

        rows.into_iter().map(row_to_whatsapp_web_message).collect()
    }

    async fn update_session_last_sync(
        &self,
        account_id: &str,
        last_sync_at: DateTime<Utc>,
    ) -> Result<(), WhatsappWebError> {
        sqlx::query(
            r#"
            UPDATE whatsapp_web_sessions
            SET last_sync_at = GREATEST(COALESCE(last_sync_at, $2), $2),
                updated_at = now()
            WHERE account_id = $1
            "#,
        )
        .bind(account_id.trim())
        .bind(last_sync_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

pub async fn project_raw_whatsapp_web_message(
    store: &MessageProjectionStore,
    raw: &crate::communications::StoredRawCommunicationRecord,
) -> Result<crate::messages::ProjectedMessage, WhatsappWebError> {
    if raw.record_kind != WHATSAPP_WEB_MESSAGE_RECORD_KIND {
        return Err(WhatsappWebError::InvalidRequest(
            "raw record kind must be whatsapp_web_message".to_owned(),
        ));
    }

    let provider_chat_id = required_payload_string(&raw.payload, "provider_chat_id")?;
    let chat_title = required_payload_string(&raw.payload, "chat_title")?;
    let sender_display_name = required_payload_string(&raw.payload, "sender_display_name")?;
    let text = required_payload_string(&raw.payload, "text")?;
    let delivery_state = WhatsappWebDeliveryState::try_from(required_payload_string(
        &raw.payload,
        "delivery_state",
    )?)?;

    Ok(store
        .upsert_channel_message(&NewProjectedMessage {
            message_id: whatsapp_web_message_id(&raw.account_id, &raw.provider_record_id),
            raw_record_id: raw.raw_record_id.clone(),
            account_id: raw.account_id.clone(),
            provider_record_id: raw.provider_record_id.clone(),
            subject: chat_title,
            sender: sender_display_name.clone(),
            recipients: vec![provider_chat_id.clone()],
            body_text: text,
            occurred_at: raw.occurred_at,
            channel_kind: "whatsapp_web".to_owned(),
            conversation_id: Some(provider_chat_id),
            sender_display_name: Some(sender_display_name),
            delivery_state: delivery_state.as_message_delivery_state().to_owned(),
            message_metadata: raw.payload.clone(),
        })
        .await?)
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct WhatsappWebAccountSetupRequest {
    pub account_id: String,
    pub provider_kind: CommunicationProviderKind,
    pub display_name: String,
    pub external_account_id: String,
    pub device_name: String,
    pub local_state_path: String,
}

impl WhatsappWebAccountSetupRequest {
    fn validate(&self) -> Result<(), WhatsappWebError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("display_name", &self.display_name)?;
        validate_non_empty("external_account_id", &self.external_account_id)?;
        validate_non_empty("device_name", &self.device_name)?;
        validate_non_empty("local_state_path", &self.local_state_path)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct WhatsappWebAccountSetupResponse {
    pub account_id: String,
    pub provider_kind: String,
    pub runtime: String,
    pub session: WhatsappWebSession,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewWhatsappWebSession {
    pub session_id: String,
    pub account_id: String,
    pub device_name: String,
    pub companion_runtime: WhatsappWebCompanionRuntime,
    pub link_state: WhatsappWebLinkState,
    pub local_state_path: String,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub metadata: Value,
}

impl NewWhatsappWebSession {
    fn validate(&self) -> Result<(), WhatsappWebError> {
        validate_non_empty("session_id", &self.session_id)?;
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("device_name", &self.device_name)?;
        validate_non_empty("local_state_path", &self.local_state_path)?;
        validate_object("metadata", &self.metadata)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct WhatsappWebSession {
    pub session_id: String,
    pub account_id: String,
    pub device_name: String,
    pub companion_runtime: String,
    pub link_state: String,
    pub local_state_path: String,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WhatsappWebCompanionRuntime {
    Fixture,
    ManualWebview,
    Blocked,
}

impl WhatsappWebCompanionRuntime {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Fixture => "fixture",
            Self::ManualWebview => "manual_webview",
            Self::Blocked => "blocked",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WhatsappWebLinkState {
    Fixture,
    QrPending,
    Linked,
    Degraded,
    Revoked,
    Blocked,
}

impl WhatsappWebLinkState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Fixture => "fixture",
            Self::QrPending => "qr_pending",
            Self::Linked => "linked",
            Self::Degraded => "degraded",
            Self::Revoked => "revoked",
            Self::Blocked => "blocked",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct NewWhatsappWebMessage {
    pub account_id: String,
    pub provider_chat_id: String,
    pub provider_message_id: String,
    pub chat_title: String,
    pub sender_id: String,
    pub sender_display_name: String,
    pub text: String,
    pub import_batch_id: String,
    pub occurred_at: DateTime<Utc>,
    pub delivery_state: WhatsappWebDeliveryState,
}

impl NewWhatsappWebMessage {
    fn validate(&self) -> Result<(), WhatsappWebError> {
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
pub enum WhatsappWebDeliveryState {
    Received,
    Sent,
    SendDryRun,
    SendBlocked,
}

impl WhatsappWebDeliveryState {
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

impl TryFrom<String> for WhatsappWebDeliveryState {
    type Error = WhatsappWebError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "received" => Ok(Self::Received),
            "sent" => Ok(Self::Sent),
            "send_dry_run" => Ok(Self::SendDryRun),
            "send_blocked" => Ok(Self::SendBlocked),
            _ => Err(WhatsappWebError::InvalidRequest(format!(
                "unsupported WhatsApp Web delivery_state `{value}`"
            ))),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct WhatsappWebMessageIngestResult {
    pub raw_record_id: String,
    pub message_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct WhatsappWebMessage {
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
pub enum WhatsappWebError {
    #[error("invalid WhatsApp Web request: {0}")]
    InvalidRequest(String),

    #[error(transparent)]
    Communication(#[from] CommunicationIngestionError),

    #[error(transparent)]
    MessageProjection(#[from] MessageProjectionError),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

fn row_to_whatsapp_web_session(row: PgRow) -> Result<WhatsappWebSession, WhatsappWebError> {
    Ok(WhatsappWebSession {
        session_id: row.try_get("session_id")?,
        account_id: row.try_get("account_id")?,
        device_name: row.try_get("device_name")?,
        companion_runtime: row.try_get("companion_runtime")?,
        link_state: row.try_get("link_state")?,
        local_state_path: row.try_get("local_state_path")?,
        last_sync_at: row.try_get("last_sync_at")?,
        metadata: row.try_get("metadata")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn row_to_whatsapp_web_message(row: PgRow) -> Result<WhatsappWebMessage, WhatsappWebError> {
    Ok(WhatsappWebMessage {
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

fn required_payload_string(
    payload: &Value,
    field: &'static str,
) -> Result<String, WhatsappWebError> {
    payload
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| {
            WhatsappWebError::InvalidRequest(format!("payload field `{field}` is required"))
        })
}

fn whatsapp_web_session_id(account_id: &str) -> String {
    format!(
        "whatsapp_web_session:v5:{}",
        stable_hash(account_id.as_bytes())
    )
}

fn whatsapp_web_message_id(account_id: &str, provider_message_id: &str) -> String {
    format!(
        "message:v5:whatsapp_web:{}",
        stable_hash([account_id, provider_message_id].join("\0").as_bytes())
    )
}

fn whatsapp_web_raw_record_id(
    account_id: &str,
    record_kind: &str,
    provider_record_id: &str,
) -> String {
    format!(
        "raw:v5:whatsapp_web:{}",
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

fn validate_limit(limit: i64) -> Result<i64, WhatsappWebError> {
    if !(1..=100).contains(&limit) {
        return Err(WhatsappWebError::InvalidRequest(
            "limit must be between 1 and 100".to_owned(),
        ));
    }
    Ok(limit)
}

fn validate_non_empty(field: &'static str, value: &str) -> Result<String, WhatsappWebError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(WhatsappWebError::InvalidRequest(format!(
            "{field} must not be empty"
        )));
    }
    Ok(trimmed.to_owned())
}

fn validate_object(field: &'static str, value: &Value) -> Result<(), WhatsappWebError> {
    if !value.is_object() {
        return Err(WhatsappWebError::InvalidRequest(format!(
            "{field} must be a JSON object"
        )));
    }
    Ok(())
}
