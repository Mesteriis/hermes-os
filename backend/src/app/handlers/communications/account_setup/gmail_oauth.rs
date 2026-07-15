use super::super::*;
use super::calendar::upsert_google_workspace_calendar_account;
use super::helpers::{gmail_pending_external_account_id, trimmed_optional};
use super::models::{
    EmailAccountSetupApiResponse, GmailOAuthCompleteApiRequest, GmailOAuthStartApiRequest,
    GmailOAuthStartApiResponse,
};
use crate::app::handlers::communications::account_support::*;
use crate::app::signal_hub_support::{
    provider_account_or_not_found, sync_provider_account_signal_connection,
};

pub(crate) async fn post_gmail_oauth_start(
    State(state): State<AppState>,
    Json(request): Json<GmailOAuthStartApiRequest>,
) -> Result<Json<GmailOAuthStartApiResponse>, ApiError> {
    require_unlocked_host_vault(&state)?;
    let service = account_setup_service(&state)?;
    let pending = service.start_gmail_oauth(request.into_setup_request(&state.config)?)?;
    let response = GmailOAuthStartApiResponse {
        setup_id: pending.setup_id.clone(),
        authorization_url: pending.authorization_url.clone(),
        state: pending.state.clone(),
        redirect_uri: pending.request.redirect_uri.clone(),
    };
    let mut pending_map = state
        .account_setup
        .pending_gmail_oauth
        .lock()
        .map_err(|_| ApiError::AccountSetupState)?;
    pending_map.insert(pending.setup_id.clone(), pending);

    Ok(Json(response))
}

pub(crate) async fn post_gmail_oauth_complete(
    State(state): State<AppState>,
    Json(request): Json<GmailOAuthCompleteApiRequest>,
) -> Result<Json<EmailAccountSetupApiResponse>, ApiError> {
    let mut pending = {
        let mut pending_map = state
            .account_setup
            .pending_gmail_oauth
            .lock()
            .map_err(|_| ApiError::AccountSetupState)?;
        pending_map
            .remove(&request.setup_id)
            .ok_or(ApiError::AccountSetupPendingGrantNotFound)?
    };
    if pending.state != request.state {
        return Err(ApiError::AccountSetupStateMismatch);
    }
    if let Some(external_account_id) = trimmed_optional(request.external_account_id) {
        pending.request = pending.request.external_account_id(external_account_id);
    }
    let mail_account_id = pending.account_id.clone();
    let display_name = pending.request.display_name.clone();
    let external_account_id = gmail_pending_external_account_id(&pending);

    let service = account_setup_service(&state)?;
    let result = service
        .complete_gmail_oauth(pending, &request.authorization_code)
        .await?;
    let account = provider_account_or_not_found(&state, &result.account_id).await?;
    sync_provider_account_signal_connection(&state, &account, Some(&result.secret_ref)).await?;
    upsert_google_workspace_calendar_account(
        &state,
        &mail_account_id,
        &display_name,
        &external_account_id,
        &result.secret_ref,
    )
    .await?;

    Ok(Json(result.into()))
}
