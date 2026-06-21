use axum::Json;
use axum::extract::{Path, State};
use serde_json::{Value, json};

use super::helpers::telegram_api_hash_from_config;
use crate::app::{ApiError, AppState};
use crate::integrations::telegram::client::{
    TelegramError, TelegramQrLoginPasswordRequest, TelegramQrLoginStartRequest,
    TelegramQrLoginStatusResponse,
};
use crate::integrations::telegram::tdjson;

pub(crate) async fn post_telegram_qr_login_start(
    State(state): State<AppState>,
    Json(request): Json<TelegramQrLoginStartRequest>,
) -> Result<Json<TelegramQrLoginStatusResponse>, ApiError> {
    let request = request.with_app_credentials(
        state.config.telegram_api_id(),
        telegram_api_hash_from_config(&state.config),
    );

    Ok(Json(
        tdjson::start_qr_login(
            state.config.clone(),
            state.account_setup.pending_telegram_qr_login.clone(),
            request,
        )
        .await?,
    ))
}

pub(crate) async fn get_telegram_qr_login_status(
    State(state): State<AppState>,
    Path(setup_id): Path<String>,
) -> Result<Json<TelegramQrLoginStatusResponse>, ApiError> {
    let pending = state
        .account_setup
        .pending_telegram_qr_login
        .lock()
        .map_err(|_| ApiError::AccountSetupState)?;
    let session = pending
        .get(setup_id.trim())
        .map(|session| session.response.clone())
        .ok_or(ApiError::Telegram(TelegramError::QrLoginNotFound))?;

    Ok(Json(session))
}

pub(crate) async fn delete_telegram_qr_login(
    State(state): State<AppState>,
    Path(setup_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let setup_id = setup_id.trim().to_owned();
    tdjson::cancel_qr_login(
        state.account_setup.pending_telegram_qr_login.clone(),
        &setup_id,
    )?;

    Ok(Json(json!({
        "setup_id": setup_id,
        "cancelled": true
    })))
}

pub(crate) async fn post_telegram_qr_login_password(
    State(state): State<AppState>,
    Path(setup_id): Path<String>,
    Json(request): Json<TelegramQrLoginPasswordRequest>,
) -> Result<Json<TelegramQrLoginStatusResponse>, ApiError> {
    Ok(Json(tdjson::submit_qr_login_password(
        state.account_setup.pending_telegram_qr_login.clone(),
        &setup_id,
        request,
    )?))
}
