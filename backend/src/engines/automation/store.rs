use serde_json::json;
use sqlx::Row;
use sqlx::postgres::PgPool;

use super::errors::AutomationError;
use super::models::{
    AutomationPolicy, AutomationTemplate, NewAutomationPolicy, NewAutomationTemplate,
    TelegramSendDryRunRequest, TelegramSendDryRunResponse,
};
use super::rows::{row_to_policy, row_to_template, string_vec_from_value};
use super::validation::validate_non_empty;

#[derive(Clone)]
pub struct AutomationStore {
    pool: PgPool,
}

impl AutomationStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert_template(
        &self,
        template: &NewAutomationTemplate,
    ) -> Result<AutomationTemplate, AutomationError> {
        template.validate()?;
        let row = sqlx::query(
            r#"
            INSERT INTO automation_templates (
                template_id,
                name,
                body_template,
                required_variables,
                updated_at
            )
            VALUES ($1, $2, $3, $4, now())
            ON CONFLICT (template_id)
            DO UPDATE SET
                name = EXCLUDED.name,
                body_template = EXCLUDED.body_template,
                required_variables = EXCLUDED.required_variables,
                updated_at = now()
            RETURNING
                template_id,
                name,
                body_template,
                required_variables,
                created_at,
                updated_at
            "#,
        )
        .bind(template.template_id.trim())
        .bind(template.name.trim())
        .bind(template.body_template.trim())
        .bind(json!(template.required_variables))
        .fetch_one(&self.pool)
        .await?;

        row_to_template(row)
    }

    pub async fn upsert_policy(
        &self,
        policy: &NewAutomationPolicy,
    ) -> Result<AutomationPolicy, AutomationError> {
        policy.validate()?;
        let row = sqlx::query(
            r#"
            INSERT INTO automation_policies (
                policy_id,
                template_id,
                name,
                enabled,
                account_id,
                allowed_chat_ids,
                trigger_kind,
                max_sends_per_hour,
                quiet_hours,
                expires_at,
                conditions,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, now())
            ON CONFLICT (policy_id)
            DO UPDATE SET
                template_id = EXCLUDED.template_id,
                name = EXCLUDED.name,
                enabled = EXCLUDED.enabled,
                account_id = EXCLUDED.account_id,
                allowed_chat_ids = EXCLUDED.allowed_chat_ids,
                trigger_kind = EXCLUDED.trigger_kind,
                max_sends_per_hour = EXCLUDED.max_sends_per_hour,
                quiet_hours = EXCLUDED.quiet_hours,
                expires_at = EXCLUDED.expires_at,
                conditions = EXCLUDED.conditions,
                updated_at = now()
            RETURNING
                policy_id,
                template_id,
                name,
                enabled,
                account_id,
                allowed_chat_ids,
                trigger_kind,
                max_sends_per_hour,
                quiet_hours,
                expires_at,
                conditions,
                created_at,
                updated_at
            "#,
        )
        .bind(policy.policy_id.trim())
        .bind(policy.template_id.trim())
        .bind(policy.name.trim())
        .bind(policy.enabled)
        .bind(policy.account_id.trim())
        .bind(json!(policy.allowed_chat_ids))
        .bind(policy.trigger_kind.trim())
        .bind(policy.max_sends_per_hour)
        .bind(&policy.quiet_hours)
        .bind(policy.expires_at)
        .bind(&policy.conditions)
        .fetch_one(&self.pool)
        .await?;

        row_to_policy(row)
    }

    pub async fn list_templates(&self) -> Result<Vec<AutomationTemplate>, AutomationError> {
        let rows = sqlx::query(
            r#"
            SELECT
                template_id,
                name,
                body_template,
                required_variables,
                created_at,
                updated_at
            FROM automation_templates
            ORDER BY updated_at DESC, template_id ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_template).collect()
    }

    pub async fn list_policies(&self) -> Result<Vec<AutomationPolicy>, AutomationError> {
        let rows = sqlx::query(
            r#"
            SELECT
                policy_id,
                template_id,
                name,
                enabled,
                account_id,
                allowed_chat_ids,
                trigger_kind,
                max_sends_per_hour,
                quiet_hours,
                expires_at,
                conditions,
                created_at,
                updated_at
            FROM automation_policies
            ORDER BY updated_at DESC, policy_id ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_policy).collect()
    }

    pub async fn dry_run_send(
        &self,
        request: &TelegramSendDryRunRequest,
        actor_id: &str,
    ) -> Result<TelegramSendDryRunResponse, AutomationError> {
        super::dry_run::dry_run_send(&self.pool, request, actor_id).await
    }

    pub(super) async fn policy_with_template(
        pool: &PgPool,
        policy_id: &str,
    ) -> Result<(AutomationPolicy, AutomationTemplate), AutomationError> {
        let policy_id = validate_non_empty("policy_id", policy_id)?;
        let row = sqlx::query(
            r#"
            SELECT
                p.policy_id,
                p.template_id,
                p.name AS policy_name,
                p.enabled,
                p.account_id,
                p.allowed_chat_ids,
                p.trigger_kind,
                p.max_sends_per_hour,
                p.quiet_hours,
                p.expires_at,
                p.conditions,
                p.created_at AS policy_created_at,
                p.updated_at AS policy_updated_at,
                t.name AS template_name,
                t.body_template,
                t.required_variables,
                t.created_at AS template_created_at,
                t.updated_at AS template_updated_at
            FROM automation_policies p
            JOIN automation_templates t ON t.template_id = p.template_id
            WHERE p.policy_id = $1
            "#,
        )
        .bind(&policy_id)
        .fetch_optional(pool)
        .await?
        .ok_or(AutomationError::PolicyNotFound)?;

        Ok((
            AutomationPolicy {
                policy_id: row.try_get("policy_id")?,
                template_id: row.try_get("template_id")?,
                name: row.try_get("policy_name")?,
                enabled: row.try_get("enabled")?,
                account_id: row.try_get("account_id")?,
                allowed_chat_ids: string_vec_from_value(row.try_get("allowed_chat_ids")?)?,
                trigger_kind: row.try_get("trigger_kind")?,
                max_sends_per_hour: row.try_get("max_sends_per_hour")?,
                quiet_hours: row.try_get("quiet_hours")?,
                expires_at: row.try_get("expires_at")?,
                conditions: row.try_get("conditions")?,
                created_at: row.try_get("policy_created_at")?,
                updated_at: row.try_get("policy_updated_at")?,
            },
            AutomationTemplate {
                template_id: row.try_get("template_id")?,
                name: row.try_get("template_name")?,
                body_template: row.try_get("body_template")?,
                required_variables: string_vec_from_value(row.try_get("required_variables")?)?,
                created_at: row.try_get("template_created_at")?,
                updated_at: row.try_get("template_updated_at")?,
            },
        ))
    }
}
