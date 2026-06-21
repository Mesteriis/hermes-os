use super::super::*;
use super::database::database_pool;
use std::sync::Arc;

pub(crate) fn telegram_store(state: &AppState) -> Result<TelegramStore, ApiError> {
    let pool = database_pool(state)?;
    Ok(TelegramStore::new(
        pool.clone(),
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
        Arc::new(crate::domains::communications::messages::ProviderChannelMessageStore::new(pool)),
    ))
}

pub(crate) fn whatsapp_web_store(state: &AppState) -> Result<WhatsappWebStore, ApiError> {
    let pool = database_pool(state)?;
    Ok(WhatsappWebStore::new(
        pool.clone(),
        Arc::new(
            crate::domains::communications::core::CommunicationProviderAccountStore::new(
                pool.clone(),
            ),
        ),
        Arc::new(crate::domains::communications::messages::ProviderChannelMessageStore::new(pool)),
    ))
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
