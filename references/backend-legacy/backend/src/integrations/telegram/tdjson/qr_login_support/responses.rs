use crate::integrations::telegram::client::errors::TelegramError;
use crate::integrations::telegram::client::models::qr_login::{
    TelegramQrLoginStatus, TelegramQrLoginStatusResponse,
};

use super::constants::QR_POLL_AFTER_MS;
use super::qr::render_qr_svg;
use super::types::TelegramQrLoginIdentity;

pub(in crate::integrations::telegram::tdjson) fn qr_waiting_response(
    setup_id: &str,
    account_id: &str,
    link: &str,
) -> Result<TelegramQrLoginStatusResponse, TelegramError> {
    Ok(TelegramQrLoginStatusResponse {
        setup_id: setup_id.to_owned(),
        account_id: account_id.to_owned(),
        status: TelegramQrLoginStatus::WaitingQrScan,
        qr_link: Some(link.to_owned()),
        qr_svg: Some(render_qr_svg(link)?),
        telegram_user_id: None,
        telegram_username: None,
        suggested_account_id: None,
        suggested_display_name: None,
        suggested_external_account_id: None,
        expires_at: None,
        poll_after_ms: QR_POLL_AFTER_MS,
        message: Some("Scan this QR code from an already logged-in Telegram device.".to_owned()),
    })
}

pub(in crate::integrations::telegram::tdjson) fn qr_preparing_response(
    setup_id: &str,
    account_id: &str,
) -> TelegramQrLoginStatusResponse {
    TelegramQrLoginStatusResponse {
        setup_id: setup_id.to_owned(),
        account_id: account_id.to_owned(),
        status: TelegramQrLoginStatus::WaitingQrScan,
        qr_link: None,
        qr_svg: None,
        telegram_user_id: None,
        telegram_username: None,
        suggested_account_id: None,
        suggested_display_name: None,
        suggested_external_account_id: None,
        expires_at: None,
        poll_after_ms: 1_000,
        message: Some("Preparing Telegram QR code.".to_owned()),
    }
}

pub(in crate::integrations::telegram::tdjson) fn password_waiting_response(
    setup_id: &str,
    account_id: &str,
    message: &str,
) -> TelegramQrLoginStatusResponse {
    TelegramQrLoginStatusResponse {
        setup_id: setup_id.to_owned(),
        account_id: account_id.to_owned(),
        status: TelegramQrLoginStatus::WaitingPassword,
        qr_link: None,
        qr_svg: None,
        telegram_user_id: None,
        telegram_username: None,
        suggested_account_id: None,
        suggested_display_name: None,
        suggested_external_account_id: None,
        expires_at: None,
        poll_after_ms: QR_POLL_AFTER_MS,
        message: Some(message.to_owned()),
    }
}

pub(in crate::integrations::telegram::tdjson) fn ready_response(
    setup_id: &str,
    account_id: &str,
    message: &str,
    identity: Option<&TelegramQrLoginIdentity>,
) -> TelegramQrLoginStatusResponse {
    TelegramQrLoginStatusResponse {
        setup_id: setup_id.to_owned(),
        account_id: identity
            .map(|identity| identity.suggested_account_id.clone())
            .unwrap_or_else(|| account_id.to_owned()),
        status: TelegramQrLoginStatus::Ready,
        qr_link: None,
        qr_svg: None,
        telegram_user_id: identity.map(|identity| identity.user_id.clone()),
        telegram_username: identity.and_then(|identity| identity.username.clone()),
        suggested_account_id: identity.map(|identity| identity.suggested_account_id.clone()),
        suggested_display_name: identity.map(|identity| identity.suggested_display_name.clone()),
        suggested_external_account_id: identity
            .map(|identity| identity.suggested_external_account_id.clone()),
        expires_at: None,
        poll_after_ms: 0,
        message: Some(message.to_owned()),
    }
}
