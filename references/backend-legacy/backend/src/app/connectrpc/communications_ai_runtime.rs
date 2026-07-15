use crate::ai::control_center::models::AiProviderAccount;
use crate::ai::control_center::store::AiControlCenterStore;
use crate::ai::hub::{AiHub, SharedAiHub, SharedAiHubUsageRecorder};
use crate::app::error::types::ApiError;
use crate::integrations::ai_runtime::{AiRuntimeClient, AiRuntimeError};
use crate::integrations::ollama::client::OllamaClient;
use crate::integrations::ollama::client::config::OllamaClientConfig;
use crate::integrations::omniroute::client::OmniRouteClient;
use crate::integrations::omniroute::client::config::OmniRouteClientConfig;
use crate::integrations::omniroute::client::error::OmniRouteError;
use crate::platform::ai_runtime::SharedAiRuntimePort;
use crate::platform::config::ai::AiRuntimeProvider;
use crate::platform::config::app_config::AppConfig;
use crate::platform::settings::ai_runtime::AiRuntimeSettings;
use crate::platform::settings::store::ApplicationSettingsStore;
use connectrpc::{ConnectError, ErrorCode};
use sqlx::postgres::PgPool;

const AI_REQUEST_RUNTIME: &str = "ai_request_runtime";

pub(super) fn provider_matches_runtime(
    provider: &AiProviderAccount,
    runtime_provider: AiRuntimeProvider,
) -> bool {
    match runtime_provider {
        AiRuntimeProvider::Ollama => {
            provider.provider_kind == "built_in" && provider.provider_key == "ollama"
        }
        AiRuntimeProvider::OmniRoute => {
            provider.provider_kind == "api" && provider.provider_key == "omniroute"
        }
    }
}

pub(super) fn build_hub(
    pool: &PgPool,
    config: &AppConfig,
    settings: &AiRuntimeSettings,
    model_routing: crate::ai::core::types::AiModelRouting,
) -> Option<SharedAiHub> {
    build_client(config, settings).ok().map(|runtime| {
        AiHub::shared_with_usage_recorder(
            std::sync::Arc::new(runtime) as SharedAiRuntimePort,
            model_routing,
            std::sync::Arc::new(AiControlCenterStore::new(pool.clone()))
                as SharedAiHubUsageRecorder,
        )
    })
}

pub(super) fn build_client(
    config: &AppConfig,
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
            let api_key = config.omniroute_api_key().cloned().ok_or_else(|| {
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

pub(super) async fn requests_allowed(pool: &PgPool) -> Result<bool, ApiError> {
    crate::domains::signal_hub::store::SignalHubStore::new(pool.clone())
        .restore_system_sources()
        .await?;
    crate::platform::events::runtime::runtime_allows_processing(
        pool,
        "ai",
        AI_REQUEST_RUNTIME,
        &serde_json::json!({
            "label": "AI request runtime",
            "scope": "runtime",
        }),
    )
    .await
    .map_err(crate::domains::signal_hub::store::SignalHubError::from)
    .map_err(ApiError::from)
}

pub(super) async fn model_routing(
    pool: &PgPool,
    settings: &AiRuntimeSettings,
) -> Result<crate::ai::core::types::AiModelRouting, ApiError> {
    let store = AiControlCenterStore::new(pool.clone());
    resolve_model_routing(&store, settings)
        .await
        .map_err(ApiError::from)
}

async fn resolve_model_routing(
    store: &AiControlCenterStore,
    settings: &AiRuntimeSettings,
) -> Result<
    crate::ai::core::types::AiModelRouting,
    crate::ai::control_center::errors::AiControlCenterError,
> {
    let default_chat = resolve_slot_model(store, settings, "default_chat").await?;
    let reasoning = resolve_slot_model(store, settings, "reasoning").await?;
    let summarization = resolve_slot_model(store, settings, "summarization").await?;
    let mail_intelligence = resolve_slot_model(store, settings, "mail_intelligence").await?;
    let reply_draft = resolve_slot_model(store, settings, "reply_draft").await?;
    let extraction = resolve_slot_model(store, settings, "extraction").await?;
    let embeddings = resolve_slot_model(store, settings, "embeddings").await?;
    let meeting_prep = resolve_slot_model(store, settings, "meeting_prep").await?;

    Ok(crate::ai::core::types::AiModelRouting {
        default_chat: default_chat.model_key.clone(),
        reasoning: reasoning.model_key.clone(),
        summarization: summarization.model_key.clone(),
        mail_intelligence: mail_intelligence.model_key.clone(),
        reply_draft: reply_draft.model_key.clone(),
        extraction: extraction.model_key.clone(),
        embeddings: embeddings.model_key.clone(),
        meeting_prep: meeting_prep.model_key.clone(),
        targets: vec![
            default_chat,
            reasoning,
            summarization,
            mail_intelligence,
            reply_draft,
            extraction,
            embeddings,
            meeting_prep,
        ],
    })
}

async fn resolve_slot_model(
    store: &AiControlCenterStore,
    settings: &AiRuntimeSettings,
    slot: &str,
) -> Result<
    crate::ai::core::types::AiModelRouteTarget,
    crate::ai::control_center::errors::AiControlCenterError,
> {
    let Some(route) = store.route_for_slot(slot).await? else {
        return Err(
            crate::ai::control_center::errors::AiControlCenterError::InvalidRequest(format!(
                "route_not_configured:{slot}: use Hub route settings"
            )),
        );
    };
    let Some(provider) = store.provider(&route.provider_id).await? else {
        return Err(
            crate::ai::control_center::errors::AiControlCenterError::InvalidRequest(format!(
                "route_provider_missing:{}",
                route.provider_id
            )),
        );
    };
    if !provider_matches_runtime(&provider, settings.provider) {
        return Err(
            crate::ai::control_center::errors::AiControlCenterError::InvalidRequest(
                "route_provider_mismatch: runtime and route provider kinds diverge".to_owned(),
            ),
        );
    }
    store
        .ensure_model_ready_for_private_context(&route.provider_id, &route.model_key)
        .await?;
    Ok(crate::ai::core::types::AiModelRouteTarget {
        capability_slot: slot.to_owned(),
        provider_id: route.provider_id,
        model_key: route.model_key,
    })
}

pub(super) async fn multilingual_service(
    pool: &PgPool,
    config: &AppConfig,
) -> Result<crate::domains::communications::multilingual::MultilingualService, ApiError> {
    Ok(
        crate::domains::communications::multilingual::MultilingualService::new(
            hub_optional(pool, config).await?,
        ),
    )
}

pub(super) async fn hub_optional(
    pool: &PgPool,
    config: &AppConfig,
) -> Result<Option<SharedAiHub>, ApiError> {
    if !requests_allowed(pool).await? {
        return Ok(None);
    }
    let settings = ApplicationSettingsStore::new(pool.clone())
        .ai_runtime_settings(config)
        .await?;
    match model_routing(pool, &settings).await {
        Ok(model_routing) => Ok(build_hub(pool, config, &settings, model_routing)),
        Err(error) => {
            tracing::warn!(error = ?error, "AI Hub routing unavailable for communications helper surface; continuing without AI runtime");
            Ok(None)
        }
    }
}

pub(super) async fn require_mail_content_egress(
    pool: &PgPool,
    config: &AppConfig,
    account_id: &str,
    kind: crate::app::api_support::stores::ai_runtime::MailAiContentEgressKind,
) -> Result<(), ConnectError> {
    let settings = ApplicationSettingsStore::new(pool.clone())
        .ai_runtime_settings(config)
        .await
        .map_err(|error| super::communications::api_error_connect_error(ApiError::from(error)))?;
    if crate::app::api_support::stores::ai_runtime::mail_ai_content_egress_allowed(
        pool,
        settings.provider,
        account_id,
        kind,
    )
    .await
    {
        return Ok(());
    }
    let setting_name = match kind {
        crate::app::api_support::stores::ai_runtime::MailAiContentEgressKind::Body => "body",
        crate::app::api_support::stores::ai_runtime::MailAiContentEgressKind::ExtractedText => {
            "extracted_text"
        }
    };
    Err(ConnectError::new(
        ErrorCode::FailedPrecondition,
        format!("external AI {setting_name} egress is disabled for this mail account"),
    ))
}
