use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use thiserror::Error;

use crate::event_log::{EventEnvelopeError, EventStore, EventStoreError, NewEventEnvelope};

const AUTOMATION_SEND_DRY_RUN_EVENT_TYPE: &str = "automation.telegram_send.dry_run";
const AUTOMATION_SOURCE_KIND: &str = "automation_policy";
const AUTOMATION_SOURCE_PROVIDER: &str = "local_policy_engine";

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
        request.validate()?;
        let actor_id = validate_non_empty("actor_id", actor_id)?;
        let (policy, template) = self.policy_with_template(&request.policy_id).await?;
        let rendered_text = evaluate_policy(&policy, &template, request)?;
        let rendered_preview_hash = sha256_hex(rendered_text.as_bytes());
        let outbound_message_id = format!(
            "telegram_outbound:v4:{}",
            sha256_hex(
                [
                    request.command_id.as_str(),
                    request.policy_id.as_str(),
                    request.provider_chat_id.as_str(),
                    rendered_preview_hash.as_str()
                ]
                .join("\0")
                .as_bytes()
            )
        );
        let mut transaction = self.pool.begin().await?;
        sqlx::query(
            r#"
            INSERT INTO telegram_outbound_messages (
                outbound_message_id,
                policy_id,
                template_id,
                account_id,
                provider_chat_id,
                send_mode,
                status,
                rendered_preview_hash,
                variables,
                source_context,
                actor_id
            )
            VALUES ($1, $2, $3, $4, $5, 'dry_run', 'allowed', $6, $7, $8, $9)
            ON CONFLICT (outbound_message_id)
            DO NOTHING
            "#,
        )
        .bind(&outbound_message_id)
        .bind(&policy.policy_id)
        .bind(&template.template_id)
        .bind(&policy.account_id)
        .bind(&request.provider_chat_id)
        .bind(&rendered_preview_hash)
        .bind(&request.variables)
        .bind(&request.source_context)
        .bind(&actor_id)
        .execute(&mut *transaction)
        .await?;

        let event_id = format!(
            "automation_telegram_send_dry_run:{}",
            request.command_id.trim()
        );
        let event = NewEventEnvelope::builder(
            event_id.clone(),
            AUTOMATION_SEND_DRY_RUN_EVENT_TYPE,
            Utc::now(),
            json!({
                "kind": AUTOMATION_SOURCE_KIND,
                "provider": AUTOMATION_SOURCE_PROVIDER,
                "policy_id": policy.policy_id,
            }),
            json!({
                "kind": "telegram_outbound_message",
                "id": outbound_message_id,
            }),
        )
        .actor(json!({"actor_id": actor_id}))
        .payload(json!({
            "command_id": request.command_id,
            "outbound_message_id": outbound_message_id,
            "policy_id": policy.policy_id,
            "template_id": template.template_id,
            "account_id": policy.account_id,
            "provider_chat_id": request.provider_chat_id,
            "rendered_preview_hash": rendered_preview_hash,
            "send_mode": "dry_run",
            "status": "allowed",
        }))
        .build()?;
        EventStore::append_in_transaction(&mut transaction, &event).await?;
        transaction.commit().await?;

        Ok(TelegramSendDryRunResponse {
            outbound_message_id,
            policy_id: policy.policy_id,
            template_id: template.template_id,
            account_id: policy.account_id,
            provider_chat_id: request.provider_chat_id.clone(),
            rendered_text,
            rendered_preview_hash,
            status: "allowed".to_owned(),
            event_id,
        })
    }

    async fn policy_with_template(
        &self,
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
        .fetch_optional(&self.pool)
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewAutomationTemplate {
    pub template_id: String,
    pub name: String,
    pub body_template: String,
    pub required_variables: Vec<String>,
}

impl NewAutomationTemplate {
    fn validate(&self) -> Result<(), AutomationError> {
        validate_non_empty("template_id", &self.template_id)?;
        validate_non_empty("name", &self.name)?;
        validate_non_empty("body_template", &self.body_template)?;
        for variable in &self.required_variables {
            validate_variable_name(variable)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct AutomationTemplate {
    pub template_id: String,
    pub name: String,
    pub body_template: String,
    pub required_variables: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewAutomationPolicy {
    pub policy_id: String,
    pub template_id: String,
    pub name: String,
    pub enabled: bool,
    pub account_id: String,
    pub allowed_chat_ids: Vec<String>,
    pub trigger_kind: String,
    pub max_sends_per_hour: i32,
    pub quiet_hours: Value,
    pub expires_at: Option<DateTime<Utc>>,
    pub conditions: Value,
}

impl NewAutomationPolicy {
    fn validate(&self) -> Result<(), AutomationError> {
        validate_non_empty("policy_id", &self.policy_id)?;
        validate_non_empty("template_id", &self.template_id)?;
        validate_non_empty("name", &self.name)?;
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("trigger_kind", &self.trigger_kind)?;
        if self.max_sends_per_hour <= 0 {
            return Err(AutomationError::InvalidRequest(
                "max_sends_per_hour must be greater than zero".to_owned(),
            ));
        }
        if self.allowed_chat_ids.is_empty() {
            return Err(AutomationError::InvalidRequest(
                "allowed_chat_ids must not be empty".to_owned(),
            ));
        }
        validate_object("quiet_hours", &self.quiet_hours)?;
        validate_object("conditions", &self.conditions)?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AutomationPolicy {
    pub policy_id: String,
    pub template_id: String,
    pub name: String,
    pub enabled: bool,
    pub account_id: String,
    pub allowed_chat_ids: Vec<String>,
    pub trigger_kind: String,
    pub max_sends_per_hour: i32,
    pub quiet_hours: Value,
    pub expires_at: Option<DateTime<Utc>>,
    pub conditions: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct TelegramSendDryRunRequest {
    pub command_id: String,
    pub policy_id: String,
    pub provider_chat_id: String,
    #[serde(default)]
    pub variables: Value,
    #[serde(default)]
    pub source_context: Value,
}

impl TelegramSendDryRunRequest {
    fn validate(&self) -> Result<(), AutomationError> {
        validate_non_empty("command_id", &self.command_id)?;
        validate_non_empty("policy_id", &self.policy_id)?;
        validate_non_empty("provider_chat_id", &self.provider_chat_id)?;
        validate_object("variables", &self.variables)?;
        validate_object("source_context", &self.source_context)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramSendDryRunResponse {
    pub outbound_message_id: String,
    pub policy_id: String,
    pub template_id: String,
    pub account_id: String,
    pub provider_chat_id: String,
    pub rendered_text: String,
    pub rendered_preview_hash: String,
    pub status: String,
    pub event_id: String,
}

#[derive(Debug, Error)]
pub enum AutomationError {
    #[error("invalid automation request: {0}")]
    InvalidRequest(String),

    #[error("automation policy was not found")]
    PolicyNotFound,

    #[error("automation policy is disabled")]
    PolicyDisabled,

    #[error("provider chat is not allowed by policy")]
    ChatNotAllowed,

    #[error("automation template variable is missing: {0}")]
    MissingTemplateVariable(String),

    #[error("automation template received undeclared variable: {0}")]
    UndeclaredTemplateVariable(String),

    #[error(transparent)]
    EventEnvelope(#[from] EventEnvelopeError),

    #[error(transparent)]
    EventStore(#[from] EventStoreError),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

fn evaluate_policy(
    policy: &AutomationPolicy,
    template: &AutomationTemplate,
    request: &TelegramSendDryRunRequest,
) -> Result<String, AutomationError> {
    if !policy.enabled {
        return Err(AutomationError::PolicyDisabled);
    }
    if let Some(expires_at) = policy.expires_at
        && expires_at < Utc::now()
    {
        return Err(AutomationError::InvalidRequest(
            "policy is expired".to_owned(),
        ));
    }
    if !policy
        .allowed_chat_ids
        .iter()
        .any(|chat_id| chat_id == &request.provider_chat_id)
    {
        return Err(AutomationError::ChatNotAllowed);
    }
    let variables = request
        .variables
        .as_object()
        .ok_or_else(|| AutomationError::InvalidRequest("variables must be an object".to_owned()))?;
    for key in variables.keys() {
        if !template
            .required_variables
            .iter()
            .any(|allowed| allowed == key)
        {
            return Err(AutomationError::UndeclaredTemplateVariable(key.clone()));
        }
    }

    let mut rendered = template.body_template.clone();
    for variable in &template.required_variables {
        let value = variables
            .get(variable)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| AutomationError::MissingTemplateVariable(variable.clone()))?;
        rendered = rendered.replace(&format!("{{{{{variable}}}}}"), value);
    }

    Ok(rendered)
}

fn row_to_template(row: PgRow) -> Result<AutomationTemplate, AutomationError> {
    Ok(AutomationTemplate {
        template_id: row.try_get("template_id")?,
        name: row.try_get("name")?,
        body_template: row.try_get("body_template")?,
        required_variables: string_vec_from_value(row.try_get("required_variables")?)?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn row_to_policy(row: PgRow) -> Result<AutomationPolicy, AutomationError> {
    Ok(AutomationPolicy {
        policy_id: row.try_get("policy_id")?,
        template_id: row.try_get("template_id")?,
        name: row.try_get("name")?,
        enabled: row.try_get("enabled")?,
        account_id: row.try_get("account_id")?,
        allowed_chat_ids: string_vec_from_value(row.try_get("allowed_chat_ids")?)?,
        trigger_kind: row.try_get("trigger_kind")?,
        max_sends_per_hour: row.try_get("max_sends_per_hour")?,
        quiet_hours: row.try_get("quiet_hours")?,
        expires_at: row.try_get("expires_at")?,
        conditions: row.try_get("conditions")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn string_vec_from_value(value: Value) -> Result<Vec<String>, AutomationError> {
    let values = value
        .as_array()
        .ok_or_else(|| AutomationError::InvalidRequest("expected array".to_owned()))?;
    values
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(ToOwned::to_owned)
                .ok_or_else(|| AutomationError::InvalidRequest("expected string array".to_owned()))
        })
        .collect()
}

fn validate_variable_name(value: &str) -> Result<String, AutomationError> {
    let value = validate_non_empty("required_variable", value)?;
    if !value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
    {
        return Err(AutomationError::InvalidRequest(
            "template variables must be ASCII letters, numbers or underscores".to_owned(),
        ));
    }
    Ok(value)
}

fn validate_non_empty(field: &'static str, value: &str) -> Result<String, AutomationError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(AutomationError::InvalidRequest(format!(
            "{field} must not be empty"
        )));
    }
    Ok(trimmed.to_owned())
}

fn validate_object(field: &'static str, value: &Value) -> Result<(), AutomationError> {
    if !matches!(value, Value::Object(_)) {
        return Err(AutomationError::InvalidRequest(format!(
            "{field} must be a JSON object"
        )));
    }
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("sha256:{:x}", hasher.finalize())
}

pub fn object_from_pairs(pairs: impl IntoIterator<Item = (String, Value)>) -> Value {
    Value::Object(Map::from_iter(pairs))
}
