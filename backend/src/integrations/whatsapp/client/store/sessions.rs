use chrono::{DateTime, Utc};

use super::WhatsappWebStore;
use crate::integrations::whatsapp::client::errors::WhatsappWebError;
use crate::integrations::whatsapp::client::models::{NewWhatsappWebSession, WhatsappWebSession};
use crate::integrations::whatsapp::client::rows::row_to_whatsapp_web_session;
use crate::integrations::whatsapp::client::validation::validate_limit;

impl WhatsappWebStore {
    pub async fn upsert_session(
        &self,
        session: &NewWhatsappWebSession,
    ) -> Result<WhatsappWebSession, WhatsappWebError> {
        session.validate()?;
        let row = sqlx::query(
            r#"
            INSERT INTO whatsapp_web_sessions (
                session_id, account_id, device_name, companion_runtime,
                link_state, local_state_path, last_sync_at, metadata, updated_at
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
                session_id, account_id, device_name, companion_runtime,
                link_state, local_state_path, last_sync_at, metadata,
                created_at, updated_at
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
                session_id, account_id, device_name, companion_runtime,
                link_state, local_state_path, last_sync_at, metadata,
                created_at, updated_at
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

    pub(in crate::integrations::whatsapp::client::store) async fn update_session_last_sync(
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
