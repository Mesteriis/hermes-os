use super::super::*;

pub(in crate::app::api_support::stores) async fn ai_model_routing(
    state: &AppState,
    settings: &AiRuntimeSettings,
) -> Result<AiModelRouting, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Ok(AiModelRouting::fallback(
            &settings.chat_model,
            &settings.embedding_model,
        ));
    };
    let store = AiControlCenterStore::new(pool.clone());
    match resolve_ai_model_routing(&store, settings).await {
        Ok(routing) => Ok(routing),
        Err(error) => {
            tracing::warn!(error = %error, "AI model routing resolution fell back to legacy ai.* settings");
            Ok(AiModelRouting::fallback(
                &settings.chat_model,
                &settings.embedding_model,
            ))
        }
    }
}

async fn resolve_ai_model_routing(
    store: &AiControlCenterStore,
    settings: &AiRuntimeSettings,
) -> Result<AiModelRouting, AiControlCenterError> {
    Ok(AiModelRouting {
        default_chat: resolve_ai_slot_model(store, settings, "default_chat", &settings.chat_model)
            .await?,
        reasoning: resolve_ai_slot_model(store, settings, "reasoning", &settings.chat_model)
            .await?,
        summarization: resolve_ai_slot_model(
            store,
            settings,
            "summarization",
            &settings.chat_model,
        )
        .await?,
        mail_intelligence: resolve_ai_slot_model(
            store,
            settings,
            "mail_intelligence",
            &settings.chat_model,
        )
        .await?,
        reply_draft: resolve_ai_slot_model(store, settings, "reply_draft", &settings.chat_model)
            .await?,
        extraction: resolve_ai_slot_model(store, settings, "extraction", &settings.chat_model)
            .await?,
        embeddings: resolve_ai_slot_model(store, settings, "embeddings", &settings.embedding_model)
            .await?,
        meeting_prep: resolve_ai_slot_model(store, settings, "meeting_prep", &settings.chat_model)
            .await?,
    })
}

async fn resolve_ai_slot_model(
    store: &AiControlCenterStore,
    settings: &AiRuntimeSettings,
    slot: &str,
    fallback_model: &str,
) -> Result<String, AiControlCenterError> {
    let Some(route) = store.route_for_slot(slot).await? else {
        return Ok(fallback_model.to_owned());
    };
    let Some(provider) = store.provider(&route.provider_id).await? else {
        return Ok(fallback_model.to_owned());
    };
    if ai_provider_matches_runtime(&provider, settings.provider)
        && store
            .model_ready_for_private_context(&route.provider_id, &route.model_key)
            .await?
    {
        Ok(route.model_key)
    } else {
        Ok(fallback_model.to_owned())
    }
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
