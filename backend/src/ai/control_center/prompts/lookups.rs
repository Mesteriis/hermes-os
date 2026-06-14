use super::super::errors::AiControlCenterError;
use super::super::models::{AiPromptTemplate, AiPromptVersion};
use super::super::rows::{row_to_prompt, row_to_prompt_version};
use super::super::store::AiControlCenterStore;

impl AiControlCenterStore {
    pub(super) async fn prompt(
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

    pub(super) async fn prompt_version(
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
}
