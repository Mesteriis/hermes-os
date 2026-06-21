use chrono::Utc;
use serde_json::json;

use super::super::errors::AiControlCenterError;
use super::super::evidence::capture_prompt_version_observation;
use super::super::models::{AiPromptVersion, AiPromptVersionCreateRequest};
use super::super::rows::row_to_prompt_version;
use super::super::store::AiControlCenterStore;
use super::super::validation::{slug_id, string_array_value, validate_non_empty};

impl AiControlCenterStore {
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
        let mut transaction = self.pool.begin().await?;
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
        .fetch_one(&mut *transaction)
        .await?;

        let version = row_to_prompt_version(row)?;
        capture_prompt_version_observation(&mut transaction, &version, "create", actor_id.trim())
            .await?;
        transaction.commit().await?;
        Ok(version)
    }
}
