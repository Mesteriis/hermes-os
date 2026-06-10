use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::{Value, json};
use sqlx::Row;
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::platform::capabilities::CapabilityDecision;

const API_FRONTEND_ACTOR_KIND: &str = "frontend";
const EVENT_TARGET_KIND: &str = "event";

#[derive(Clone)]
pub struct ApiAuditLog {
    pool: PgPool,
}

impl ApiAuditLog {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn record(&self, record: &NewApiAuditRecord) -> Result<i64, ApiAuditError> {
        let audit_id = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO api_audit_log (
                actor_kind,
                actor_id,
                operation,
                method,
                path_template,
                target_kind,
                target_id,
                metadata
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING audit_id
            "#,
        )
        .bind(&record.actor_kind)
        .bind(&record.actor_id)
        .bind(&record.operation)
        .bind(&record.method)
        .bind(&record.path_template)
        .bind(&record.target_kind)
        .bind(&record.target_id)
        .bind(&record.metadata)
        .fetch_one(&self.pool)
        .await?;

        Ok(audit_id)
    }

    pub async fn list_event_records(
        &self,
        target_id: Option<&str>,
        actor_id: Option<&str>,
        after_audit_id: i64,
        limit: u32,
    ) -> Result<Vec<ApiAuditRecord>, ApiAuditError> {
        let target_id = target_id.map(str::trim).filter(|value| !value.is_empty());
        let actor_id = actor_id.map(str::trim).filter(|value| !value.is_empty());
        let after_audit_id = after_audit_id.max(0);
        let limit = i64::from(limit.clamp(1, 500));

        let rows = sqlx::query(
            r#"
            SELECT
                audit_id,
                recorded_at,
                actor_kind,
                actor_id,
                operation,
                method,
                path_template,
                target_kind,
                target_id,
                metadata
            FROM api_audit_log
            WHERE target_kind = 'event'
              AND ($1::text IS NULL OR target_id = $1)
              AND ($2::text IS NULL OR actor_id = $2)
              AND audit_id > $3
            ORDER BY audit_id ASC
            LIMIT $4
            "#,
        )
        .bind(target_id)
        .bind(actor_id)
        .bind(after_audit_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| {
                Ok(ApiAuditRecord {
                    audit_id: row.try_get("audit_id")?,
                    recorded_at: row.try_get("recorded_at")?,
                    actor_kind: row.try_get("actor_kind")?,
                    actor_id: row.try_get("actor_id")?,
                    operation: row.try_get("operation")?,
                    method: row.try_get("method")?,
                    path_template: row.try_get("path_template")?,
                    target_kind: row.try_get("target_kind")?,
                    target_id: row.try_get("target_id")?,
                    metadata: row.try_get("metadata")?,
                })
            })
            .collect()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ApiAuditRecord {
    pub audit_id: i64,
    pub recorded_at: DateTime<Utc>,
    pub actor_kind: String,
    pub actor_id: Option<String>,
    pub operation: String,
    pub method: String,
    pub path_template: String,
    pub target_kind: String,
    pub target_id: Option<String>,
    pub metadata: Value,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewApiAuditRecord {
    actor_kind: String,
    actor_id: String,
    operation: String,
    method: String,
    path_template: String,
    target_kind: String,
    target_id: Option<String>,
    metadata: Value,
}

impl NewApiAuditRecord {
    pub fn event_append(actor_id: impl Into<String>, event_id: impl Into<String>) -> Self {
        Self {
            actor_kind: API_FRONTEND_ACTOR_KIND.to_owned(),
            actor_id: actor_id.into(),
            operation: "event.append".to_owned(),
            method: "POST".to_owned(),
            path_template: "/api/v1/events".to_owned(),
            target_kind: EVENT_TARGET_KIND.to_owned(),
            target_id: Some(event_id.into()),
            metadata: json!({}),
        }
    }

    pub fn event_get(actor_id: impl Into<String>, event_id: impl Into<String>) -> Self {
        Self {
            actor_kind: API_FRONTEND_ACTOR_KIND.to_owned(),
            actor_id: actor_id.into(),
            operation: "event.get".to_owned(),
            method: "GET".to_owned(),
            path_template: "/api/v1/events/{event_id}".to_owned(),
            target_kind: EVENT_TARGET_KIND.to_owned(),
            target_id: Some(event_id.into()),
            metadata: json!({}),
        }
    }

    pub fn project_link_review_set(
        actor_id: impl Into<String>,
        project_id: impl Into<String>,
        target_kind: impl Into<String>,
        target_id: impl Into<String>,
    ) -> Self {
        let project_id = project_id.into();
        let target_kind = target_kind.into();
        let target_id = target_id.into();

        Self {
            actor_kind: API_FRONTEND_ACTOR_KIND.to_owned(),
            actor_id: actor_id.into(),
            operation: "project.link_review.set".to_owned(),
            method: "PUT".to_owned(),
            path_template: "/api/v1/projects/{project_id}/link-reviews".to_owned(),
            target_kind: "project_link".to_owned(),
            target_id: Some(format!("{project_id}:{target_kind}:{target_id}")),
            metadata: json!({
                "project_id": project_id,
                "target_kind": target_kind,
                "target_id": target_id,
            }),
        }
    }

    pub fn task_candidate_review_set(
        actor_id: impl Into<String>,
        task_candidate_id: impl Into<String>,
    ) -> Self {
        Self {
            actor_kind: API_FRONTEND_ACTOR_KIND.to_owned(),
            actor_id: actor_id.into(),
            operation: "task_candidate.review.set".to_owned(),
            method: "PUT".to_owned(),
            path_template: "/api/v1/task-candidates/{task_candidate_id}/review".to_owned(),
            target_kind: "task_candidate".to_owned(),
            target_id: Some(task_candidate_id.into()),
            metadata: json!({}),
        }
    }

    pub fn message_workflow_state_set(
        actor_id: impl Into<String>,
        message_id: impl Into<String>,
    ) -> Self {
        Self {
            actor_kind: API_FRONTEND_ACTOR_KIND.to_owned(),
            actor_id: actor_id.into(),
            operation: "message.workflow_state.set".to_owned(),
            method: "PUT".to_owned(),
            path_template: "/api/v1/communications/messages/{message_id}/workflow-state".to_owned(),
            target_kind: "communication_message".to_owned(),
            target_id: Some(message_id.into()),
            metadata: json!({}),
        }
    }

    pub fn communication_email_send(
        actor_id: impl Into<String>,
        account_id: impl Into<String>,
        recipient_count: usize,
    ) -> Self {
        Self {
            actor_kind: API_FRONTEND_ACTOR_KIND.to_owned(),
            actor_id: actor_id.into(),
            operation: "communication.email.send".to_owned(),
            method: "POST".to_owned(),
            path_template: "/api/v1/communications/send".to_owned(),
            target_kind: "communication_provider_account".to_owned(),
            target_id: Some(account_id.into()),
            metadata: json!({
                "action_class": "provider_write",
                "transport": "smtp",
                "recipient_count": recipient_count,
            }),
        }
    }

    pub fn person_identity_review_set(
        actor_id: impl Into<String>,
        identity_candidate_id: impl Into<String>,
    ) -> Self {
        let identity_candidate_id = identity_candidate_id.into();
        Self {
            actor_kind: API_FRONTEND_ACTOR_KIND.to_owned(),
            actor_id: actor_id.into(),
            operation: "person_identity.review.set".to_owned(),
            method: "PUT".to_owned(),
            path_template: "/api/v1/identity-candidates/{identity_candidate_id}/review".to_owned(),
            target_kind: "person_identity_candidate".to_owned(),
            target_id: Some(identity_candidate_id),
            metadata: json!({}),
        }
    }

    pub fn application_setting_set(
        actor_id: impl Into<String>,
        setting_key: impl Into<String>,
    ) -> Self {
        let setting_key = setting_key.into();
        Self {
            actor_kind: API_FRONTEND_ACTOR_KIND.to_owned(),
            actor_id: actor_id.into(),
            operation: "application_setting.set".to_owned(),
            method: "PUT".to_owned(),
            path_template: "/api/v1/settings/{setting_key}".to_owned(),
            target_kind: "application_setting".to_owned(),
            target_id: Some(setting_key),
            metadata: json!({}),
        }
    }

    pub fn automation_telegram_send_dry_run(
        actor_id: impl Into<String>,
        outbound_message_id: impl Into<String>,
        policy_id: impl Into<String>,
        template_id: impl Into<String>,
        account_id: impl Into<String>,
        provider_chat_id: impl Into<String>,
        rendered_preview_hash: impl Into<String>,
    ) -> Self {
        let outbound_message_id = outbound_message_id.into();
        let policy_id = policy_id.into();
        let template_id = template_id.into();
        let account_id = account_id.into();
        let provider_chat_id = provider_chat_id.into();
        let rendered_preview_hash = rendered_preview_hash.into();
        let decision =
            CapabilityDecision::scoped_automation_allowed("telegram.send", policy_id.clone());

        Self::automation_telegram_send_dry_run_decision(
            actor_id,
            TelegramSendDryRunAuditDecision {
                target_kind: "telegram_outbound_message",
                target_id: Some(outbound_message_id),
                policy_id,
                template_id: Some(template_id),
                account_id: Some(account_id),
                provider_chat_id,
                rendered_preview_hash: Some(rendered_preview_hash),
                decision: &decision,
            },
        )
    }

    pub fn automation_telegram_send_dry_run_rejected(
        actor_id: impl Into<String>,
        command_id: impl Into<String>,
        policy_id: impl Into<String>,
        provider_chat_id: impl Into<String>,
        decision: &CapabilityDecision,
    ) -> Self {
        Self::automation_telegram_send_dry_run_decision(
            actor_id,
            TelegramSendDryRunAuditDecision {
                target_kind: "telegram_send_request",
                target_id: non_empty_optional(command_id.into()),
                policy_id: policy_id.into(),
                template_id: None,
                account_id: None,
                provider_chat_id: provider_chat_id.into(),
                rendered_preview_hash: None,
                decision,
            },
        )
    }

    fn automation_telegram_send_dry_run_decision(
        actor_id: impl Into<String>,
        audit_decision: TelegramSendDryRunAuditDecision<'_>,
    ) -> Self {
        let mut metadata = audit_decision.decision.audit_metadata();
        let metadata_object = metadata
            .as_object_mut()
            .expect("capability decision metadata must be an object");
        insert_non_empty(metadata_object, "policy_id", audit_decision.policy_id);
        insert_optional(metadata_object, "template_id", audit_decision.template_id);
        insert_optional(metadata_object, "account_id", audit_decision.account_id);
        insert_non_empty(
            metadata_object,
            "provider_chat_id",
            audit_decision.provider_chat_id,
        );
        insert_optional(
            metadata_object,
            "rendered_preview_hash",
            audit_decision.rendered_preview_hash,
        );

        Self {
            actor_kind: API_FRONTEND_ACTOR_KIND.to_owned(),
            actor_id: actor_id.into(),
            operation: "automation.telegram_send.dry_run".to_owned(),
            method: "POST".to_owned(),
            path_template: "/api/v1/policies/telegram-send/dry-run".to_owned(),
            target_kind: audit_decision.target_kind.to_owned(),
            target_id: audit_decision.target_id,
            metadata,
        }
    }

    pub fn document_processing_job_retry(
        actor_id: impl Into<String>,
        job_id: impl Into<String>,
    ) -> Self {
        let job_id = job_id.into();
        Self {
            actor_kind: API_FRONTEND_ACTOR_KIND.to_owned(),
            actor_id: actor_id.into(),
            operation: "document_processing.job.retry".to_owned(),
            method: "POST".to_owned(),
            path_template: "/api/v1/document-processing/jobs/{job_id}/retry".to_owned(),
            target_kind: "document_processing_job".to_owned(),
            target_id: Some(job_id),
            metadata: json!({}),
        }
    }
}

struct TelegramSendDryRunAuditDecision<'a> {
    target_kind: &'static str,
    target_id: Option<String>,
    policy_id: String,
    template_id: Option<String>,
    account_id: Option<String>,
    provider_chat_id: String,
    rendered_preview_hash: Option<String>,
    decision: &'a CapabilityDecision,
}

fn insert_non_empty(
    metadata: &mut serde_json::Map<String, Value>,
    key: &'static str,
    value: String,
) {
    let value = value.trim();
    if !value.is_empty() {
        metadata.insert(key.to_owned(), json!(value));
    }
}

fn insert_optional(
    metadata: &mut serde_json::Map<String, Value>,
    key: &'static str,
    value: Option<String>,
) {
    if let Some(value) = value {
        insert_non_empty(metadata, key, value);
    }
}

fn non_empty_optional(value: String) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_owned())
    }
}

#[derive(Debug, Error)]
pub enum ApiAuditError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}
