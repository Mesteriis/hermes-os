use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, QueryBuilder, Row};

use super::MessageProjectionStore;
use crate::domains::communications::messages::errors::MessageProjectionError;
use crate::domains::communications::messages::models::{
    MessageSearchMatchMode, MessageSearchQuery, ProjectedMessage, ProjectedMessagePage,
    ProjectedMessagePageQuery, ProjectedMessageSummary, WorkflowStateCount,
};
use crate::domains::communications::messages::rows::{
    row_to_projected_message, row_to_projected_message_summary,
};
use crate::domains::communications::messages::search::append_message_search_filter;
use crate::domains::communications::messages::states::{LocalMessageState, WorkflowState};
use crate::domains::communications::messages::validation::{validate_limit, validate_non_empty};

const TELEGRAM_CHANNEL_KIND_ALIAS: &[&str] = &["telegram_user", "telegram_bot"];
const WHATSAPP_CHANNEL_KIND_ALIAS: &[&str] = &["whatsapp_web", "whatsapp_business_cloud"];
const MAIL_CHANNEL_KIND_ALIAS: &[&str] = &["email"];

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
                m.observation_id,
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
                s.ai_state,
                m.local_state,
                m.local_state_changed_at,
                m.local_state_reason,
                m.is_read,
                m.read_changed_at,
                m.read_origin,
                count(a.attachment_id)::BIGINT AS attachment_count
            FROM communication_messages m
            LEFT JOIN communication_ai_states s ON s.message_id = m.message_id
            LEFT JOIN communication_attachments a ON a.message_id = m.message_id
            WHERE m.local_state = 'active'
            GROUP BY
                m.message_id,
                m.raw_record_id,
                m.observation_id,
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
                s.ai_state,
                m.local_state,
                m.local_state_changed_at,
                m.local_state_reason,
                m.is_read,
                m.read_changed_at,
                m.read_origin
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
                observation_id,
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
                (SELECT s.ai_state FROM communication_ai_states s WHERE s.message_id = communication_messages.message_id) AS ai_state,
                local_state,
                local_state_changed_at,
                local_state_reason,
                is_read,
                read_changed_at,
                read_origin
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
        Ok(self
            .list_messages_page(ProjectedMessagePageQuery {
                account_id,
                workflow_state,
                is_read: None,
                channel_kind,
                conversation_id: None,
                query,
                match_mode: MessageSearchMatchMode::All,
                search: MessageSearchQuery::default(),
                local_state,
                cursor: None,
                limit,
            })
            .await?
            .items)
    }

    pub async fn list_messages_page(
        &self,
        request: ProjectedMessagePageQuery<'_>,
    ) -> Result<ProjectedMessagePage, MessageProjectionError> {
        let limit = validate_limit(request.limit)?;
        let workflow_state_str = request.workflow_state.map(|s| s.as_str().to_owned());
        let local_state_filter = request.local_state.persisted_filter();
        let query = request
            .query
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned);
        let search = if request.search.is_empty() {
            fallback_message_search(query.as_deref(), request.match_mode)
        } else {
            request.search.clone()
        };
        let cursor = request
            .cursor
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(decode_message_list_cursor)
            .transpose()?;
        let cursor_sort_at = cursor.as_ref().map(|cursor| cursor.sort_at);
        let cursor_projected_at = cursor.as_ref().map(|cursor| cursor.projected_at);
        let cursor_message_id = cursor.as_ref().map(|cursor| cursor.message_id.as_str());
        let fetch_limit = limit + 1;
        let mut builder = QueryBuilder::<Postgres>::new(
            r#"
            SELECT
                m.message_id, m.raw_record_id, m.observation_id, m.account_id, m.provider_record_id,
                m.subject, m.sender, m.recipients, m.body_text,
                m.occurred_at, m.projected_at, m.channel_kind, m.conversation_id,
                m.sender_display_name, m.delivery_state, m.message_metadata,
                m.workflow_state, m.importance_score, m.ai_category,
                m.ai_summary, m.ai_summary_generated_at, s.ai_state,
                m.local_state, m.local_state_changed_at, m.local_state_reason,
                m.is_read, m.read_changed_at, m.read_origin,
                count(a.attachment_id)::BIGINT AS attachment_count
            FROM communication_messages m
            LEFT JOIN communication_ai_states s ON s.message_id = m.message_id
            LEFT JOIN communication_attachments a ON a.message_id = m.message_id
            WHERE 1 = 1
            "#,
        );
        if let Some(account_id) = request.account_id {
            builder.push(" AND m.account_id = ");
            builder.push_bind(account_id);
        }
        if let Some(workflow_state) = workflow_state_str.as_deref() {
            builder.push(" AND m.workflow_state = ");
            builder.push_bind(workflow_state);
        }
        if let Some(is_read) = request.is_read {
            builder.push(" AND m.is_read = ");
            builder.push_bind(is_read);
        }
        if let Some(channel_kind) = request.channel_kind {
            append_channel_kind_filter(&mut builder, channel_kind);
        }
        if let Some(conversation_id) = request.conversation_id {
            builder.push(" AND m.conversation_id = ");
            builder.push_bind(conversation_id);
        }
        if let Some(local_state) = local_state_filter {
            builder.push(" AND m.local_state = ");
            builder.push_bind(local_state);
        }
        append_message_search_filter(&mut builder, "m", &search);
        if let Some(sort_at) = cursor_sort_at {
            builder.push(" AND (COALESCE(m.occurred_at, m.projected_at) < ");
            builder.push_bind(sort_at);
            builder.push(" OR (COALESCE(m.occurred_at, m.projected_at) = ");
            builder.push_bind(sort_at);
            builder.push(" AND m.projected_at < ");
            builder.push_bind(cursor_projected_at.expect("cursor projected_at"));
            builder.push(") OR (COALESCE(m.occurred_at, m.projected_at) = ");
            builder.push_bind(sort_at);
            builder.push(" AND m.projected_at = ");
            builder.push_bind(cursor_projected_at.expect("cursor projected_at"));
            builder.push(" AND m.message_id > ");
            builder.push_bind(cursor_message_id.expect("cursor message_id"));
            builder.push("))");
        }
        builder.push(
            r#"
            GROUP BY
                m.message_id, m.raw_record_id, m.observation_id, m.account_id, m.provider_record_id,
                m.subject, m.sender, m.recipients, m.body_text,
                m.occurred_at, m.projected_at, m.channel_kind, m.conversation_id,
                m.sender_display_name, m.delivery_state, m.message_metadata,
                m.workflow_state, m.importance_score, m.ai_category,
                m.ai_summary, m.ai_summary_generated_at, s.ai_state,
                m.local_state, m.local_state_changed_at, m.local_state_reason,
                m.is_read, m.read_changed_at, m.read_origin
            ORDER BY COALESCE(m.occurred_at, m.projected_at) DESC, m.projected_at DESC, m.message_id ASC
            LIMIT 
            "#,
        );
        builder.push_bind(fetch_limit);
        let rows = builder.build().fetch_all(&self.pool).await?;
        let has_more = rows.len() > limit as usize;
        let summaries = rows
            .into_iter()
            .take(limit as usize)
            .map(row_to_projected_message_summary)
            .collect::<Result<Vec<_>, _>>()?;
        let next_cursor = if has_more {
            summaries
                .last()
                .map(|summary| encode_message_list_cursor(&summary.message))
                .transpose()?
        } else {
            None
        };

        Ok(ProjectedMessagePage {
            items: summaries,
            next_cursor,
            has_more,
        })
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

fn append_channel_kind_filter<'a>(builder: &mut QueryBuilder<'a, Postgres>, channel_kind: &'a str) {
    if let Some(channel_kinds) = channel_kind_alias_values(channel_kind) {
        builder.push(" AND m.channel_kind = ANY(");
        builder.push_bind(channel_kinds);
        builder.push(")");
        return;
    }

    builder.push(" AND m.channel_kind = ");
    builder.push_bind(channel_kind);
}

fn channel_kind_alias_values(channel_kind: &str) -> Option<&'static [&'static str]> {
    match channel_kind.trim() {
        "telegram" => Some(TELEGRAM_CHANNEL_KIND_ALIAS),
        "whatsapp" => Some(WHATSAPP_CHANNEL_KIND_ALIAS),
        "mail" => Some(MAIL_CHANNEL_KIND_ALIAS),
        _ => None,
    }
}

fn fallback_message_search(
    query: Option<&str>,
    match_mode: MessageSearchMatchMode,
) -> MessageSearchQuery {
    let plain_terms = query
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| {
            value
                .split_whitespace()
                .map(str::to_owned)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    MessageSearchQuery {
        plain_terms,
        match_mode,
        ..MessageSearchQuery::default()
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct MessageListCursor {
    sort_at: DateTime<Utc>,
    projected_at: DateTime<Utc>,
    message_id: String,
}

fn encode_message_list_cursor(
    message: &ProjectedMessage,
) -> Result<String, MessageProjectionError> {
    let cursor = MessageListCursor {
        sort_at: message.occurred_at.unwrap_or(message.projected_at),
        projected_at: message.projected_at,
        message_id: message.message_id.clone(),
    };
    let bytes = serde_json::to_vec(&cursor).map_err(|_| MessageProjectionError::InvalidCursor)?;

    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

fn decode_message_list_cursor(cursor: &str) -> Result<MessageListCursor, MessageProjectionError> {
    let bytes = URL_SAFE_NO_PAD
        .decode(cursor)
        .map_err(|_| MessageProjectionError::InvalidCursor)?;
    let cursor: MessageListCursor =
        serde_json::from_slice(&bytes).map_err(|_| MessageProjectionError::InvalidCursor)?;
    validate_non_empty("message_id", &cursor.message_id)?;

    Ok(cursor)
}
