use chrono::Utc;
use serde_json::{Value, json};

use super::super::errors::AiControlCenterError;
use super::super::models::{AiPromptEvalRun, AiPromptTestRequest};
use super::super::rows::row_to_eval_run;
use super::super::store::AiControlCenterStore;
use super::super::validation::{
    object_value, reject_secret_like_json, render_prompt, validate_non_empty,
};

impl AiControlCenterStore {
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
            .ensure_model_ready_for_private_context(&request.provider_id, &request.model_key)
            .await?;
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
}
