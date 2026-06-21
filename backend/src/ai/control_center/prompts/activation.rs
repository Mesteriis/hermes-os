use super::super::errors::AiControlCenterError;
use super::super::evidence::{
    capture_prompt_template_observation, capture_prompt_version_observation,
};
use super::super::models::{AiPromptActivateRequest, AiPromptTemplate, AiPromptVersion};
use super::super::rows::{row_to_prompt, row_to_prompt_version};
use super::super::store::AiControlCenterStore;
use super::super::validation::validate_non_empty;

impl AiControlCenterStore {
    pub async fn activate_prompt_version(
        &self,
        prompt_id: &str,
        request: &AiPromptActivateRequest,
        actor_id: &str,
    ) -> Result<AiPromptTemplate, AiControlCenterError> {
        validate_non_empty("prompt_id", prompt_id)?;
        validate_non_empty("prompt_version_id", &request.prompt_version_id)?;
        validate_non_empty("actor_id", actor_id)?;
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
        let version_row = sqlx::query(
            "UPDATE ai_prompt_template_versions SET status = 'active', updated_at = now() WHERE prompt_version_id = $1",
        )
        .bind(request.prompt_version_id.trim())
        .execute(&mut *tx)
        .await?;
        if version_row.rows_affected() == 0 {
            return Err(AiControlCenterError::PromptVersionNotFound);
        }
        let active_version_row = sqlx::query(
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
            WHERE prompt_version_id = $1
            "#,
        )
        .bind(request.prompt_version_id.trim())
        .fetch_one(&mut *tx)
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
        let active_version: AiPromptVersion = row_to_prompt_version(active_version_row)?;
        let prompt = row_to_prompt(row)?;
        capture_prompt_version_observation(&mut tx, &active_version, "activate", actor_id.trim())
            .await?;
        capture_prompt_template_observation(&mut tx, &prompt, "activate", actor_id.trim()).await?;
        tx.commit().await?;

        Ok(prompt)
    }
}
