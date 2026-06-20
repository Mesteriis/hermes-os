use axum::Json;
use axum::extract::{Path, Query, State};

use crate::ai::core::{
    AI_EMBEDDING_DIMENSION, AiAgentListResponse, AiAgentRun, AiAnswerRequest, AiMeetingPrepRequest,
    AiStatusResponse, AiTaskCandidateRefreshRequest, v3_agents,
};
use crate::app::api_support::{
    AiRunListResponse, AiRunsQuery, ai_run_store, ai_runtime_client, ai_runtime_settings,
    ai_service,
};
use crate::app::{ApiError, AppState};
use crate::domains::persons::api::PersonProjectionStore;

pub(crate) async fn get_ai_status(
    State(state): State<AppState>,
) -> Result<Json<AiStatusResponse>, ApiError> {
    let runtime_settings = ai_runtime_settings(&state).await?;
    let runtime = ai_runtime_client(&state, &runtime_settings)?;
    let version = runtime.version().await;
    let models = runtime.models().await;
    let chat_model = runtime_settings.chat_model;
    let embedding_model = runtime_settings.embedding_model;
    let chat_model_available = models
        .as_ref()
        .map(|models| models.iter().any(|model| model == &chat_model))
        .unwrap_or(false);
    let embedding_model_available = models
        .as_ref()
        .map(|models| models.iter().any(|model| model == &embedding_model))
        .unwrap_or(false);

    Ok(Json(AiStatusResponse {
        runtime: runtime.runtime_name().to_owned(),
        status: if version.is_ok()
            && models.is_ok()
            && chat_model_available
            && embedding_model_available
        {
            "ok"
        } else {
            "unavailable"
        }
        .to_owned(),
        version: version.ok().flatten(),
        chat_model,
        embedding_model,
        embedding_dimension: AI_EMBEDDING_DIMENSION,
        chat_model_available,
        embedding_model_available,
    }))
}

pub(crate) async fn get_ai_agents(
    State(state): State<AppState>,
) -> Result<Json<AiAgentListResponse>, ApiError> {
    let runtime_settings = ai_runtime_settings(&state).await?;
    let mut items = v3_agents(&runtime_settings.chat_model);

    if let Some(pool) = state.database.pool() {
        let store = PersonProjectionStore::new(pool.clone());
        for item in &mut items {
            let persona = store
                .upsert_ai_agent_persona(item.agent_id, item.display_name)
                .await?;
            item.persona_id = Some(persona.person_id);
            item.persona_type = Some(persona.persona_type.as_str());
            item.persona_email = Some(persona.email_address);
        }
    }

    Ok(Json(AiAgentListResponse { items }))
}

pub(crate) async fn get_ai_runs(
    State(state): State<AppState>,
    Query(query): Query<AiRunsQuery>,
) -> Result<Json<AiRunListResponse>, ApiError> {
    let limit = query.limit.unwrap_or(25).clamp(1, 100);
    let runs = ai_run_store(&state)?.list_runs(limit).await?;

    Ok(Json(AiRunListResponse { items: runs }))
}

pub(crate) async fn get_ai_run(
    State(state): State<AppState>,
    Path(run_id): Path<String>,
) -> Result<Json<AiAgentRun>, ApiError> {
    let Some(run) = ai_run_store(&state)?.get_run(&run_id).await? else {
        return Err(ApiError::AiRunNotFound);
    };

    Ok(Json(run))
}

pub(crate) async fn post_ai_answer(
    State(state): State<AppState>,
    Json(request): Json<AiAnswerRequest>,
) -> Result<Json<crate::ai::core::AiAnswerResponse>, ApiError> {
    let actor_id = "hermes-frontend".to_string();
    let service = ai_service(&state).await?;
    let response = service.answer(request, &actor_id).await?;

    Ok(Json(response))
}

pub(crate) async fn post_ai_task_candidates_refresh(
    State(state): State<AppState>,
    Json(request): Json<AiTaskCandidateRefreshRequest>,
) -> Result<Json<crate::ai::core::AiTaskCandidateRefreshResponse>, ApiError> {
    let actor_id = "hermes-frontend".to_string();
    let service = ai_service(&state).await?;
    let response = service.refresh_task_candidates(request, &actor_id).await?;

    Ok(Json(response))
}

pub(crate) async fn post_ai_meeting_prep(
    State(state): State<AppState>,
    Json(request): Json<AiMeetingPrepRequest>,
) -> Result<Json<crate::ai::core::AiMeetingPrepResponse>, ApiError> {
    let actor_id = "hermes-frontend".to_string();
    let service = ai_service(&state).await?;
    let response = service.meeting_prep(request, &actor_id).await?;

    Ok(Json(response))
}
