use axum::Json;
use axum::extract::State;

use super::helpers::telegram_secret_store;
use crate::app::{ApiError, AppState};
use crate::domains::api_support::{
    communication_ingestion_store, mail_storage_store, telegram_store,
};
use crate::integrations::telegram::runtime::{
    TelegramMediaDownloadContext, TelegramMediaDownloadRequest, TelegramMediaDownloadResponse,
};

pub(crate) async fn post_telegram_media_download(
    State(state): State<AppState>,
    Json(request): Json<TelegramMediaDownloadRequest>,
) -> Result<Json<TelegramMediaDownloadResponse>, ApiError> {
    let secret_store = telegram_secret_store(&state)?;
    let communication_store = communication_ingestion_store(&state)?;
    let telegram_store = telegram_store(&state)?;
    let mail_store = mail_storage_store(&state)?;
    Ok(Json(
        state
            .telegram_runtime
            .download_media(
                TelegramMediaDownloadContext {
                    communication_store: &communication_store,
                    telegram_store: &telegram_store,
                    mail_store: &mail_store,
                    secret_store: &secret_store,
                    secret_resolver: &state.vault,
                    config: &state.config,
                },
                &request,
            )
            .await?,
    ))
}
