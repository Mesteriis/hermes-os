use super::super::errors::AiControlCenterError;
use super::super::models::AiPromptEvalRun;
use super::super::rows::row_to_eval_run;
use super::super::store::AiControlCenterStore;

impl AiControlCenterStore {
    pub(in crate::ai::control_center) async fn list_prompt_eval_runs(
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
