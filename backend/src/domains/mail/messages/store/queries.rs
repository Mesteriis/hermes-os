use sqlx::Row;

use super::MessageProjectionStore;
use crate::domains::mail::messages::errors::MessageProjectionError;
use crate::domains::mail::messages::models::{
    ProjectedMessage, ProjectedMessageSummary, WorkflowStateCount,
};
use crate::domains::mail::messages::rows::{
    row_to_projected_message, row_to_projected_message_summary,
};
use crate::domains::mail::messages::states::{LocalMessageState, WorkflowState};
use crate::domains::mail::messages::validation::{validate_limit, validate_non_empty};

impl MessageProjectionStore {
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
}
