use axum::Json;
use axum::extract::{Path, State};
use axum::http::HeaderMap;

use crate::ai::control_center::{
    AiModelRoute, AiModelRouteUpdateRequest, AiPromptActivateRequest, AiPromptCreateRequest,
    AiPromptEvalRun, AiPromptTemplate, AiPromptTestRequest, AiPromptVersion,
    AiPromptVersionCreateRequest, AiProviderAccount, AiProviderCommandKind,
    AiProviderCommandResponse, AiProviderConsentRequest, AiProviderCreateRequest,
    AiProviderPatchRequest, AiSettingsOverviewResponse, store_api_key_in_host_vault,
};
use crate::app::{ApiError, AppState};

use super::helpers::{ai_control_center_store, request_actor_id};
use super::models::{AiModelListResponse, AiPromptListResponse, AiProviderListResponse};

pub(crate) async fn get_ai_settings_overview(
    State(state): State<AppState>,
) -> Result<Json<AiSettingsOverviewResponse>, ApiError> {
    Ok(Json(ai_control_center_store(&state)?.overview().await?))
}

pub(crate) async fn get_ai_providers(
    State(state): State<AppState>,
) -> Result<Json<AiProviderListResponse>, ApiError> {
    Ok(Json(AiProviderListResponse {
        items: ai_control_center_store(&state)?.list_providers().await?,
    }))
}

pub(crate) async fn post_ai_provider(
    State(state): State<AppState>,
    Json(request): Json<AiProviderCreateRequest>,
) -> Result<Json<AiProviderAccount>, ApiError> {
    let store = ai_control_center_store(&state)?;
    let provider = store.create_provider(&request).await?;
    if let Some(api_key) = request
        .api_key
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        let Some(pool) = state.database.pool() else {
            return Err(ApiError::DatabaseNotConfigured);
        };
        store_api_key_in_host_vault(pool, &state.vault, &provider.provider_id, api_key).await?;
        let Some(provider) = store.provider(&provider.provider_id).await? else {
            return Err(ApiError::NotFound);
        };
        return Ok(Json(provider));
    }

    Ok(Json(provider))
}

pub(crate) async fn patch_ai_provider(
    State(state): State<AppState>,
    Path(provider_id): Path<String>,
    Json(request): Json<AiProviderPatchRequest>,
) -> Result<Json<AiProviderAccount>, ApiError> {
    let store = ai_control_center_store(&state)?;
    let provider = store.update_provider(&provider_id, &request).await?;
    if let Some(api_key) = request
        .api_key
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        let Some(pool) = state.database.pool() else {
            return Err(ApiError::DatabaseNotConfigured);
        };
        store_api_key_in_host_vault(pool, &state.vault, &provider.provider_id, api_key).await?;
        let Some(provider) = store.provider(&provider.provider_id).await? else {
            return Err(ApiError::NotFound);
        };
        return Ok(Json(provider));
    }

    Ok(Json(provider))
}

pub(crate) async fn post_ai_provider_test(
    State(state): State<AppState>,
    Path(provider_id): Path<String>,
) -> Result<Json<AiProviderCommandResponse>, ApiError> {
    Ok(Json(
        ai_control_center_store(&state)?
            .provider_command(&provider_id, AiProviderCommandKind::Test)
            .await?,
    ))
}

pub(crate) async fn post_ai_provider_sync_models(
    State(state): State<AppState>,
    Path(provider_id): Path<String>,
) -> Result<Json<AiProviderCommandResponse>, ApiError> {
    Ok(Json(
        ai_control_center_store(&state)?
            .provider_command(&provider_id, AiProviderCommandKind::SyncModels)
            .await?,
    ))
}

pub(crate) async fn post_ai_provider_consent(
    State(state): State<AppState>,
    Path(provider_id): Path<String>,
    Json(request): Json<AiProviderConsentRequest>,
) -> Result<Json<AiProviderAccount>, ApiError> {
    Ok(Json(
        ai_control_center_store(&state)?
            .record_consent(&provider_id, &request)
            .await?,
    ))
}

pub(crate) async fn get_ai_models(
    State(state): State<AppState>,
) -> Result<Json<AiModelListResponse>, ApiError> {
    Ok(Json(AiModelListResponse {
        items: ai_control_center_store(&state)?.list_models().await?,
    }))
}

pub(crate) async fn put_ai_model_route(
    State(state): State<AppState>,
    Path(slot): Path<String>,
    Json(request): Json<AiModelRouteUpdateRequest>,
) -> Result<Json<AiModelRoute>, ApiError> {
    Ok(Json(
        ai_control_center_store(&state)?
            .put_model_route(&slot, &request)
            .await?,
    ))
}

pub(crate) async fn get_ai_prompts(
    State(state): State<AppState>,
) -> Result<Json<AiPromptListResponse>, ApiError> {
    Ok(Json(AiPromptListResponse {
        items: ai_control_center_store(&state)?.list_prompts().await?,
    }))
}

pub(crate) async fn post_ai_prompt(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<AiPromptCreateRequest>,
) -> Result<Json<AiPromptTemplate>, ApiError> {
    Ok(Json(
        ai_control_center_store(&state)?
            .create_prompt(&request, &request_actor_id(&headers))
            .await?,
    ))
}

pub(crate) async fn post_ai_prompt_version(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(prompt_id): Path<String>,
    Json(request): Json<AiPromptVersionCreateRequest>,
) -> Result<Json<AiPromptVersion>, ApiError> {
    Ok(Json(
        ai_control_center_store(&state)?
            .create_prompt_version(&prompt_id, &request, &request_actor_id(&headers))
            .await?,
    ))
}

pub(crate) async fn post_ai_prompt_activate(
    State(state): State<AppState>,
    Path(prompt_id): Path<String>,
    Json(request): Json<AiPromptActivateRequest>,
) -> Result<Json<AiPromptTemplate>, ApiError> {
    Ok(Json(
        ai_control_center_store(&state)?
            .activate_prompt_version(&prompt_id, &request)
            .await?,
    ))
}

pub(crate) async fn post_ai_prompt_test(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(prompt_id): Path<String>,
    Json(request): Json<AiPromptTestRequest>,
) -> Result<Json<AiPromptEvalRun>, ApiError> {
    Ok(Json(
        ai_control_center_store(&state)?
            .test_prompt(&prompt_id, &request, &request_actor_id(&headers))
            .await?,
    ))
}
