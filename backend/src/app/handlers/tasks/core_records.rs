use axum::Json;
use axum::extract::{Path, State};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::app::{ApiError, AppState};
use crate::domains::tasks::core::{
    ExternalTaskIdentity, ExternalTaskIdentityStore, TaskChecklist, TaskChecklistStore,
    TaskContextPack, TaskContextPackStore, TaskEvidence, TaskEvidenceStore, TaskRelation,
    TaskRelationStore, TaskSubtask, TaskSubtaskStore,
};
use crate::domains::tasks::service::TaskCommandService;

use super::support::database_pool;

pub(crate) async fn get_task_context_pack(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = database_pool(&state)?;
    let pack = TaskContextPackStore::new(pool)
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
    let pack = TaskContextPackStore::new(pool)
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
    let items = TaskEvidenceStore::new(pool)
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
    let items = TaskRelationStore::new(pool)
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
    let relation = TaskCommandService::new(pool)
        .add_relation_manual(
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
    let checklist = TaskChecklistStore::new(pool)
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
    let items = TaskSubtaskStore::new(pool)
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
    let items = ExternalTaskIdentityStore::new(pool)
        .list(&task_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(ExtIdentitiesResponse { items }))
}
