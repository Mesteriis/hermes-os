use axum::Json;
use axum::extract::{Path, Query, State};

use crate::ai::core::{
    AI_EMBEDDING_DIMENSION, AiAgentListResponse, AiAgentRun, AiAnswerRequest, AiMeetingPrepRequest,
    AiStatusResponse, AiTaskCandidateRefreshRequest, v3_agents,
};
use crate::app::api_support::{
    AiRunListResponse, AiRunsQuery, ai_persona_attribution_port_optional, ai_run_store,
    ai_runtime_client, ai_runtime_settings, ai_service,
};
use crate::app::{ApiError, AppState};
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

    if let Some(persona_attribution) = ai_persona_attribution_port_optional(&state) {
        for item in &mut items {
            let persona = persona_attribution
                .upsert_ai_agent_persona(item.agent_id, item.display_name)
                .await
                .map_err(crate::ai::core::AiError::from)?;
            item.persona_id = Some(persona.persona_id);
            item.persona_type = Some(persona.persona_type);
            item.persona_email = Some(persona.persona_email);
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
    ensure_ai_requests_allowed(&state).await?;
    let actor_id = "hermes-frontend".to_string();
    let service = ai_service(&state).await?;
    let response = service.answer(request, &actor_id).await?;

    Ok(Json(response))
}

pub(crate) async fn post_ai_task_candidates_refresh(
    State(state): State<AppState>,
    Json(request): Json<AiTaskCandidateRefreshRequest>,
) -> Result<Json<crate::ai::core::AiTaskCandidateRefreshResponse>, ApiError> {
    ensure_ai_requests_allowed(&state).await?;
    let actor_id = "hermes-frontend".to_string();
    let service = ai_service(&state).await?;
    let response = service.refresh_task_candidates(request, &actor_id).await?;

    Ok(Json(response))
}

pub(crate) async fn post_ai_meeting_prep(
    State(state): State<AppState>,
    Json(request): Json<AiMeetingPrepRequest>,
) -> Result<Json<crate::ai::core::AiMeetingPrepResponse>, ApiError> {
    ensure_ai_requests_allowed(&state).await?;
    let actor_id = "hermes-frontend".to_string();
    let service = ai_service(&state).await?;
    let response = service.meeting_prep(request, &actor_id).await?;

    Ok(Json(response))
}

async fn ensure_ai_requests_allowed(state: &AppState) -> Result<(), ApiError> {
    if crate::app::api_support::ai_requests_allowed(state).await? {
        return Ok(());
    }

    Err(ApiError::FailedPrecondition(
        "AI runtime is disabled by Signal Hub policy or runtime state".to_owned(),
    ))
}
