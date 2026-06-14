use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::postgres::PgPool;

use crate::domains::decisions::DecisionStore;
use crate::domains::mail::core::{
    CommunicationIngestionStore, CommunicationProviderKind, NewProviderAccount,
    NewRawCommunicationRecord,
};
use crate::domains::mail::messages::MessageProjectionStore;
use crate::domains::tasks::candidates::TaskCandidateStore;

use super::constants::WHATSAPP_WEB_MESSAGE_RECORD_KIND;
use super::errors::WhatsappWebError;
use super::ids::{whatsapp_web_raw_record_id, whatsapp_web_session_id};
use super::models::{
    NewWhatsappWebMessage, NewWhatsappWebSession, WhatsappWebAccountSetupRequest,
    WhatsappWebAccountSetupResponse, WhatsappWebCompanionRuntime, WhatsappWebLinkState,
    WhatsappWebMessage, WhatsappWebMessageIngestResult, WhatsappWebSession,
};
use super::projection::project_raw_whatsapp_web_message;
use super::rows::{row_to_whatsapp_web_message, row_to_whatsapp_web_session};
use super::validation::validate_limit;

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
        self.refresh_message_intelligence_candidates(&projected.message_id)
            .await?;

        self.update_session_last_sync(&message.account_id, message.occurred_at)
            .await?;

        Ok(WhatsappWebMessageIngestResult {
            raw_record_id: raw.raw_record_id,
            message_id: projected.message_id,
        })
    }

    async fn refresh_message_intelligence_candidates(
        &self,
        message_id: &str,
    ) -> Result<(), WhatsappWebError> {
        let message_ids = vec![message_id.to_owned()];
        DecisionStore::new(self.pool.clone())
            .refresh_message_candidates_for_ids(&message_ids)
            .await?;
        TaskCandidateStore::new(self.pool.clone())
            .refresh_message_candidates_for_ids(&message_ids)
            .await?;
        Ok(())
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
