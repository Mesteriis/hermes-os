use crate::app::api_support::stores::domain_stores::communication_provider_account_store;
use crate::app::api_support::stores::integration_stores::{
    whatsapp_provider_runtime_service, whatsapp_secret_reference_store,
};
use crate::app::error::types::ApiError;
use crate::app::provider_runtime_handlers::whatsapp::NewWhatsappWebRuntimeEvent;
use crate::app::state::AppState;
use crate::integrations::whatsapp::client::errors::WhatsappWebError;
use serde_json::Value;

pub(super) async fn project_runtime_bridge_lifecycle_state(
    state: &AppState,
    request: &NewWhatsappWebRuntimeEvent,
) -> Result<(), ApiError> {
    let Some(lifecycle_state) = request.effective_lifecycle_state() else {
        return Ok(());
    };
    if !matches!(
        lifecycle_state,
        "linked"
            | "available"
            | "syncing"
            | "degraded"
            | "blocked"
            | "revoked"
            | "removed"
            | "qr_pending"
            | "pair_code_pending"
            | "created"
    ) {
        return Ok(());
    }
    communication_provider_account_store(state)?
        .update_whatsapp_lifecycle_state(&request.account_id, lifecycle_state)
        .await
        .map_err(|error| WhatsappWebError::ProviderAccountStore(error.to_string()))?;
    Ok(())
}

pub(super) async fn authorized_session_lifecycle_source(
    state: &AppState,
    account_id: &str,
) -> Result<&'static str, ApiError> {
    let status = whatsapp_provider_runtime_service(state)?
        .runtime_status(
            &whatsapp_secret_reference_store(state)?,
            &state.vault,
            account_id,
        )
        .await?;
    Ok(if status.session_restore_available {
        "session_rotated"
    } else {
        "session_authorized"
    })
}

pub(super) fn merge_runtime_event_metadata(current: Value, patch: Value) -> Value {
    let mut current_map = current.as_object().cloned().unwrap_or_default();
    if let Some(patch_map) = patch.as_object() {
        current_map.extend(patch_map.clone());
    }
    Value::Object(current_map)
}
