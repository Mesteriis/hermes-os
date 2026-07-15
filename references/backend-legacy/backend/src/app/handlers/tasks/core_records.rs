use axum::Json;
use axum::extract::{Path, State};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::app::error::types::ApiError;
use crate::app::state::AppState;
use crate::application::task_relationship::TaskRelationshipApplicationService;
use crate::domains::tasks::command_service::TaskCommandService;
use crate::domains::tasks::core::checklists::{TaskChecklist, TaskChecklistStore};
use crate::domains::tasks::core::context_packs::{TaskContextPack, TaskContextPackStore};
use crate::domains::tasks::core::evidence::{TaskEvidence, TaskEvidenceStore};
use crate::domains::tasks::core::external_identities::{
    ExternalTaskIdentity, ExternalTaskIdentityStore,
};
use crate::domains::tasks::core::relations::{TaskRelation, TaskRelationStore};
use crate::domains::tasks::core::subtasks::{TaskSubtask, TaskSubtaskStore};

use super::support::database_pool;

pub(crate) async fn get_task_context_pack(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = database_pool(&state)?;
    let pack =
        crate::app::api_support::stores::domain_stores::app_store::<TaskContextPackStore>(pool)
            .get(&task_id)
            .await
            .map_err(ApiError::from)?;
    Ok(Json(serde_json::to_value(&pack).unwrap_or_default()))
}

#[derive(Deserialize)]
pub(crate) struct UpsertContextPackRequest {
    summary: Option<String>,
    open_questions: Option<Value>,
    blockers: Option<Value>,
    risks: Option<Value>,
    suggested_next_action: Option<String>,
}

pub(crate) async fn post_task_context_pack(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
    Json(req): Json<UpsertContextPackRequest>,
) -> Result<Json<TaskContextPack>, ApiError> {
    let pool = database_pool(&state)?;
    let pack =
        crate::app::api_support::stores::domain_stores::app_store::<TaskContextPackStore>(pool)
            .upsert(
                &task_id,
                req.summary.as_deref(),
                req.open_questions.unwrap_or(json!([])),
                req.blockers.unwrap_or(json!([])),
                req.risks.unwrap_or(json!([])),
                req.suggested_next_action.as_deref(),
            )
            .await
            .map_err(ApiError::from)?;
    Ok(Json(pack))
}

#[derive(Serialize)]
pub(crate) struct TaskEvidenceResponse {
    items: Vec<TaskEvidence>,
}

pub(crate) async fn get_task_evidence(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<TaskEvidenceResponse>, ApiError> {
    let pool = database_pool(&state)?;
    let items =
        crate::app::api_support::stores::domain_stores::app_store::<TaskEvidenceStore>(pool)
            .list(&task_id)
            .await
            .map_err(ApiError::from)?;
    Ok(Json(TaskEvidenceResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewEvidenceRequest {
    source_type: Option<String>,
    source_id: Option<String>,
    quote: Option<String>,
    confidence: Option<f64>,
}

pub(crate) async fn post_task_evidence(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
    Json(req): Json<NewEvidenceRequest>,
) -> Result<Json<TaskEvidence>, ApiError> {
    let pool = database_pool(&state)?;
    let evidence = TaskCommandService::new(pool)
        .add_evidence(
            &task_id,
            req.source_type.as_deref(),
            req.source_id.as_deref(),
            req.quote,
            req.confidence,
        )
        .await?;
    Ok(Json(evidence))
}

#[derive(Serialize)]
pub(crate) struct TaskRelationsResponse {
    items: Vec<TaskRelation>,
}

pub(crate) async fn get_task_relations(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<TaskRelationsResponse>, ApiError> {
    let pool = database_pool(&state)?;
    let items =
        crate::app::api_support::stores::domain_stores::app_store::<TaskRelationStore>(pool)
            .list(&task_id)
            .await
            .map_err(ApiError::from)?;
    Ok(Json(TaskRelationsResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewRelationReq {
    entity_type: String,
    entity_id: String,
    relation_type: String,
}

pub(crate) async fn post_task_relation(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
    Json(req): Json<NewRelationReq>,
) -> Result<Json<TaskRelation>, ApiError> {
    let pool = database_pool(&state)?;
    let relation = TaskRelationshipApplicationService::new(pool)
        .add_manual(
            &task_id,
            &req.entity_type,
            &req.entity_id,
            &req.relation_type,
        )
        .await?;
    Ok(Json(relation))
}

pub(crate) async fn get_task_checklist(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = database_pool(&state)?;
    let checklist =
        crate::app::api_support::stores::domain_stores::app_store::<TaskChecklistStore>(pool)
            .get(&task_id)
            .await
            .map_err(ApiError::from)?;
    Ok(Json(serde_json::to_value(&checklist).unwrap_or_default()))
}

#[derive(Deserialize)]
pub(crate) struct SetChecklistReq {
    items: Value,
    source: Option<String>,
}

pub(crate) async fn post_task_checklist(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
    Json(req): Json<SetChecklistReq>,
) -> Result<Json<TaskChecklist>, ApiError> {
    let items = req.items;
    let pool = database_pool(&state)?;
    let checklist = TaskCommandService::new(pool)
        .set_checklist_manual(&task_id, items, req.source.as_deref())
        .await?;
    Ok(Json(checklist))
}

#[derive(Serialize)]
pub(crate) struct TaskSubtasksResponse {
    items: Vec<TaskSubtask>,
}

pub(crate) async fn get_task_subtasks(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<TaskSubtasksResponse>, ApiError> {
    let pool = database_pool(&state)?;
    let items = crate::app::api_support::stores::domain_stores::app_store::<TaskSubtaskStore>(pool)
        .list(&task_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(TaskSubtasksResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewSubtaskReq {
    child_task_id: String,
    sort_order: Option<i32>,
}

pub(crate) async fn post_task_subtask(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
    Json(req): Json<NewSubtaskReq>,
) -> Result<Json<TaskSubtask>, ApiError> {
    let pool = database_pool(&state)?;
    let subtask = TaskCommandService::new(pool)
        .add_subtask_manual(&task_id, &req.child_task_id, req.sort_order.unwrap_or(0))
        .await?;
    Ok(Json(subtask))
}

#[derive(Serialize)]
pub(crate) struct ExtIdentitiesResponse {
    items: Vec<ExternalTaskIdentity>,
}

pub(crate) async fn get_task_external(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<ExtIdentitiesResponse>, ApiError> {
    let pool = database_pool(&state)?;
    let items = crate::app::api_support::stores::domain_stores::app_store::<
        ExternalTaskIdentityStore,
    >(pool)
    .list(&task_id)
    .await
    .map_err(ApiError::from)?;
    Ok(Json(ExtIdentitiesResponse { items }))
}
