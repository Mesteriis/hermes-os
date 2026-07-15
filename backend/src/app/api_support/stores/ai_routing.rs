use super::super::*;

pub(in crate::app::api_support::stores) async fn ai_model_routing(
    state: &AppState,
    settings: &AiRuntimeSettings,
) -> Result<AiModelRouting, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };
    let store = AiControlCenterStore::new(pool.clone());
    resolve_ai_model_routing(&store, settings)
        .await
        .map_err(ApiError::from)
}

async fn resolve_ai_model_routing(
    store: &AiControlCenterStore,
    settings: &AiRuntimeSettings,
) -> Result<AiModelRouting, AiControlCenterError> {
    let default_chat = resolve_ai_slot_model(store, settings, "default_chat").await?;
    let reasoning = resolve_ai_slot_model(store, settings, "reasoning").await?;
    let summarization = resolve_ai_slot_model(store, settings, "summarization").await?;
    let mail_intelligence = resolve_ai_slot_model(store, settings, "mail_intelligence").await?;
    let reply_draft = resolve_ai_slot_model(store, settings, "reply_draft").await?;
    let extraction = resolve_ai_slot_model(store, settings, "extraction").await?;
    let embeddings = resolve_ai_slot_model(store, settings, "embeddings").await?;
    let meeting_prep = resolve_ai_slot_model(store, settings, "meeting_prep").await?;

    Ok(AiModelRouting {
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

async fn resolve_ai_slot_model(
    store: &AiControlCenterStore,
    settings: &AiRuntimeSettings,
    slot: &str,
) -> Result<crate::ai::core::types::AiModelRouteTarget, AiControlCenterError> {
    let Some(route) = store.route_for_slot(slot).await? else {
        return Err(AiControlCenterError::InvalidRequest(format!(
            "route_not_configured:{slot}: use Hub route settings"
        )));
    };
    let Some(provider) = store.provider(&route.provider_id).await? else {
        return Err(AiControlCenterError::InvalidRequest(format!(
            "route_provider_missing:{}",
            route.provider_id
        )));
    };
    if !ai_provider_matches_runtime(&provider, settings.provider) {
        return Err(AiControlCenterError::InvalidRequest(
            "route_provider_mismatch: runtime and route provider kinds diverge".to_owned(),
        ));
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

fn ai_provider_matches_runtime(
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
