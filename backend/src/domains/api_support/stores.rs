use super::*;

pub(crate) fn event_store(state: &AppState) -> Result<EventStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(EventStore::new(pool.clone()))
}

pub(crate) fn graph_store(
    state: &AppState,
) -> Result<crate::domains::graph::core::GraphStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(crate::domains::graph::core::GraphStore::new(pool.clone()))
}

pub(crate) fn message_store(state: &AppState) -> Result<MessageProjectionStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(MessageProjectionStore::new(pool.clone()))
}

pub(crate) fn mail_storage_store(state: &AppState) -> Result<MailStorageStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(MailStorageStore::new(pool.clone()))
}

pub(crate) fn communication_ingestion_store(
    state: &AppState,
) -> Result<CommunicationIngestionStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(CommunicationIngestionStore::new(pool.clone()))
}

pub(crate) fn project_store(state: &AppState) -> Result<ProjectStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(ProjectStore::new(pool.clone()))
}

pub(crate) fn project_link_review_store(
    state: &AppState,
) -> Result<ProjectLinkReviewStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(ProjectLinkReviewStore::new(pool.clone()))
}

pub(crate) fn task_candidate_store(state: &AppState) -> Result<TaskCandidateStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(TaskCandidateStore::new(pool.clone()))
}

pub(crate) fn ai_run_store(state: &AppState) -> Result<crate::ai::core::AiRunStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(crate::ai::core::AiRunStore::new(pool.clone()))
}

pub(crate) async fn ai_service(state: &AppState) -> Result<AiService, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };
    let runtime_settings = ai_runtime_settings(state).await?;
    let model_routing = ai_model_routing(state, &runtime_settings).await?;
    let runtime = ai_runtime_client(state, &runtime_settings)?;

    Ok(AiService::new_with_routing(
        pool.clone(),
        runtime,
        model_routing,
    ))
}

pub(crate) fn telegram_store(state: &AppState) -> Result<TelegramStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(TelegramStore::new(pool.clone()))
}

pub(crate) fn whatsapp_web_store(state: &AppState) -> Result<WhatsappWebStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(WhatsappWebStore::new(pool.clone()))
}

pub(crate) fn automation_store(state: &AppState) -> Result<AutomationStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(AutomationStore::new(pool.clone()))
}

pub(crate) fn call_intelligence_store(state: &AppState) -> Result<CallIntelligenceStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(CallIntelligenceStore::new(pool.clone()))
}

pub(crate) fn settings_store(state: &AppState) -> Result<ApplicationSettingsStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(ApplicationSettingsStore::new(pool.clone()))
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

async fn ai_model_routing(
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

pub(crate) fn document_processing_store(
    state: &AppState,
) -> Result<DocumentProcessingStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(DocumentProcessingStore::new(pool.clone()))
}

pub(crate) fn person_identity_store(state: &AppState) -> Result<PersonIdentityStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(PersonIdentityStore::new(pool.clone()))
}

pub(crate) async fn email_multilingual_service(
    state: &AppState,
) -> Result<crate::domains::mail::multilingual::MultilingualService, ApiError> {
    let settings = ai_runtime_settings(state).await?;
    Ok(
        crate::domains::mail::multilingual::MultilingualService::new(
            ai_runtime_client(state, &settings).ok(),
        ),
    )
}

pub(crate) async fn email_ai_reply_service(
    state: &AppState,
) -> Result<crate::domains::mail::ai_reply::AiReplyService, ApiError> {
    let settings = ai_runtime_settings(state).await?;
    Ok(crate::domains::mail::ai_reply::AiReplyService::new(
        ai_runtime_client(state, &settings).ok(),
    ))
}

pub(crate) fn api_audit_log(state: &AppState) -> Result<ApiAuditLog, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(ApiAuditLog::new(pool.clone()))
}

pub(crate) fn account_setup_service(
    state: &AppState,
) -> Result<EmailAccountSetupService, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(EmailAccountSetupService::new_with_host_vault(
        CommunicationIngestionStore::new(pool.clone()),
        SecretReferenceStore::new(pool.clone()),
        state.vault.clone(),
    ))
}

pub(crate) fn database_encrypted_vault(
    config: &AppConfig,
    pool: sqlx::postgres::PgPool,
) -> Option<DatabaseEncryptedSecretVault> {
    Some(DatabaseEncryptedSecretVault::new(
        pool,
        config.secret_vault_key()?.clone(),
    ))
}
