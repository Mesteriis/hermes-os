use serde_json::{Value, json};

use super::super::errors::AiControlCenterError;
use super::super::models::{AiPromptCreateRequest, AiPromptTemplate};
use super::super::rows::row_to_prompt;
use super::super::store::AiControlCenterStore;
use super::super::validation::{
    object_value, reject_secret_like_json, slug_id, validate_non_empty,
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
}
