use super::super::*;
use super::database::database_pool;
use std::sync::Arc;

fn build_telegram_provider_store(
    state: &AppState,
) -> Result<crate::application::TelegramProviderRuntimeStore, ApiError> {
    Ok(crate::application::telegram_provider_runtime_store(
        database_pool(state)?,
    ))
}

pub(crate) fn telegram_provider_runtime_service(
    state: &AppState,
) -> Result<crate::application::TelegramProviderRuntimeApplicationService, ApiError> {
    Ok(crate::application::telegram_provider_runtime_service(
        database_pool(state)?,
    ))
}

pub(crate) fn telegram_secret_reference_store(
    state: &AppState,
) -> Result<SecretReferenceStore, ApiError> {
    Ok(SecretReferenceStore::new(database_pool(state)?))
}

pub(crate) fn whatsapp_secret_reference_store(
    state: &AppState,
) -> Result<SecretReferenceStore, ApiError> {
    Ok(SecretReferenceStore::new(database_pool(state)?))
}

pub(crate) fn zulip_secret_reference_store(
    state: &AppState,
) -> Result<SecretReferenceStore, ApiError> {
    Ok(SecretReferenceStore::new(database_pool(state)?))
}

pub(crate) fn zoom_secret_reference_store(
    state: &AppState,
) -> Result<SecretReferenceStore, ApiError> {
    Ok(SecretReferenceStore::new(database_pool(state)?))
}

pub(crate) fn telegram_runtime_use_case_context(
    state: &AppState,
) -> Result<crate::application::telegram_runtime::TelegramRuntimeUseCaseContext<'_>, ApiError> {
    let pool = database_pool(state)?;
    Ok(
        crate::application::telegram_runtime::TelegramRuntimeUseCaseContext::new(
            crate::application::telegram_runtime::TelegramRuntimeUseCaseStores {
                provider_account_store:
                    crate::domains::communications::core::CommunicationProviderAccountStore::new(
                        pool.clone(),
                    ),
                provider_secret_binding_store:
                    crate::domains::communications::core::CommunicationProviderSecretBindingStore::new(
                        pool.clone(),
                    ),
                telegram_store: build_telegram_provider_store(state)?,
                secret_store: SecretReferenceStore::new(pool),
            },
            crate::application::telegram_runtime::TelegramRuntimeUseCaseRuntime {
                secret_resolver: &state.vault,
                config: &state.config,
                event_bus: &state.event_bus,
                runtime: &state.telegram_runtime,
            },
        ),
    )
}

pub(crate) fn telegram_message_write_service(
    state: &AppState,
) -> Result<
    crate::application::communication_provider_writes::TelegramMessageWriteApplicationService,
    ApiError,
> {
    Ok(
        crate::application::communication_provider_writes::TelegramMessageWriteApplicationService::new(
            build_telegram_provider_store(state)?,
            api_audit_log(state)?,
            event_store(state)?,
            state.event_bus.clone(),
        ),
    )
}

pub(crate) fn telegram_fixture_ingest_service(
    state: &AppState,
) -> Result<
    crate::application::communication_fixture_ingest::TelegramFixtureIngestApplicationService,
    ApiError,
> {
    Ok(
        crate::application::communication_fixture_ingest::TelegramFixtureIngestApplicationService::new(
            database_pool(state)?,
            build_telegram_provider_store(state)?,
            event_store(state)?,
            state.event_bus.clone(),
        ),
    )
}

fn build_whatsapp_provider_store(
    state: &AppState,
) -> Result<crate::application::WhatsAppProviderRuntimeRef, ApiError> {
    Ok(crate::application::whatsapp_provider_runtime(
        database_pool(state)?,
    ))
}

pub(crate) fn whatsapp_provider_runtime_service(
    state: &AppState,
) -> Result<crate::application::WhatsappProviderRuntimeApplicationService, ApiError> {
    Ok(crate::application::whatsapp_provider_runtime_service(
        database_pool(state)?,
    ))
}

pub(crate) fn zoom_provider_runtime_service(
    state: &AppState,
) -> Result<crate::application::ZoomProviderRuntimeApplicationService, ApiError> {
    Ok(crate::application::zoom_provider_runtime_service(
        database_pool(state)?,
        state.event_bus.clone(),
    ))
}

pub(crate) fn yandex_telemost_secret_reference_store(
    state: &AppState,
) -> Result<SecretReferenceStore, ApiError> {
    Ok(SecretReferenceStore::new(database_pool(state)?))
}

pub(crate) fn yandex_telemost_provider_runtime_store(
    state: &AppState,
) -> Result<crate::integrations::yandex_telemost::client::YandexTelemostStore, ApiError> {
    let pool = database_pool(state)?;
    Ok(
        crate::integrations::yandex_telemost::client::YandexTelemostStore::new(
            Arc::new(
                crate::domains::communications::core::CommunicationProviderAccountStore::new(
                    pool.clone(),
                ),
            ),
            Arc::new(
                crate::domains::communications::core::CommunicationProviderSecretBindingStore::new(
                    pool.clone(),
                ),
            ),
            event_store(state)?,
            state.event_bus.clone(),
        ),
    )
}

pub(crate) fn yandex_telemost_provider_runtime_service(
    state: &AppState,
) -> Result<crate::application::YandexTelemostProviderRuntimeApplicationService, ApiError> {
    Ok(
        crate::application::yandex_telemost_provider_runtime_service(
            database_pool(state)?,
            state.event_bus.clone(),
        ),
    )
}

pub(crate) fn whatsapp_fixture_ingest_service(
    state: &AppState,
) -> Result<
    crate::application::communication_fixture_ingest::WhatsappFixtureIngestApplicationService,
    ApiError,
> {
    Ok(
        crate::application::communication_fixture_ingest::WhatsappFixtureIngestApplicationService::new(
            database_pool(state)?,
            build_whatsapp_provider_store(state)?,
            event_store(state)?,
            state.event_bus.clone(),
        ),
    )
}

pub(crate) fn automation_store(state: &AppState) -> Result<AutomationStore, ApiError> {
    Ok(AutomationStore::new(database_pool(state)?))
}

pub(crate) fn call_intelligence_store(state: &AppState) -> Result<CallIntelligenceStore, ApiError> {
    Ok(CallIntelligenceStore::new(database_pool(state)?))
}

pub(crate) fn account_setup_service(
    state: &AppState,
) -> Result<EmailAccountSetupService, ApiError> {
    let pool = database_pool(state)?;
    Ok(EmailAccountSetupService::new_with_host_vault(
        pool.clone(),
        SecretReferenceStore::new(pool.clone()),
        state.vault.clone(),
        Arc::new(
            crate::domains::communications::core::CommunicationProviderAccountStore::new(
                pool.clone(),
            ),
        ),
        Arc::new(
            crate::domains::communications::core::CommunicationProviderSecretBindingStore::new(
                pool,
            ),
        ),
    ))
}
