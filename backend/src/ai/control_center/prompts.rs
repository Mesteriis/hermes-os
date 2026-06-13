use chrono::Utc;
use serde_json::{Value, json};

use super::errors::AiControlCenterError;
use super::models::{
    AiPromptActivateRequest, AiPromptCreateRequest, AiPromptEvalRun, AiPromptTemplate,
    AiPromptTestRequest, AiPromptVersion, AiPromptVersionCreateRequest,
};
use super::rows::{row_to_eval_run, row_to_prompt, row_to_prompt_version};
use super::store::AiControlCenterStore;
use super::validation::{
    object_value, reject_secret_like_json, render_prompt, slug_id, string_array_value,
    validate_non_empty,
};

impl AiControlCenterStore {
    pub async fn list_prompts(&self) -> Result<Vec<AiPromptTemplate>, AiControlCenterError> {
        let rows = sqlx::query(
            r#"
            SELECT
                prompt_id,
                name,
                entity_scope,
                capability_slot,
                description,
                is_system,
                active_version_id,
                metadata,
                created_at,
                updated_at
            FROM ai_prompt_templates
            ORDER BY entity_scope ASC, capability_slot ASC, name ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_prompt).collect()
    }

    pub async fn create_prompt(
        &self,
        request: &AiPromptCreateRequest,
        actor_id: &str,
    ) -> Result<AiPromptTemplate, AiControlCenterError> {
        validate_non_empty("actor_id", actor_id)?;
        request.validate()?;
        let prompt_id = request
            .prompt_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
            .unwrap_or_else(|| {
                format!(
                    "prompt:user:{}:{}",
                    request.entity_scope.trim(),
                    slug_id(request.name.trim())
                )
            });
        let metadata = object_value(
            request.metadata.clone().unwrap_or_else(|| json!({})),
            "metadata",
        )?;
        reject_secret_like_json(&Value::Object(metadata.clone()))?;
        let row = sqlx::query(
            r#"
            INSERT INTO ai_prompt_templates (
                prompt_id,
                name,
                entity_scope,
                capability_slot,
                description,
                is_system,
                metadata
            )
            VALUES ($1, $2, $3, $4, $5, false, $6)
            RETURNING
                prompt_id,
                name,
                entity_scope,
                capability_slot,
                description,
                is_system,
                active_version_id,
                metadata,
                created_at,
                updated_at
            "#,
        )
        .bind(prompt_id)
        .bind(request.name.trim())
        .bind(request.entity_scope.trim())
        .bind(request.capability_slot.trim())
        .bind(request.description.as_deref().map(str::trim))
        .bind(Value::Object(metadata))
        .fetch_one(&self.pool)
        .await?;

        row_to_prompt(row)
    }

    pub async fn create_prompt_version(
        &self,
        prompt_id: &str,
        request: &AiPromptVersionCreateRequest,
        actor_id: &str,
    ) -> Result<AiPromptVersion, AiControlCenterError> {
        validate_non_empty("prompt_id", prompt_id)?;
        validate_non_empty("actor_id", actor_id)?;
        request.validate()?;
        let prompt = self
            .prompt(prompt_id)
            .await?
            .ok_or(AiControlCenterError::PromptNotFound)?;
        if prompt.is_system {
            return Err(AiControlCenterError::InvalidRequest(
                "system prompts are read-only".to_owned(),
            ));
        }
        let variables = string_array_value(request.variables.clone(), "variables")?;
        let version_label = request
            .version_label
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
            .unwrap_or_else(|| Utc::now().format("v%Y%m%d%H%M%S").to_string());
        let prompt_version_id = request
            .prompt_version_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
            .unwrap_or_else(|| {
                format!(
                    "prompt-version:{}:{}",
                    prompt_id.trim(),
                    slug_id(&version_label)
                )
            });
        let row = sqlx::query(
            r#"
            INSERT INTO ai_prompt_template_versions (
                prompt_version_id,
                prompt_id,
                version_label,
                body_template,
                variables,
                status,
                created_by_actor_id
            )
            VALUES ($1, $2, $3, $4, $5, 'draft', $6)
            RETURNING
                prompt_version_id,
                prompt_id,
                version_label,
                body_template,
                variables,
                status,
                created_by_actor_id,
                created_at,
                updated_at
            "#,
        )
        .bind(prompt_version_id)
        .bind(prompt_id.trim())
        .bind(version_label)
        .bind(request.body_template.trim())
        .bind(json!(variables))
        .bind(actor_id.trim())
        .fetch_one(&self.pool)
        .await?;

        row_to_prompt_version(row)
    }

    pub async fn activate_prompt_version(
        &self,
        prompt_id: &str,
        request: &AiPromptActivateRequest,
    ) -> Result<AiPromptTemplate, AiControlCenterError> {
        validate_non_empty("prompt_id", prompt_id)?;
        validate_non_empty("prompt_version_id", &request.prompt_version_id)?;
        let prompt = self
            .prompt(prompt_id)
            .await?
            .ok_or(AiControlCenterError::PromptNotFound)?;
        if prompt.is_system {
            return Err(AiControlCenterError::InvalidRequest(
                "system prompts are read-only".to_owned(),
            ));
        }
        let mut tx = self.pool.begin().await?;
        let version_exists: Option<String> = sqlx::query_scalar(
            "SELECT prompt_version_id FROM ai_prompt_template_versions WHERE prompt_id = $1 AND prompt_version_id = $2",
        )
        .bind(prompt_id.trim())
        .bind(request.prompt_version_id.trim())
        .fetch_optional(&mut *tx)
        .await?;
        if version_exists.is_none() {
            return Err(AiControlCenterError::PromptVersionNotFound);
        }
        sqlx::query(
            "UPDATE ai_prompt_template_versions SET status = 'draft', updated_at = now() WHERE prompt_id = $1 AND status = 'active'",
        )
        .bind(prompt_id.trim())
        .execute(&mut *tx)
        .await?;
        sqlx::query(
            "UPDATE ai_prompt_template_versions SET status = 'active', updated_at = now() WHERE prompt_version_id = $1",
        )
        .bind(request.prompt_version_id.trim())
        .execute(&mut *tx)
        .await?;
        let row = sqlx::query(
            r#"
            UPDATE ai_prompt_templates
            SET
                active_version_id = $2,
                updated_at = now()
            WHERE prompt_id = $1
            RETURNING
                prompt_id,
                name,
                entity_scope,
                capability_slot,
                description,
                is_system,
                active_version_id,
                metadata,
                created_at,
                updated_at
            "#,
        )
        .bind(prompt_id.trim())
        .bind(request.prompt_version_id.trim())
        .fetch_one(&mut *tx)
        .await?;
        tx.commit().await?;

        row_to_prompt(row)
    }

    pub async fn test_prompt(
        &self,
        prompt_id: &str,
        request: &AiPromptTestRequest,
        actor_id: &str,
    ) -> Result<AiPromptEvalRun, AiControlCenterError> {
        validate_non_empty("prompt_id", prompt_id)?;
        validate_non_empty("actor_id", actor_id)?;
        validate_non_empty("prompt_version_id", &request.prompt_version_id)?;
        validate_non_empty("provider_id", &request.provider_id)?;
        validate_non_empty("model_key", &request.model_key)?;
        let version = self
            .prompt_version(prompt_id, &request.prompt_version_id)
            .await?
            .ok_or(AiControlCenterError::PromptVersionNotFound)?;
        let _model = self
            .model(&request.provider_id, &request.model_key)
            .await?
            .ok_or(AiControlCenterError::ModelNotFound)?;
        let variables = object_value(request.variables.clone(), "variables")?;
        reject_secret_like_json(&Value::Object(variables.clone()))?;
        let source_refs = request.source_refs.clone().unwrap_or_default();
        let rendered = render_prompt(&version.body_template, &variables);
        let output_text = format!("Prompt studio preview\n\n{rendered}");
        let eval_run_id = format!(
            "prompt-eval:{}:{}",
            prompt_id.trim(),
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        );
        let row = sqlx::query(
            r#"
            INSERT INTO ai_prompt_eval_runs (
                eval_run_id,
                prompt_id,
                prompt_version_id,
                provider_id,
                model_key,
                source_refs,
                variables,
                output_text,
                score,
                notes,
                actor_id
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING
                eval_run_id,
                prompt_id,
                prompt_version_id,
                provider_id,
                model_key,
                source_refs,
                variables,
                output_text,
                score,
                notes,
                actor_id,
                created_at
            "#,
        )
        .bind(eval_run_id)
        .bind(prompt_id.trim())
        .bind(request.prompt_version_id.trim())
        .bind(request.provider_id.trim())
        .bind(request.model_key.trim())
        .bind(json!(source_refs))
        .bind(Value::Object(variables))
        .bind(output_text)
        .bind(request.score)
        .bind(request.notes.as_deref().map(str::trim))
        .bind(actor_id.trim())
        .fetch_one(&self.pool)
        .await?;

        row_to_eval_run(row)
    }

    async fn prompt(
        &self,
        prompt_id: &str,
    ) -> Result<Option<AiPromptTemplate>, AiControlCenterError> {
        let row = sqlx::query(
            r#"
            SELECT
                prompt_id,
                name,
                entity_scope,
                capability_slot,
                description,
                is_system,
                active_version_id,
                metadata,
                created_at,
                updated_at
            FROM ai_prompt_templates
            WHERE prompt_id = $1
            "#,
        )
        .bind(prompt_id.trim())
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_prompt).transpose()
    }

    async fn prompt_version(
        &self,
        prompt_id: &str,
        prompt_version_id: &str,
    ) -> Result<Option<AiPromptVersion>, AiControlCenterError> {
        let row = sqlx::query(
            r#"
            SELECT
                prompt_version_id,
                prompt_id,
                version_label,
                body_template,
                variables,
                status,
                created_by_actor_id,
                created_at,
                updated_at
            FROM ai_prompt_template_versions
            WHERE prompt_id = $1 AND prompt_version_id = $2
            "#,
        )
        .bind(prompt_id.trim())
        .bind(prompt_version_id.trim())
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_prompt_version).transpose()
    }

    pub(super) async fn list_prompt_eval_runs(
        &self,
        limit: i64,
    ) -> Result<Vec<AiPromptEvalRun>, AiControlCenterError> {
        let rows = sqlx::query(
            r#"
            SELECT
                eval_run_id,
                prompt_id,
                prompt_version_id,
                provider_id,
                model_key,
                source_refs,
                variables,
                output_text,
                score,
                notes,
                actor_id,
                created_at
            FROM ai_prompt_eval_runs
            ORDER BY created_at DESC, eval_run_id ASC
            LIMIT $1
            "#,
        )
        .bind(limit.clamp(1, 100))
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_eval_run).collect()
    }
}
