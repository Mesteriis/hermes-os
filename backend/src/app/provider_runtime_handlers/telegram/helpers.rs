use crate::app::api_support::{
    automation_calls::*,
    communications::*,
    ensure_fixture_routes_enabled,
    messaging_integrations::*,
    platform_dtos::*,
    query_parsing::{communication::*, documents::*, graph::*, personas::*, projects::*, tasks::*},
    review_commands::*,
    review_lists::*,
    stores::{ai_runtime::*, domain_stores::*, integration_stores::*, settings_vault::*},
    telegram_capabilities::*,
    whatsapp_capabilities::*,
};
use crate::app::{ApiError, AppState};
use crate::integrations::telegram::client::TelegramError;
use crate::platform::config::AppConfig;
use hermes_events_api::NewEventEnvelope;

pub(super) const AUDIT_ACTOR_ID: &str = "hermes-frontend";

pub(super) fn telegram_api_hash_from_config(config: &AppConfig) -> Option<String> {
    config
        .telegram_api_hash()
        .map(|secret| secret.expose_for_runtime().to_owned())
}

pub(super) fn telegram_secret_store(
    state: &AppState,
) -> Result<crate::platform::secrets::SecretReferenceStore, ApiError> {
    telegram_secret_reference_store(state)
}

pub(super) async fn publish_telegram_event(
    state: &AppState,
    event: NewEventEnvelope,
) -> Result<(), ApiError> {
    if state.database.pool().is_some()
        && let Err(error) = event_store(state)?.append(&event).await
    {
        tracing::warn!(error = %error, "failed to append event to event store");
    }

    let _ = state.event_bus.broadcast(event);
    Ok(())
}

pub(super) async fn ensure_telegram_account_operation_allowed(
    state: &AppState,
    account_id: &str,
    operation: &str,
) -> Result<(), ApiError> {
    let account = telegram_provider_runtime_service(state)?
        .telegram_account_record(account_id)
        .await?;
    let capabilities = TelegramCapabilitiesResponse::current_for_account(&state.config, &account);
    let capability = capabilities
        .capabilities
        .iter()
        .find(|item| item.operation == operation)
        .ok_or_else(|| {
            ApiError::Telegram(TelegramError::InvalidRequest(format!(
                "Telegram capability `{operation}` is not defined"
            )))
        })?;

    if matches!(capability.status.as_str(), "available" | "degraded") {
        return Ok(());
    }

    Err(ApiError::Telegram(TelegramError::InvalidRequest(
        capability.reason.clone(),
    )))
}
