use std::sync::Arc;

use sqlx::postgres::PgPool;

use crate::platform::config::AppConfig;
use crate::platform::settings::AiRuntimeSettings;
use crate::vault::HostVault;

pub(super) async fn mail_ai_hub_optional(
    pool: &PgPool,
    config: &AppConfig,
    vault: &HostVault,
) -> Option<(crate::ai::hub::SharedAiHub, bool)> {
    let settings = match crate::platform::settings::ApplicationSettingsStore::new(pool.clone())
        .ai_runtime_settings(config)
        .await
    {
        Ok(settings) => settings,
        Err(error) => {
            tracing::warn!(error = %error, "mail AI pipeline settings lookup failed");
            AiRuntimeSettings::from_config(config)
        }
    };
    let store = crate::ai::control_center::AiControlCenterStore::new(pool.clone());
    let model_routing = match resolve_mail_ai_model_routing(&store).await {
        Ok(routing) => routing,
        Err(error) => {
            tracing::warn!(error = %error, "mail AI pipeline routing unavailable");
            return None;
        }
    };
    let mail_provider_id = match mail_ai_route_provider_id(&model_routing) {
        Some(provider_id) => provider_id,
        None => {
            tracing::warn!("mail AI pipeline route target is unavailable");
            return None;
        }
    };
    let provider = match store.provider(mail_provider_id).await {
        Ok(Some(provider)) => provider,
        Ok(None) => {
            tracing::warn!(
                provider_id = %mail_provider_id,
                "mail AI pipeline provider is missing"
            );
            return None;
        }
        Err(error) => {
            tracing::warn!(error = %error, "mail AI pipeline provider lookup failed");
            return None;
        }
    };
    let runtime =
        match mail_ai_runtime_client(pool, &store, vault, &settings, &provider, &model_routing)
            .await
        {
            Ok(runtime) => runtime,
            Err(error) => {
                tracing::warn!(error = %error, "mail AI pipeline runtime unavailable");
                return None;
            }
        };

    Some((
        crate::ai::hub::AiHub::shared_with_usage_recorder(
            Arc::new(runtime) as crate::platform::ai_runtime::SharedAiRuntimePort,
            model_routing,
            Arc::new(store) as crate::ai::hub::SharedAiHubUsageRecorder,
        ),
        provider.provider_kind == "api",
    ))
}

async fn resolve_mail_ai_model_routing(
    store: &crate::ai::control_center::AiControlCenterStore,
) -> Result<crate::ai::core::AiModelRouting, crate::ai::control_center::AiControlCenterError> {
    let mail_intelligence = resolve_mail_ai_slot_model(store, "mail_intelligence").await?;
    let mail_model_key = mail_intelligence.model_key.clone();

    Ok(crate::ai::core::AiModelRouting {
        default_chat: mail_model_key.clone(),
        reasoning: mail_model_key.clone(),
        summarization: mail_model_key.clone(),
        mail_intelligence: mail_model_key.clone(),
        reply_draft: mail_model_key.clone(),
        extraction: mail_model_key.clone(),
        embeddings: mail_model_key.clone(),
        meeting_prep: mail_model_key,
        targets: vec![mail_intelligence],
    })
}

async fn resolve_mail_ai_slot_model(
    store: &crate::ai::control_center::AiControlCenterStore,
    slot: &str,
) -> Result<crate::ai::core::AiModelRouteTarget, crate::ai::control_center::AiControlCenterError> {
    let Some(route) = store.route_for_slot(slot).await? else {
        return Err(
            crate::ai::control_center::AiControlCenterError::InvalidRequest(format!(
                "route_not_configured:{slot}: use Hub route settings"
            )),
        );
    };
    let Some(provider) = store.provider(&route.provider_id).await? else {
        return Err(
            crate::ai::control_center::AiControlCenterError::InvalidRequest(format!(
                "route_provider_missing:{}",
                route.provider_id
            )),
        );
    };
    store
        .ensure_model_ready_for_private_context(&route.provider_id, &route.model_key)
        .await?;

    Ok(crate::ai::core::AiModelRouteTarget {
        capability_slot: slot.to_owned(),
        provider_id: route.provider_id,
        model_key: route.model_key,
    })
}

fn mail_ai_route_provider_id(routing: &crate::ai::core::AiModelRouting) -> Option<&str> {
    routing
        .targets
        .iter()
        .find(|target| target.capability_slot == "mail_intelligence")
        .map(|target| target.provider_id.as_str())
}

async fn mail_ai_runtime_client(
    pool: &PgPool,
    store: &crate::ai::control_center::AiControlCenterStore,
    vault: &HostVault,
    settings: &AiRuntimeSettings,
    provider: &crate::ai::control_center::AiProviderAccount,
    routing: &crate::ai::core::AiModelRouting,
) -> Result<crate::integrations::ai_runtime::AiRuntimeClient, MailAiRuntimeBuildError> {
    match (
        provider.provider_kind.as_str(),
        provider.provider_key.as_str(),
    ) {
        ("built_in", "ollama") => Ok(crate::integrations::ai_runtime::AiRuntimeClient::Ollama(
            crate::integrations::ollama::client::OllamaClient::new(
                crate::integrations::ollama::client::config::OllamaClientConfig::new(
                    mail_ai_provider_base_url(provider)
                        .as_deref()
                        .unwrap_or(&settings.base_url),
                    &routing.mail_intelligence,
                    &routing.embeddings,
                )
                .with_timeout_seconds(settings.timeout_seconds),
            )
            .map_err(crate::integrations::ai_runtime::AiRuntimeError::from)?,
        )),
        ("api", _) => {
            let base_url = mail_ai_provider_base_url(provider).ok_or_else(|| {
                MailAiRuntimeBuildError::MissingBaseUrl(provider.provider_id.clone())
            })?;
            let api_key =
                mail_ai_provider_api_key(pool, store, vault, &provider.provider_id).await?;
            Ok(crate::integrations::ai_runtime::AiRuntimeClient::OmniRoute(
                crate::integrations::omniroute::client::OmniRouteClient::new(
                    crate::integrations::omniroute::client::config::OmniRouteClientConfig::new(
                        base_url,
                        &routing.mail_intelligence,
                        &routing.embeddings,
                        api_key,
                    )
                    .with_timeout_seconds(settings.timeout_seconds),
                )
                .map_err(crate::integrations::ai_runtime::AiRuntimeError::from)?,
            ))
        }
        _ => Err(MailAiRuntimeBuildError::UnsupportedProvider {
            provider_kind: provider.provider_kind.clone(),
            provider_key: provider.provider_key.clone(),
        }),
    }
}

async fn mail_ai_provider_api_key(
    pool: &PgPool,
    store: &crate::ai::control_center::AiControlCenterStore,
    vault: &HostVault,
    provider_id: &str,
) -> Result<crate::platform::secrets::ResolvedSecret, MailAiRuntimeBuildError> {
    let secret_ref = store
        .api_key_secret_ref(provider_id)
        .await?
        .ok_or_else(|| MailAiRuntimeBuildError::MissingApiKey(provider_id.to_owned()))?;
    let reference = crate::platform::secrets::SecretReferenceStore::new(pool.clone())
        .secret_reference(&secret_ref)
        .await?
        .ok_or_else(|| MailAiRuntimeBuildError::MissingSecretReference(secret_ref.clone()))?;
    Ok(vault.resolve_host_secret(&reference)?)
}

fn mail_ai_provider_base_url(
    provider: &crate::ai::control_center::AiProviderAccount,
) -> Option<String> {
    provider
        .config
        .get("base_url")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

#[derive(Debug, thiserror::Error)]
enum MailAiRuntimeBuildError {
    #[error("AI provider is not supported by mail AI runtime: {provider_kind}:{provider_key}")]
    UnsupportedProvider {
        provider_kind: String,
        provider_key: String,
    },

    #[error("AI API provider base_url is missing: {0}")]
    MissingBaseUrl(String),

    #[error("AI API provider key is not configured: {0}")]
    MissingApiKey(String),

    #[error("AI API provider secret reference is missing: {0}")]
    MissingSecretReference(String),

    #[error(transparent)]
    ControlCenter(#[from] crate::ai::control_center::AiControlCenterError),

    #[error(transparent)]
    SecretReference(#[from] crate::platform::secrets::SecretReferenceError),

    #[error(transparent)]
    SecretResolution(#[from] crate::platform::secrets::SecretResolutionError),

    #[error(transparent)]
    Runtime(#[from] crate::integrations::ai_runtime::AiRuntimeError),
}

pub(super) async fn mail_ai_target_language(pool: &PgPool) -> String {
    let language = sqlx::query_scalar::<_, String>(
        r#"
        SELECT language
        FROM personas
        WHERE is_self = true
          AND language IS NOT NULL
          AND length(trim(language)) > 0
        ORDER BY updated_at DESC
        LIMIT 1
        "#,
    )
    .fetch_optional(pool)
    .await;

    let Ok(language) = language else {
        return "ru".to_owned();
    };
    language
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "ru".to_owned())
}
