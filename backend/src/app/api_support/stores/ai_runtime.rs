use super::super::*;
use super::ai_routing::ai_model_routing;
use super::database::database_pool;

pub(crate) fn ai_run_store(state: &AppState) -> Result<crate::ai::core::AiRunStore, ApiError> {
    Ok(crate::ai::core::AiRunStore::new(database_pool(state)?))
}

pub(crate) async fn ai_service(state: &AppState) -> Result<AiService, ApiError> {
    let pool = database_pool(state)?;
    let runtime_settings = ai_runtime_settings(state).await?;
    let model_routing = ai_model_routing(state, &runtime_settings).await?;
    let runtime = ai_runtime_client(state, &runtime_settings)?;

    Ok(AiService::new_with_routing(pool, runtime, model_routing))
}

pub(crate) async fn ai_runtime_settings(state: &AppState) -> Result<AiRuntimeSettings, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Ok(AiRuntimeSettings::from_config(&state.config));
    };

    Ok(ApplicationSettingsStore::new(pool.clone())
        .ai_runtime_settings(&state.config)
        .await?)
}

pub(crate) fn ai_runtime_client(
    state: &AppState,
    settings: &AiRuntimeSettings,
) -> Result<AiRuntimeClient, ApiError> {
    match settings.provider {
        AiRuntimeProvider::Ollama => Ok(AiRuntimeClient::Ollama(OllamaClient::new(
            OllamaClientConfig::new(
                &settings.base_url,
                &settings.chat_model,
                &settings.embedding_model,
            )
            .with_timeout_seconds(settings.timeout_seconds),
        )?)),
        AiRuntimeProvider::OmniRoute => {
            let api_key = state.config.omniroute_api_key().cloned().ok_or_else(|| {
                ApiError::Ai(AiError::Runtime(AiRuntimeError::OmniRoute(
                    OmniRouteError::MissingApiKey,
                )))
            })?;
            Ok(AiRuntimeClient::OmniRoute(OmniRouteClient::new(
                OmniRouteClientConfig::new(
                    &settings.base_url,
                    &settings.chat_model,
                    &settings.embedding_model,
                    api_key,
                )
                .with_timeout_seconds(settings.timeout_seconds),
            )?))
        }
    }
}

pub(crate) async fn email_multilingual_service(
    state: &AppState,
) -> Result<crate::domains::communications::multilingual::MultilingualService, ApiError> {
    let settings = ai_runtime_settings(state).await?;
    Ok(
        crate::domains::communications::multilingual::MultilingualService::new(
            ai_runtime_client(state, &settings).ok(),
        ),
    )
}

pub(crate) async fn email_ai_reply_service(
    state: &AppState,
) -> Result<crate::domains::communications::ai_reply::AiReplyService, ApiError> {
    let settings = ai_runtime_settings(state).await?;
    Ok(
        crate::domains::communications::ai_reply::AiReplyService::new(
            ai_runtime_client(state, &settings).ok(),
        ),
    )
}
