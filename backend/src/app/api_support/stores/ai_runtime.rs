use super::super::*;
use super::ai_routing::ai_model_routing;
use super::database::database_pool;
use crate::ai::control_center::AiControlCenterStore;
use crate::domains::signal_hub::{SignalHubError, SignalHubStore};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

const AI_REQUEST_RUNTIME: &str = "ai_request_runtime";

#[derive(Clone)]
struct PersonaProjectionAiPersonaAttributionPort {
    pool: sqlx::postgres::PgPool,
}

impl PersonaProjectionAiPersonaAttributionPort {
    fn new(pool: sqlx::postgres::PgPool) -> Self {
        Self { pool }
    }
}

impl crate::ai::core::AiPersonaAttributionPort for PersonaProjectionAiPersonaAttributionPort {
    fn upsert_ai_agent_persona<'a>(
        &'a self,
        agent_id: &'a str,
        display_name: &'a str,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<
                        crate::ai::core::AiAgentPersonaAttribution,
                        crate::ai::core::AiPersonaAttributionError,
                    >,
                > + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            let persona =
                crate::domains::personas::api::PersonaProjectionStore::new(self.pool.clone())
                    .upsert_ai_agent_persona(agent_id, display_name)
                    .await
                    .map_err(|error| {
                        crate::ai::core::AiPersonaAttributionError::Store(error.to_string())
                    })?;

            Ok(crate::ai::core::AiAgentPersonaAttribution {
                persona_id: persona.persona_id,
                persona_type: persona.persona_type.as_str(),
                persona_email: persona.email_address.ok_or_else(|| {
                    crate::ai::core::AiPersonaAttributionError::Store(
                        "ai agent persona is missing email_address".to_owned(),
                    )
                })?,
            })
        })
    }

    fn owner_persona_id<'a>(
        &'a self,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<Option<String>, crate::ai::core::AiPersonaAttributionError>>
                + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            Ok(
                crate::domains::personas::api::PersonaProjectionStore::new(self.pool.clone())
                    .owner_persona()
                    .await
                    .map_err(|error| {
                        crate::ai::core::AiPersonaAttributionError::Store(error.to_string())
                    })?
                    .map(|persona| persona.persona_id),
            )
        })
    }
}

pub(crate) fn ai_run_store(state: &AppState) -> Result<crate::ai::core::AiRunStore, ApiError> {
    Ok(crate::ai::core::AiRunStore::new(database_pool(state)?))
}

pub(crate) async fn ai_service(state: &AppState) -> Result<AiService, ApiError> {
    let pool = database_pool(state)?;
    let runtime_settings = ai_runtime_settings(state).await?;
    let model_routing = ai_model_routing(state, &runtime_settings).await?;
    let runtime = ai_runtime_client(state, &runtime_settings)?;
    let hub = ai_hub_from_runtime_client(runtime, model_routing);

    Ok(AiService::new_with_hub(pool.clone(), hub)
        .with_persona_attribution(ai_persona_attribution_port_from_pool(pool)))
}

pub(crate) async fn ai_requests_allowed(state: &AppState) -> Result<bool, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Ok(true);
    };

    SignalHubStore::new(pool.clone())
        .restore_system_sources()
        .await?;
    crate::platform::events::runtime_allows_processing(
        pool,
        "ai",
        AI_REQUEST_RUNTIME,
        &serde_json::json!({
            "label": "AI request runtime",
            "scope": "runtime",
        }),
    )
    .await
    .map_err(SignalHubError::from)
    .map_err(ApiError::from)
}

pub(crate) fn ai_persona_attribution_port_from_pool(
    pool: sqlx::postgres::PgPool,
) -> crate::ai::core::SharedAiPersonaAttributionPort {
    Arc::new(PersonaProjectionAiPersonaAttributionPort::new(pool))
        as crate::ai::core::SharedAiPersonaAttributionPort
}

pub(crate) fn ai_persona_attribution_port_optional(
    state: &AppState,
) -> Option<crate::ai::core::SharedAiPersonaAttributionPort> {
    state
        .database
        .pool()
        .map(|pool| ai_persona_attribution_port_from_pool(pool.clone()))
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
                ApiError::from(AiRuntimeError::OmniRoute(OmniRouteError::MissingApiKey))
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

pub(crate) fn ai_hub_from_runtime_client(
    runtime: AiRuntimeClient,
    model_routing: AiModelRouting,
) -> crate::ai::hub::SharedAiHub {
    crate::ai::hub::AiHub::shared(
        Arc::new(runtime) as crate::platform::ai_runtime::SharedAiRuntimePort,
        model_routing,
    )
}

pub(crate) fn ai_hub_from_runtime_client_with_store(
    runtime: AiRuntimeClient,
    model_routing: AiModelRouting,
    store: AiControlCenterStore,
) -> crate::ai::hub::SharedAiHub {
    crate::ai::hub::AiHub::shared_with_usage_recorder(
        Arc::new(runtime) as crate::platform::ai_runtime::SharedAiRuntimePort,
        model_routing,
        Arc::new(store) as crate::ai::hub::SharedAiHubUsageRecorder,
    )
}

pub(crate) async fn ai_hub_optional(
    state: &AppState,
) -> Result<Option<crate::ai::hub::SharedAiHub>, ApiError> {
    if !ai_requests_allowed(state).await? {
        return Ok(None);
    }

    let settings = ai_runtime_settings(state).await?;
    let model_routing = match ai_model_routing(state, &settings).await {
        Ok(model_routing) => model_routing,
        Err(error) => {
            tracing::warn!(error = ?error, "AI Hub routing unavailable; helper service disabled");
            return Ok(None);
        }
    };
    match ai_runtime_client(state, &settings) {
        Ok(runtime) => {
            let Some(pool) = state.database.pool() else {
                return Ok(Some(ai_hub_from_runtime_client(runtime, model_routing)));
            };
            Ok(Some(ai_hub_from_runtime_client_with_store(
                runtime,
                model_routing,
                AiControlCenterStore::new(pool.clone()),
            )))
        }
        Err(error) => {
            tracing::warn!(error = ?error, "AI Hub runtime construction failed; helper service disabled");
            Ok(None)
        }
    }
}

pub(crate) async fn email_multilingual_service(
    state: &AppState,
) -> Result<crate::domains::communications::multilingual::MultilingualService, ApiError> {
    Ok(
        crate::domains::communications::multilingual::MultilingualService::new(
            ai_hub_optional(state).await?,
        ),
    )
}

pub(crate) async fn email_ai_reply_service(
    state: &AppState,
) -> Result<crate::domains::communications::ai_reply::AiReplyService, ApiError> {
    Ok(
        crate::domains::communications::ai_reply::AiReplyService::new(
            ai_hub_optional(state).await?,
        ),
    )
}
