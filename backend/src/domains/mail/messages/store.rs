use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use sqlx::{Postgres, Row, Transaction};

use super::errors::MessageProjectionError;
use super::ids::message_id;
use super::models::{
    NewProjectedMessage, ProjectedMessage, ProjectedMessageSummary, WorkflowStateCount,
};
use super::rows::{row_to_projected_message, row_to_projected_message_summary};
use super::states::{LocalMessageState, WorkflowState};
use super::validation::{validate_limit, validate_non_empty};

#[derive(Clone)]
pub struct MessageProjectionStore {
    pool: PgPool,
}

impl MessageProjectionStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn recent_messages(
        &self,
        limit: i64,
    ) -> Result<Vec<ProjectedMessageSummary>, MessageProjectionError> {
        let limit = validate_limit(limit)?;
        let rows = sqlx::query(
            r#"
            SELECT
                m.message_id,
                m.raw_record_id,
                m.account_id,
                m.provider_record_id,
                m.subject,
                m.sender,
                m.recipients,
                m.body_text,
                m.occurred_at,
                m.projected_at,
                m.channel_kind,
                m.conversation_id,
                m.sender_display_name,
                m.delivery_state,
                m.message_metadata,
                m.workflow_state,
                m.importance_score,
                m.ai_category,
                m.ai_summary,
                m.ai_summary_generated_at,
                m.local_state,
                m.local_state_changed_at,
                m.local_state_reason,
                count(a.attachment_id)::BIGINT AS attachment_count
            FROM communication_messages m
            LEFT JOIN communication_attachments a ON a.message_id = m.message_id
            WHERE m.local_state = 'active'
            GROUP BY
                m.message_id,
                m.raw_record_id,
                m.account_id,
                m.provider_record_id,
                m.subject,
                m.sender,
                m.recipients,
                m.body_text,
                m.occurred_at,
                m.projected_at,
                m.channel_kind,
                m.conversation_id,
                m.sender_display_name,
                m.delivery_state,
                m.message_metadata,
                m.workflow_state,
                m.importance_score,
                m.ai_category,
                m.ai_summary,
                m.ai_summary_generated_at,
                m.local_state,
                m.local_state_changed_at,
                m.local_state_reason
            ORDER BY
                COALESCE(m.occurred_at, m.projected_at) DESC,
                m.projected_at DESC,
                m.message_id ASC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(row_to_projected_message_summary)
            .collect()
    }

    pub async fn message(
        &self,
        message_id: &str,
    ) -> Result<Option<ProjectedMessage>, MessageProjectionError> {
        validate_non_empty("message_id", message_id)?;

        let row = sqlx::query(
            r#"
            SELECT
                message_id,
                raw_record_id,
                account_id,
                provider_record_id,
                subject,
                sender,
                recipients,
                body_text,
                occurred_at,
                projected_at,
                channel_kind,
                conversation_id,
                sender_display_name,
                delivery_state,
                message_metadata,
                workflow_state,
                importance_score,
                ai_category,
                ai_summary,
                ai_summary_generated_at,
                local_state,
                local_state_changed_at,
                local_state_reason
            FROM communication_messages
            WHERE message_id = $1
            "#,
        )
        .bind(message_id.trim())
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_projected_message).transpose()
    }

    pub async fn upsert_message(
        &self,
        message: &NewProjectedMessage,
    ) -> Result<ProjectedMessage, MessageProjectionError> {
        message.validate()?;
        let canonical_message_id = message_id(&message.account_id, &message.provider_record_id);

        let row = sqlx::query(
            r#"
            INSERT INTO communication_messages (
                message_id,
                raw_record_id,
                account_id,
                provider_record_id,
                subject,
                sender,
                recipients,
                body_text,
                occurred_at,
                channel_kind,
                conversation_id,
                sender_display_name,
                delivery_state,
                message_metadata
            )
            SELECT
                $1,
                raw_record_id,
                account_id,
                provider_record_id,
                $5,
                $6,
                $7,
                $8,
                $9,
                'email',
                NULL,
                $6,
                'received',
                '{}'::jsonb
            FROM communication_raw_records
            WHERE raw_record_id = $2
              AND account_id = $3
              AND provider_record_id = $4
              AND record_kind = 'email_message'
            ON CONFLICT (account_id, provider_record_id)
            DO UPDATE SET
                message_id = EXCLUDED.message_id,
                raw_record_id = EXCLUDED.raw_record_id,
                subject = EXCLUDED.subject,
                sender = EXCLUDED.sender,
                recipients = EXCLUDED.recipients,
                body_text = EXCLUDED.body_text,
                occurred_at = EXCLUDED.occurred_at,
                channel_kind = EXCLUDED.channel_kind,
                conversation_id = EXCLUDED.conversation_id,
                sender_display_name = EXCLUDED.sender_display_name,
                delivery_state = EXCLUDED.delivery_state,
                message_metadata = EXCLUDED.message_metadata,
                projected_at = now()
            RETURNING
                message_id,
                raw_record_id,
                account_id,
                provider_record_id,
                subject,
                sender,
                recipients,
                body_text,
                occurred_at,
                projected_at,
                channel_kind,
                conversation_id,
                sender_display_name,
                delivery_state,
                message_metadata,
                workflow_state,
                importance_score,
                ai_category,
                ai_summary,
                ai_summary_generated_at,
                local_state,
                local_state_changed_at,
                local_state_reason
            "#,
        )
        .bind(&canonical_message_id)
        .bind(&message.raw_record_id)
        .bind(&message.account_id)
        .bind(&message.provider_record_id)
        .bind(&message.subject)
        .bind(&message.sender)
        .bind(json!(message.recipients))
        .bind(&message.body_text)
        .bind(message.occurred_at)
        .fetch_optional(&self.pool)
        .await?;

        let Some(row) = row else {
            return Err(MessageProjectionError::RawRecordTupleMismatch {
                raw_record_id: message.raw_record_id.clone(),
                account_id: message.account_id.clone(),
                provider_record_id: message.provider_record_id.clone(),
            });
        };

        row_to_projected_message(row)
    }

    pub async fn upsert_channel_message(
        &self,
        message: &NewProjectedMessage,
    ) -> Result<ProjectedMessage, MessageProjectionError> {
        self.upsert_channel_message_with_body_policy(message, false)
            .await
    }

    pub async fn upsert_channel_message_allowing_empty_body_text(
        &self,
        message: &NewProjectedMessage,
    ) -> Result<ProjectedMessage, MessageProjectionError> {
        self.upsert_channel_message_with_body_policy(message, true)
            .await
    }

    async fn upsert_channel_message_with_body_policy(
        &self,
        message: &NewProjectedMessage,
        allow_empty_body_text: bool,
    ) -> Result<ProjectedMessage, MessageProjectionError> {
        message.validate_with_body_policy(allow_empty_body_text)?;

        let row = sqlx::query(
            r#"
            INSERT INTO communication_messages (
                message_id,
                raw_record_id,
                account_id,
                provider_record_id,
                subject,
                sender,
                recipients,
                body_text,
                occurred_at,
                channel_kind,
                conversation_id,
                sender_display_name,
                delivery_state,
                message_metadata
            )
            SELECT
                $1,
                raw_record_id,
                account_id,
                provider_record_id,
                $5,
                $6,
                $7,
                $8,
                $9,
                $10,
                $11,
                $12,
                $13,
                $14
            FROM communication_raw_records
            WHERE raw_record_id = $2
              AND account_id = $3
              AND provider_record_id = $4
            ON CONFLICT (account_id, provider_record_id)
            DO UPDATE SET
                message_id = EXCLUDED.message_id,
                raw_record_id = EXCLUDED.raw_record_id,
                subject = EXCLUDED.subject,
                sender = EXCLUDED.sender,
                recipients = EXCLUDED.recipients,
                body_text = EXCLUDED.body_text,
                occurred_at = EXCLUDED.occurred_at,
                channel_kind = EXCLUDED.channel_kind,
                conversation_id = EXCLUDED.conversation_id,
                sender_display_name = EXCLUDED.sender_display_name,
                delivery_state = EXCLUDED.delivery_state,
                message_metadata = EXCLUDED.message_metadata,
                projected_at = now()
            RETURNING
                message_id,
                raw_record_id,
                account_id,
                provider_record_id,
                subject,
                sender,
                recipients,
                body_text,
                occurred_at,
                projected_at,
                channel_kind,
                conversation_id,
                sender_display_name,
                delivery_state,
                message_metadata,
                workflow_state,
                importance_score,
                ai_category,
                ai_summary,
                ai_summary_generated_at,
                local_state,
                local_state_changed_at,
                local_state_reason
            "#,
        )
        .bind(&message.message_id)
        .bind(&message.raw_record_id)
        .bind(&message.account_id)
        .bind(&message.provider_record_id)
        .bind(&message.subject)
        .bind(&message.sender)
        .bind(json!(message.recipients))
        .bind(&message.body_text)
        .bind(message.occurred_at)
        .bind(&message.channel_kind)
        .bind(message.conversation_id.as_deref())
        .bind(message.sender_display_name.as_deref())
        .bind(&message.delivery_state)
        .bind(&message.message_metadata)
        .fetch_optional(&self.pool)
        .await?;

        let Some(row) = row else {
            return Err(MessageProjectionError::RawRecordTupleMismatch {
                raw_record_id: message.raw_record_id.clone(),
                account_id: message.account_id.clone(),
                provider_record_id: message.provider_record_id.clone(),
            });
        };

        row_to_projected_message(row)
    }

    pub async fn list_messages(
        &self,
        account_id: Option<&str>,
        workflow_state: Option<WorkflowState>,
        channel_kind: Option<&str>,
        query: Option<&str>,
        local_state: LocalMessageState,
        limit: i64,
    ) -> Result<Vec<ProjectedMessageSummary>, MessageProjectionError> {
        let limit = validate_limit(limit)?;
        let workflow_state_str = workflow_state.map(|s| s.as_str().to_owned());
        let local_state_filter = local_state.persisted_filter();
        let query = query
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned);
        let rows = sqlx::query(
            r#"
            SELECT
                m.message_id, m.raw_record_id, m.account_id, m.provider_record_id,
                m.subject, m.sender, m.recipients, m.body_text,
                m.occurred_at, m.projected_at, m.channel_kind, m.conversation_id,
                m.sender_display_name, m.delivery_state, m.message_metadata,
                m.workflow_state, m.importance_score, m.ai_category,
                m.ai_summary, m.ai_summary_generated_at,
                m.local_state, m.local_state_changed_at, m.local_state_reason,
                count(a.attachment_id)::BIGINT AS attachment_count
            FROM communication_messages m
            LEFT JOIN communication_attachments a ON a.message_id = m.message_id
            WHERE ($1::text IS NULL OR m.account_id = $1)
              AND ($2::text IS NULL OR m.workflow_state = $2)
              AND ($3::text IS NULL OR m.channel_kind = $3)
              AND ($4::text IS NULL OR m.local_state = $4)
              AND (
                $5::text IS NULL
                OR NOT EXISTS (
                  SELECT 1
                  FROM unnest(regexp_split_to_array(lower(trim($5)), '\s+')) AS term
                  WHERE term <> ''
                    AND lower(
                      concat_ws(
                        ' ',
                        m.subject,
                        m.sender,
                        m.body_text,
                        m.provider_record_id,
                        m.sender_display_name
                      )
                    ) NOT LIKE '%' || term || '%'
                )
              )
            GROUP BY
                m.message_id, m.raw_record_id, m.account_id, m.provider_record_id,
                m.subject, m.sender, m.recipients, m.body_text,
                m.occurred_at, m.projected_at, m.channel_kind, m.conversation_id,
                m.sender_display_name, m.delivery_state, m.message_metadata,
                m.workflow_state, m.importance_score, m.ai_category,
                m.ai_summary, m.ai_summary_generated_at,
                m.local_state, m.local_state_changed_at, m.local_state_reason
            ORDER BY COALESCE(m.occurred_at, m.projected_at) DESC, m.projected_at DESC, m.message_id ASC
            LIMIT $6
            "#,
        )
        .bind(account_id)
        .bind(workflow_state_str.as_deref())
        .bind(channel_kind)
        .bind(local_state_filter)
        .bind(query.as_deref())
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(row_to_projected_message_summary)
            .collect()
    }

    pub async fn count_messages_by_state(
        &self,
        account_id: Option<&str>,
    ) -> Result<Vec<WorkflowStateCount>, MessageProjectionError> {
        self.count_messages_by_state_with_local_state(account_id, LocalMessageState::Active)
            .await
    }

    pub async fn count_messages_by_state_with_local_state(
        &self,
        account_id: Option<&str>,
        local_state: LocalMessageState,
    ) -> Result<Vec<WorkflowStateCount>, MessageProjectionError> {
        let local_state_filter = local_state.persisted_filter();
        let rows = sqlx::query(
            r#"SELECT m.workflow_state, count(*)::BIGINT AS msg_count
            FROM communication_messages m
            WHERE ($1::text IS NULL OR m.account_id = $1)
              AND ($2::text IS NULL OR m.local_state = $2)
            GROUP BY m.workflow_state ORDER BY m.workflow_state"#,
        )
        .bind(account_id)
        .bind(local_state_filter)
        .fetch_all(&self.pool)
        .await?;
        let mut counts = Vec::new();
        for row in rows {
            let state: String = row.try_get("workflow_state")?;
            counts.push(WorkflowStateCount {
                state: state.parse::<WorkflowState>().unwrap_or(WorkflowState::New),
                count: row.try_get::<i64, _>("msg_count")?,
            });
        }
        Ok(counts)
    }

    pub async fn transition_workflow_state(
        &self,
        message_id: &str,
        new_state: WorkflowState,
    ) -> Result<ProjectedMessage, MessageProjectionError> {
        let mut transaction = self.pool.begin().await?;
        let message =
            Self::transition_workflow_state_in_transaction(&mut transaction, message_id, new_state)
                .await?;
        transaction.commit().await?;
        Ok(message)
    }

    pub(crate) async fn transition_workflow_state_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        message_id: &str,
        new_state: WorkflowState,
    ) -> Result<ProjectedMessage, MessageProjectionError> {
        validate_non_empty("message_id", message_id)?;
        let row = sqlx::query(
            r#"UPDATE communication_messages SET workflow_state = $2, projected_at = now()
            WHERE message_id = $1
            RETURNING
                message_id, raw_record_id, account_id, provider_record_id,
                subject, sender, recipients, body_text,
                occurred_at, projected_at, channel_kind, conversation_id,
                sender_display_name, delivery_state, message_metadata,
                workflow_state, importance_score, ai_category,
                ai_summary, ai_summary_generated_at,
                local_state, local_state_changed_at, local_state_reason"#,
        )
        .bind(message_id.trim())
        .bind(new_state.as_str())
        .fetch_optional(&mut **transaction)
        .await?;
        let Some(row) = row else {
            return Err(MessageProjectionError::MessageNotFound);
        };
        row_to_projected_message(row)
    }

    pub async fn move_to_local_trash(
        &self,
        message_id: &str,
        reason: &str,
    ) -> Result<ProjectedMessage, MessageProjectionError> {
        validate_non_empty("message_id", message_id)?;
        validate_non_empty("local_state_reason", reason)?;
        let row = sqlx::query(
            r#"UPDATE communication_messages
            SET local_state = 'trash',
                local_state_changed_at = now(),
                local_state_reason = $2,
                projected_at = now()
            WHERE message_id = $1
            RETURNING
                message_id, raw_record_id, account_id, provider_record_id,
                subject, sender, recipients, body_text,
                occurred_at, projected_at, channel_kind, conversation_id,
                sender_display_name, delivery_state, message_metadata,
                workflow_state, importance_score, ai_category,
                ai_summary, ai_summary_generated_at,
                local_state, local_state_changed_at, local_state_reason"#,
        )
        .bind(message_id.trim())
        .bind(reason.trim())
        .fetch_optional(&self.pool)
        .await?;
        let Some(row) = row else {
            return Err(MessageProjectionError::MessageNotFound);
        };
        row_to_projected_message(row)
    }

    pub async fn restore_from_local_trash(
        &self,
        message_id: &str,
    ) -> Result<ProjectedMessage, MessageProjectionError> {
        validate_non_empty("message_id", message_id)?;
        let row = sqlx::query(
            r#"UPDATE communication_messages
            SET local_state = 'active',
                local_state_changed_at = now(),
                local_state_reason = NULL,
                projected_at = now()
            WHERE message_id = $1
            RETURNING
                message_id, raw_record_id, account_id, provider_record_id,
                subject, sender, recipients, body_text,
                occurred_at, projected_at, channel_kind, conversation_id,
                sender_display_name, delivery_state, message_metadata,
                workflow_state, importance_score, ai_category,
                ai_summary, ai_summary_generated_at,
                local_state, local_state_changed_at, local_state_reason"#,
        )
        .bind(message_id.trim())
        .fetch_optional(&self.pool)
        .await?;
        let Some(row) = row else {
            return Err(MessageProjectionError::MessageNotFound);
        };
        row_to_projected_message(row)
    }

    pub async fn set_ai_analysis(
        &self,
        message_id: &str,
        category: Option<&str>,
        summary: Option<&str>,
        importance_score: Option<i16>,
    ) -> Result<ProjectedMessage, MessageProjectionError> {
        validate_non_empty("message_id", message_id)?;
        if let Some(score) = importance_score
            && !(0..=100).contains(&score)
        {
            return Err(MessageProjectionError::InvalidImportanceScore(score));
        }
        let row = sqlx::query(
            r#"UPDATE communication_messages SET
                ai_category = COALESCE($2, ai_category),
                ai_summary = COALESCE($3, ai_summary),
                ai_summary_generated_at = CASE WHEN $3 IS NOT NULL THEN now() ELSE ai_summary_generated_at END,
                importance_score = COALESCE($4, importance_score),
                projected_at = now()
            WHERE message_id = $1
            RETURNING
                message_id, raw_record_id, account_id, provider_record_id,
                subject, sender, recipients, body_text,
                occurred_at, projected_at, channel_kind, conversation_id,
                sender_display_name, delivery_state, message_metadata,
                workflow_state, importance_score, ai_category,
                ai_summary, ai_summary_generated_at,
                local_state, local_state_changed_at, local_state_reason"#,
        )
        .bind(message_id.trim())
        .bind(category)
        .bind(summary)
        .bind(importance_score)
        .fetch_optional(&self.pool)
        .await?;
        let Some(row) = row else {
            return Err(MessageProjectionError::MessageNotFound);
        };
        row_to_projected_message(row)
    }
    pub async fn set_message_metadata(
        &self,
        message_id: &str,
        metadata: &serde_json::Value,
    ) -> Result<ProjectedMessage, MessageProjectionError> {
        validate_non_empty("message_id", message_id)?;
        if !metadata.is_object() {
            return Err(MessageProjectionError::InvalidMessageMetadata);
        }
        let row = sqlx::query(
            r#"UPDATE communication_messages SET message_metadata = $2, projected_at = now()
            WHERE message_id = $1
            RETURNING
                message_id, raw_record_id, account_id, provider_record_id,
                subject, sender, recipients, body_text,
                occurred_at, projected_at, channel_kind, conversation_id,
                sender_display_name, delivery_state, message_metadata,
                workflow_state, importance_score, ai_category,
                ai_summary, ai_summary_generated_at,
                local_state, local_state_changed_at, local_state_reason"#,
        )
        .bind(message_id.trim())
        .bind(metadata)
        .fetch_optional(&self.pool)
        .await?;
        let Some(row) = row else {
            return Err(MessageProjectionError::MessageNotFound);
        };
        row_to_projected_message(row)
    }
}
