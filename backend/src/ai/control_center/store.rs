use sqlx::postgres::PgPool;

use super::errors::AiControlCenterError;
use super::models::AiSettingsOverviewResponse;
use super::presets::{capability_slots, provider_presets};

#[derive(Clone)]
pub struct AiControlCenterStore {
    pub(super) pool: PgPool,
}

impl AiControlCenterStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn overview(&self) -> Result<AiSettingsOverviewResponse, AiControlCenterError> {
        Ok(AiSettingsOverviewResponse {
            providers: self.list_providers().await?,
            models: self.list_models().await?,
            routes: self.list_model_routes().await?,
            prompts: self.list_prompts().await?,
            eval_runs: self.list_prompt_eval_runs(25).await?,
            capability_slots: capability_slots(),
            provider_presets: provider_presets(),
        })
    }
}
