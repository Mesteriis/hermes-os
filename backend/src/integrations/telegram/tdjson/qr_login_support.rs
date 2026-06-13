use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

use chrono::Utc;
use qrcode::QrCode;
use qrcode::render::svg;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::integrations::telegram::client::{
    TelegramError, TelegramQrLoginStartRequest, TelegramQrLoginStatus,
    TelegramQrLoginStatusResponse,
};

use super::client::TdJsonClient;
use super::identifiers::safe_path_segment;
use super::parsing::tdlib_error_message;

pub(super) const QR_FIRST_LINK_TIMEOUT: Duration = Duration::from_secs(20);
pub(super) const QR_SESSION_LIFETIME: Duration = Duration::from_secs(10 * 60);
const QR_CANCEL_WAIT_TIMEOUT: Duration = Duration::from_secs(5);
const QR_GET_ME_TIMEOUT: Duration = Duration::from_secs(5);
pub(super) const QR_POLL_AFTER_MS: u64 = 2_000;

pub(crate) type PendingQrLoginMap = Arc<Mutex<HashMap<String, TelegramQrLoginSession>>>;
pub(super) type QrLoginWorkerCompletion = Arc<(Mutex<bool>, Condvar)>;

#[derive(Clone)]
pub(crate) struct TelegramQrLoginSession {
    pub(crate) response: TelegramQrLoginStatusResponse,
    pub(super) command_tx: Sender<TelegramQrLoginCommand>,
    pub(super) worker_completion: QrLoginWorkerCompletion,
}

#[derive(Debug, Eq, PartialEq)]
pub(super) enum TelegramQrLoginCommand {
    CheckPassword(String),
    Cancel,
}

#[derive(Debug, Eq, PartialEq)]
pub(super) enum DrainedQrLoginCommand {
    None,
    PasswordSubmitted,
    Cancelled,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct TelegramQrLoginIdentity {
    pub(super) user_id: String,
    pub(super) username: Option<String>,
    pub(super) suggested_account_id: String,
    pub(super) suggested_display_name: String,
    pub(super) suggested_external_account_id: String,
}

pub(super) fn new_worker_completion() -> QrLoginWorkerCompletion {
    Arc::new((Mutex::new(false), Condvar::new()))
}

pub(super) fn mark_worker_complete(worker_completion: &QrLoginWorkerCompletion) {
    let (lock, cvar) = &**worker_completion;
    if let Ok(mut completed) = lock.lock() {
        *completed = true;
        cvar.notify_all();
    }
}

pub(super) fn wait_for_worker_completion(
    worker_completion: &QrLoginWorkerCompletion,
) -> Result<(), TelegramError> {
    let (lock, cvar) = &**worker_completion;
    let completed = lock.lock().map_err(|_| {
        TelegramError::TdlibRuntime("Telegram QR login worker lock was poisoned".to_owned())
    })?;
    if *completed {
        return Ok(());
    }
    let _ = cvar
        .wait_timeout(completed, QR_CANCEL_WAIT_TIMEOUT)
        .map_err(|_| {
            TelegramError::TdlibRuntime("Telegram QR login worker lock was poisoned".to_owned())
        })?;
    Ok(())
}

pub(super) fn qr_waiting_response(
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

pub(super) fn qr_preparing_response(
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

pub(super) fn password_waiting_response(
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

pub(super) fn ready_response(
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

pub(super) fn fetch_authorized_user_identity(
    client: &TdJsonClient,
) -> Result<Option<TelegramQrLoginIdentity>, TelegramError> {
    client.send_json(&json!({
        "@type": "getMe",
        "@extra": "hermes-get-me"
    }))?;

    let started_at = Instant::now();
    while started_at.elapsed() < QR_GET_ME_TIMEOUT {
        let Some(event) = client.receive_json(1.0)? else {
            continue;
        };

        if event.get("@type").and_then(Value::as_str) == Some("user") {
            return Ok(parse_tdlib_user_identity(&event));
        }

        if event.get("@extra").and_then(Value::as_str) == Some("hermes-get-me") {
            if let Some(message) = tdlib_error_message(&event) {
                return Err(TelegramError::TdlibRuntime(message));
            }
            return Ok(parse_tdlib_user_identity(&event));
        }
    }

    Ok(None)
}

pub(super) fn parse_tdlib_user_identity(user: &Value) -> Option<TelegramQrLoginIdentity> {
    let user_id = user
        .get("id")
        .and_then(|value| {
            value
                .as_i64()
                .map(|value| value.to_string())
                .or_else(|| value.as_u64().map(|value| value.to_string()))
        })
        .filter(|value| !value.trim().is_empty())?;
    let username = tdlib_user_username(user);
    let safe_user_id = safe_account_identifier(&user_id);
    let suggested_account_id = username
        .as_deref()
        .map(safe_account_identifier)
        .filter(|value| !value.is_empty())
        .map(|username| format!("{safe_user_id}_account_{username}"))
        .unwrap_or_else(|| format!("{safe_user_id}_account"));
    let suggested_display_name = username
        .as_deref()
        .map(|value| format!("@{value}"))
        .unwrap_or_else(|| user_id.clone());
    let suggested_external_account_id = format!("telegram:{user_id}");

    Some(TelegramQrLoginIdentity {
        user_id,
        username,
        suggested_account_id,
        suggested_display_name,
        suggested_external_account_id,
    })
}

fn tdlib_user_username(user: &Value) -> Option<String> {
    user.get("username")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| {
            user.get("usernames")
                .and_then(|value| value.get("active_usernames"))
                .and_then(Value::as_array)
                .and_then(|values| {
                    values
                        .iter()
                        .filter_map(Value::as_str)
                        .find(|value| !value.trim().is_empty())
                })
                .map(str::trim)
                .map(ToOwned::to_owned)
        })
}

fn safe_account_identifier(value: &str) -> String {
    let sanitized = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '_' {
                character.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_owned();

    if sanitized.is_empty() {
        "telegram".to_owned()
    } else {
        sanitized
    }
}

pub(super) fn render_qr_svg(link: &str) -> Result<String, TelegramError> {
    let code = QrCode::new(link.as_bytes())
        .map_err(|error| TelegramError::QrGeneration(format!("failed to encode QR: {error}")))?;
    Ok(code
        .render::<svg::Color<'_>>()
        .min_dimensions(240, 240)
        .build())
}

pub(super) fn upsert_pending_response(
    pending_logins: &PendingQrLoginMap,
    response: TelegramQrLoginStatusResponse,
    command_tx: Sender<TelegramQrLoginCommand>,
    worker_completion: QrLoginWorkerCompletion,
) -> Result<(), TelegramError> {
    let mut pending_logins = pending_logins.lock().map_err(|_| {
        TelegramError::TdlibRuntime("Telegram QR login state lock was poisoned".to_owned())
    })?;
    pending_logins.insert(
        response.setup_id.clone(),
        TelegramQrLoginSession {
            response,
            command_tx,
            worker_completion,
        },
    );
    Ok(())
}

pub(super) fn mark_pending_status(
    pending_logins: &PendingQrLoginMap,
    setup_id: &str,
    status: TelegramQrLoginStatus,
    message: &str,
    poll_after_ms: u64,
) -> Result<(), TelegramError> {
    let mut pending_logins = pending_logins.lock().map_err(|_| {
        TelegramError::TdlibRuntime("Telegram QR login state lock was poisoned".to_owned())
    })?;
    if let Some(session) = pending_logins.get_mut(setup_id) {
        let response = &mut session.response;
        response.status = status;
        response.poll_after_ms = poll_after_ms;
        response.message = Some(message.to_owned());
        if !matches!(
            status,
            TelegramQrLoginStatus::WaitingQrScan | TelegramQrLoginStatus::WaitingPassword
        ) {
            response.expires_at = None;
        }
    }
    Ok(())
}

pub(super) fn mark_pending_ready_status(
    pending_logins: &PendingQrLoginMap,
    setup_id: &str,
    message: &str,
    identity: Option<&TelegramQrLoginIdentity>,
) -> Result<(), TelegramError> {
    let mut pending_logins = pending_logins.lock().map_err(|_| {
        TelegramError::TdlibRuntime("Telegram QR login state lock was poisoned".to_owned())
    })?;
    if let Some(session) = pending_logins.get_mut(setup_id) {
        let response = &mut session.response;
        response.status = TelegramQrLoginStatus::Ready;
        response.poll_after_ms = 0;
        response.message = Some(message.to_owned());
        response.expires_at = None;
        if let Some(identity) = identity {
            response.telegram_user_id = Some(identity.user_id.clone());
            response.telegram_username = identity.username.clone();
            response.suggested_account_id = Some(identity.suggested_account_id.clone());
            response.suggested_display_name = Some(identity.suggested_display_name.clone());
            response.suggested_external_account_id =
                Some(identity.suggested_external_account_id.clone());
        }
    }
    Ok(())
}

pub(super) fn password_hint(authorization_state: &Value) -> Option<String> {
    authorization_state
        .get("password_hint")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

pub(super) fn state_allows_qr_request(state_type: &str) -> bool {
    matches!(
        state_type,
        "authorizationStateWaitPhoneNumber"
            | "authorizationStateWaitPremiumPurchase"
            | "authorizationStateWaitEmailAddress"
            | "authorizationStateWaitEmailCode"
            | "authorizationStateWaitCode"
            | "authorizationStateWaitRegistration"
    )
}

pub(super) fn new_setup_id(account_id: &str) -> String {
    let timestamp = Utc::now().timestamp_nanos_opt().unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(account_id.as_bytes());
    hasher.update(b"\0");
    hasher.update(timestamp.to_string().as_bytes());
    let digest = format!("{:x}", hasher.finalize());
    format!(
        "telegram-qr-{}-{}",
        safe_path_segment(account_id),
        &digest[..16]
    )
}

pub(super) fn short_thread_suffix(account_id: &str) -> String {
    safe_path_segment(account_id).chars().take(32).collect()
}
