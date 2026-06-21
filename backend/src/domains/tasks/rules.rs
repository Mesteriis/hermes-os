use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::PgPool;
use thiserror::Error;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskRule {
    pub rule_id: String,
    pub name: String,
    pub natural_language_description: Option<String>,
    pub compiled_dsl: Value,
    pub enabled: bool,
    pub approval_mode: String,
    pub last_run_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct TaskRuleStore {
    pool: PgPool,
}
impl TaskRuleStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    pub async fn list(&self) -> Result<Vec<TaskRule>, TaskRuleError> {
        let rows = sqlx::query("SELECT rule_id, name, natural_language_description, compiled_dsl, enabled, approval_mode, last_run_at, created_at, updated_at FROM task_rules ORDER BY name")
            .fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(|r| {
                Ok(TaskRule {
                    rule_id: r.try_get("rule_id")?,
                    name: r.try_get("name")?,
                    natural_language_description: r.try_get("natural_language_description")?,
                    compiled_dsl: r.try_get("compiled_dsl")?,
                    enabled: r.try_get("enabled")?,
                    approval_mode: r.try_get("approval_mode")?,
                    last_run_at: r.try_get("last_run_at")?,
                    created_at: r.try_get("created_at")?,
                    updated_at: r.try_get("updated_at")?,
                })
            })
            .collect()
    }
    pub async fn create(
        &self,
        name: &str,
        desc: Option<&str>,
        dsl: Value,
        approval: Option<&str>,
    ) -> Result<TaskRule, TaskRuleError> {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let rule_id = format!("taskrule:v1:{ts:x}");
        let row = sqlx::query("INSERT INTO task_rules (rule_id, name, natural_language_description, compiled_dsl, approval_mode) VALUES ($1,$2,$3,$4,$5) RETURNING rule_id, name, natural_language_description, compiled_dsl, enabled, approval_mode, last_run_at, created_at, updated_at")
            .bind(&rule_id).bind(name).bind(desc).bind(&dsl).bind(approval.unwrap_or("suggest_only")).fetch_one(&self.pool).await?;
        Ok(TaskRule {
            rule_id: row.try_get("rule_id")?,
            name: row.try_get("name")?,
            natural_language_description: row.try_get("natural_language_description")?,
            compiled_dsl: row.try_get("compiled_dsl")?,
            enabled: row.try_get("enabled")?,
            approval_mode: row.try_get("approval_mode")?,
            last_run_at: row.try_get("last_run_at")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
    pub async fn delete(&self, rule_id: &str) -> Result<(), TaskRuleError> {
        sqlx::query("DELETE FROM task_rules WHERE rule_id=$1")
            .bind(rule_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskTemplate {
    pub template_id: String,
    pub name: String,
    pub description: Option<String>,
    pub default_fields: Value,
    pub default_checklist: Value,
    pub default_priority: String,
    pub default_energy_type: Option<String>,
    pub required_documents: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct TaskTemplateStore {
    pool: PgPool,
}
impl TaskTemplateStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    pub async fn list(&self) -> Result<Vec<TaskTemplate>, TaskRuleError> {
        let rows = sqlx::query("SELECT template_id, name, description, default_fields, default_checklist, default_priority, default_energy_type, required_documents, created_at, updated_at FROM task_templates ORDER BY name")
            .fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(|r| {
                Ok(TaskTemplate {
                    template_id: r.try_get("template_id")?,
                    name: r.try_get("name")?,
                    description: r.try_get("description")?,
                    default_fields: r.try_get("default_fields")?,
                    default_checklist: r.try_get("default_checklist")?,
                    default_priority: r.try_get("default_priority")?,
                    default_energy_type: r.try_get("default_energy_type")?,
                    required_documents: r.try_get("required_documents")?,
                    created_at: r.try_get("created_at")?,
                    updated_at: r.try_get("updated_at")?,
                })
            })
            .collect()
    }
}

#[derive(Debug, Error)]
pub enum TaskRuleError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error("not found")]
    NotFound,
}
