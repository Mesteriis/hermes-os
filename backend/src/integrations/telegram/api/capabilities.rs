use axum::Json;
use axum::extract::{Path, State};

use crate::app::{ApiError, AppState};
use crate::domains::api_support::{TelegramCapabilitiesResponse, communication_ingestion_store};

pub(crate) async fn get_telegram_capabilities(
    State(state): State<AppState>,
) -> Result<Json<TelegramCapabilitiesResponse>, ApiError> {
    Ok(Json(TelegramCapabilitiesResponse::current(&state.config)))
}

pub(crate) async fn get_telegram_account_capabilities(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
) -> Result<Json<TelegramCapabilitiesResponse>, ApiError> {
    let account = communication_ingestion_store(&state)?
        .provider_account(&account_id)
        .await?
        .ok_or_else(|| {
            crate::app::ApiError::Telegram(
                crate::integrations::telegram::client::TelegramError::InvalidRequest(format!(
                    "Telegram account `{account_id}` is not configured"
                )),
            )
        })?;
    if !account.provider_kind.is_telegram() {
        return Err(crate::app::ApiError::Telegram(
            crate::integrations::telegram::client::TelegramError::InvalidRequest(format!(
                "account `{}` is not a Telegram provider account",
                account.account_id
            )),
        ));
    }

    Ok(Json(TelegramCapabilitiesResponse::current_for_account(
        &state.config,
        &account,
    )))
}
