use axum::Json;
use axum::extract::State;
use serde::{Deserialize, Serialize};

use crate::app::error::types::ApiError;
use crate::app::state::AppState;
use crate::domains::tasks::core::providers::TaskProviderAccount;

use super::support::database_pool;

#[derive(Serialize)]
pub(crate) struct TaskProvidersResponse {
    items: Vec<TaskProviderAccount>,
}

pub(crate) async fn get_task_providers(
    State(state): State<AppState>,
) -> Result<Json<TaskProvidersResponse>, ApiError> {
    let pool = database_pool(&state)?;
    let items = crate::app::api_support::stores::domain_stores::app_store::<
        crate::domains::tasks::core::provider_store::TaskProviderStore,
    >(pool)
    .list()
    .await
    .map_err(ApiError::from)?;
    Ok(Json(TaskProvidersResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewTaskProviderReq {
    provider: String,
    account_name: String,
}

pub(crate) async fn post_task_provider(
    State(state): State<AppState>,
    Json(req): Json<NewTaskProviderReq>,
) -> Result<Json<TaskProviderAccount>, ApiError> {
    let pool = database_pool(&state)?;
    let provider = crate::app::api_support::stores::domain_stores::app_store::<
        crate::domains::tasks::core::provider_store::TaskProviderStore,
    >(pool)
    .create(&req.provider, &req.account_name)
    .await
    .map_err(ApiError::from)?;
    Ok(Json(provider))
}
