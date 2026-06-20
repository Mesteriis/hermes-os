use super::super::*;
use super::database::database_pool;

pub(crate) fn telegram_store(state: &AppState) -> Result<TelegramStore, ApiError> {
    Ok(TelegramStore::new(database_pool(state)?))
}

pub(crate) fn whatsapp_web_store(state: &AppState) -> Result<WhatsappWebStore, ApiError> {
    Ok(WhatsappWebStore::new(database_pool(state)?))
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
        SecretReferenceStore::new(pool),
        state.vault.clone(),
    ))
}
