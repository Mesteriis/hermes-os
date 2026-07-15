use axum::Json;
use axum::extract::{Path, State};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::app::error::types::ApiError;
use crate::app::state::AppState;
use crate::domains::tasks::rules::{TaskRule, TaskRuleStore, TaskTemplate, TaskTemplateStore};

use super::support::database_pool;

#[derive(Serialize)]
pub(crate) struct TaskRulesResponse {
    items: Vec<TaskRule>,
}

pub(crate) async fn get_task_rules(
    State(state): State<AppState>,
) -> Result<Json<TaskRulesResponse>, ApiError> {
    let pool = database_pool(&state)?;
    let items = crate::app::api_support::stores::domain_stores::app_store::<TaskRuleStore>(pool)
        .list()
        .await
        .map_err(ApiError::from)?;
    Ok(Json(TaskRulesResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewTaskRuleReq {
    name: String,
    description: Option<String>,
    dsl: Option<Value>,
    config: Option<Value>,
    rule_type: Option<String>,
    approval_mode: Option<String>,
}

pub(crate) async fn post_task_rule(
    State(state): State<AppState>,
    Json(req): Json<NewTaskRuleReq>,
) -> Result<Json<TaskRule>, ApiError> {
    let pool = database_pool(&state)?;
    let dsl = req.dsl.or(req.config).unwrap_or_else(|| json!({}));
    let description = req.description.or(req.rule_type);
    let rule = crate::app::api_support::stores::domain_stores::app_store::<TaskRuleStore>(pool)
        .create(
            &req.name,
            description.as_deref(),
            dsl,
            req.approval_mode.as_deref(),
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(rule))
}

pub(crate) async fn delete_task_rule(
    State(state): State<AppState>,
    Path(rule_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = database_pool(&state)?;
    crate::app::api_support::stores::domain_stores::app_store::<TaskRuleStore>(pool)
        .delete(&rule_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(json!({"deleted": true})))
}

#[derive(Serialize)]
pub(crate) struct TaskTemplatesResponse {
    items: Vec<TaskTemplate>,
}

pub(crate) async fn get_task_templates(
    State(state): State<AppState>,
) -> Result<Json<TaskTemplatesResponse>, ApiError> {
    let pool = database_pool(&state)?;
    let items =
        crate::app::api_support::stores::domain_stores::app_store::<TaskTemplateStore>(pool)
            .list()
            .await
            .map_err(ApiError::from)?;
    Ok(Json(TaskTemplatesResponse { items }))
}
