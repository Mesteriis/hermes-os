use std::collections::HashMap;
use std::env;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use chrono::Utc;
use libloading::Library;
use qrcode::QrCode;
use qrcode::render::svg;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use tokio::task;

use crate::integrations::telegram::client::{
    TelegramError, TelegramQrLoginPasswordRequest, TelegramQrLoginStartRequest,
    TelegramQrLoginStatus, TelegramQrLoginStatusResponse,
};
use crate::platform::config::AppConfig;

const QR_FIRST_LINK_TIMEOUT: Duration = Duration::from_secs(20);
const QR_SESSION_LIFETIME: Duration = Duration::from_secs(10 * 60);
const QR_GET_ME_TIMEOUT: Duration = Duration::from_secs(5);
const QR_POLL_AFTER_MS: u64 = 2_000;

pub(crate) type PendingQrLoginMap = Arc<Mutex<HashMap<String, TelegramQrLoginSession>>>;
type TdJsonClientCreate = unsafe extern "C" fn() -> *mut c_void;
type TdJsonClientSend = unsafe extern "C" fn(*mut c_void, *const c_char);
type TdJsonClientReceive = unsafe extern "C" fn(*mut c_void, f64) -> *const c_char;
type TdJsonClientExecute = unsafe extern "C" fn(*mut c_void, *const c_char) -> *const c_char;
type TdJsonClientDestroy = unsafe extern "C" fn(*mut c_void);

#[derive(Clone)]
pub(crate) struct TelegramQrLoginSession {
    pub(crate) response: TelegramQrLoginStatusResponse,
    command_tx: Sender<TelegramQrLoginCommand>,
}

#[derive(Debug, Eq, PartialEq)]
enum TelegramQrLoginCommand {
    CheckPassword(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct TelegramQrLoginIdentity {
    user_id: String,
    username: Option<String>,
    suggested_account_id: String,
    suggested_display_name: String,
    suggested_external_account_id: String,
}

pub(crate) fn runtime_available(configured_path: Option<&Path>) -> bool {
    TdJsonLibrary::load(configured_path).is_ok()
}

struct TdJsonLibrary {
    create: TdJsonClientCreate,
    send: TdJsonClientSend,
    receive: TdJsonClientReceive,
    execute: TdJsonClientExecute,
    destroy: TdJsonClientDestroy,
    _library: Library,
}

impl TdJsonLibrary {
    fn load(configured_path: Option<&Path>) -> Result<Self, TelegramError> {
        let candidates = tdjson_library_candidates(configured_path);
        let mut load_errors = Vec::new();

        for candidate in candidates {
            let library = {
                // SAFETY: Loading a dynamic library is unsafe because the symbols may not
                // match the expected ABI. Symbols are verified immediately below and kept
                // alive by storing the Library inside TdJsonLibrary.
                unsafe { Library::new(&candidate) }
            };
            match library {
                Ok(library) => return Self::from_library(library, &candidate),
                Err(error) => {
                    load_errors.push(format!("{}: {error}", candidate.display()));
                    if configured_path.is_some() {
                        break;
                    }
                }
            }
        }

        Err(TelegramError::TdlibRuntimeUnavailable(format!(
            "unable to load libtdjson; tried {}",
            load_errors.join("; ")
        )))
    }

    fn from_library(library: Library, candidate: &Path) -> Result<Self, TelegramError> {
        let create = load_symbol(&library, b"td_json_client_create\0", candidate)?;
        let send = load_symbol(&library, b"td_json_client_send\0", candidate)?;
        let receive = load_symbol(&library, b"td_json_client_receive\0", candidate)?;
        let execute = load_symbol(&library, b"td_json_client_execute\0", candidate)?;
        let destroy = load_symbol(&library, b"td_json_client_destroy\0", candidate)?;

        Ok(Self {
            create,
            send,
            receive,
            execute,
            destroy,
            _library: library,
        })
    }

    fn create_client(self) -> Result<TdJsonClient, TelegramError> {
        let client = {
            // SAFETY: The function pointer was loaded from libtdjson with the documented
            // C ABI and returns an opaque TDLib client pointer owned by the caller.
            unsafe { (self.create)() }
        };
        if client.is_null() {
            return Err(TelegramError::TdlibRuntime(
                "td_json_client_create returned null".to_owned(),
            ));
        }

        Ok(TdJsonClient {
            client,
            library: self,
        })
    }
}

struct TdJsonClient {
    client: *mut c_void,
    library: TdJsonLibrary,
}

impl TdJsonClient {
    fn send_json(&self, request: &Value) -> Result<(), TelegramError> {
        let request = CString::new(request.to_string()).map_err(|_| {
            TelegramError::TdlibRuntime("TDLib request contained an interior NUL byte".to_owned())
        })?;
        // SAFETY: The client pointer is created by td_json_client_create and remains
        // valid until Drop calls td_json_client_destroy. CString is NUL-terminated
        // and lives for the duration of the call.
        unsafe {
            (self.library.send)(self.client, request.as_ptr());
        }
        Ok(())
    }

    fn receive_json(&self, timeout_seconds: f64) -> Result<Option<Value>, TelegramError> {
        let response = {
            // SAFETY: TDLib owns the returned pointer until the next receive/execute
            // call on this client. The string is copied into an owned Rust String
            // before another TDLib call can invalidate it.
            unsafe { (self.library.receive)(self.client, timeout_seconds) }
        };
        if response.is_null() {
            return Ok(None);
        }

        let response = {
            // SAFETY: td_json_client_receive returns a NUL-terminated UTF-8 JSON string
            // pointer or NULL. NULL was handled above.
            unsafe { CStr::from_ptr(response) }
        }
        .to_str()
        .map_err(|error| TelegramError::TdlibRuntime(format!("invalid TDLib JSON UTF-8: {error}")))?
        .to_owned();

        serde_json::from_str(&response)
            .map(Some)
            .map_err(|error| TelegramError::TdlibRuntime(format!("invalid TDLib JSON: {error}")))
    }

    fn execute_json(&self, request: &Value) -> Result<Option<Value>, TelegramError> {
        let request = CString::new(request.to_string()).map_err(|_| {
            TelegramError::TdlibRuntime("TDLib request contained an interior NUL byte".to_owned())
        })?;
        let response = {
            // SAFETY: The client pointer and request CString satisfy td_json_client_execute
            // requirements. The returned pointer is copied before the next TDLib call.
            unsafe { (self.library.execute)(self.client, request.as_ptr()) }
        };
        if response.is_null() {
            return Ok(None);
        }

        let response = {
            // SAFETY: td_json_client_execute returns a NUL-terminated JSON string pointer
            // or NULL. NULL was handled above.
            unsafe { CStr::from_ptr(response) }
        }
        .to_str()
        .map_err(|error| TelegramError::TdlibRuntime(format!("invalid TDLib JSON UTF-8: {error}")))?
        .to_owned();

        serde_json::from_str(&response)
            .map(Some)
            .map_err(|error| TelegramError::TdlibRuntime(format!("invalid TDLib JSON: {error}")))
    }
}

impl Drop for TdJsonClient {
    fn drop(&mut self) {
        if !self.client.is_null() {
            // SAFETY: The pointer was created by td_json_client_create and is destroyed
            // exactly once here, before the backing library is unloaded.
            unsafe {
                (self.library.destroy)(self.client);
            }
            self.client = std::ptr::null_mut();
        }
    }
}

pub(crate) async fn start_qr_login(
    config: AppConfig,
    pending_logins: PendingQrLoginMap,
    request: TelegramQrLoginStartRequest,
) -> Result<TelegramQrLoginStatusResponse, TelegramError> {
    request.validate()?;
    task::spawn_blocking(move || start_qr_login_driver(config, pending_logins, request))
        .await
        .map_err(|error| {
            TelegramError::TdlibRuntime(format!("Telegram QR login worker failed: {error}"))
        })?
}

pub(crate) fn submit_qr_login_password(
    pending_logins: PendingQrLoginMap,
    setup_id: &str,
    request: TelegramQrLoginPasswordRequest,
) -> Result<TelegramQrLoginStatusResponse, TelegramError> {
    let setup_id = setup_id.trim();
    if setup_id.is_empty() {
        return Err(TelegramError::InvalidRequest(
            "setup_id must not be empty".to_owned(),
        ));
    }
    if request.password.is_empty() {
        return Err(TelegramError::InvalidRequest(
            "password must not be empty".to_owned(),
        ));
    }

    let mut pending_logins = pending_logins.lock().map_err(|_| {
        TelegramError::TdlibRuntime("Telegram QR login state lock was poisoned".to_owned())
    })?;
    let session = pending_logins
        .get_mut(setup_id)
        .ok_or(TelegramError::QrLoginNotFound)?;
    if session.response.status != TelegramQrLoginStatus::WaitingPassword {
        return Err(TelegramError::InvalidRequest(
            "Telegram QR login is not waiting for a password".to_owned(),
        ));
    }

    session
        .command_tx
        .send(TelegramQrLoginCommand::CheckPassword(request.password))
        .map_err(|_| {
            TelegramError::TdlibRuntime(
                "Telegram QR login worker is no longer accepting password commands".to_owned(),
            )
        })?;
    session.response.message = Some("Checking Telegram password.".to_owned());
    session.response.poll_after_ms = QR_POLL_AFTER_MS;

    Ok(session.response.clone())
}

fn start_qr_login_driver(
    config: AppConfig,
    pending_logins: PendingQrLoginMap,
    request: TelegramQrLoginStartRequest,
) -> Result<TelegramQrLoginStatusResponse, TelegramError> {
    let (first_response_tx, first_response_rx) = mpsc::channel();
    let (command_tx, command_rx) = mpsc::channel();
    let setup_id = new_setup_id(&request.account_id);
    let thread_name = format!(
        "telegram-qr-login-{}",
        short_thread_suffix(&request.account_id)
    );

    thread::Builder::new()
        .name(thread_name)
        .spawn({
            let setup_id = setup_id.clone();
            let pending_logins = Arc::clone(&pending_logins);
            move || {
                let mut first_response_tx = Some(first_response_tx);
                let result = drive_qr_login(
                    config,
                    pending_logins.clone(),
                    request,
                    setup_id,
                    command_tx,
                    command_rx,
                    &mut first_response_tx,
                );
                if let Err(error) = result {
                    if let Some(first_response_tx) = first_response_tx.take() {
                        let _ = first_response_tx.send(Err(error));
                    } else {
                        tracing::warn!(error = %error, "Telegram QR login failed after QR link was issued");
                    }
                }
            }
        })
        .map_err(|error| {
            TelegramError::TdlibRuntime(format!("failed to spawn Telegram QR login worker: {error}"))
        })?;

    first_response_rx
        .recv_timeout(QR_FIRST_LINK_TIMEOUT)
        .map_err(|error| {
            TelegramError::TdlibRuntime(format!(
                "Telegram TDLib did not return a QR confirmation link within {} seconds: {error}",
                QR_FIRST_LINK_TIMEOUT.as_secs()
            ))
        })?
}

fn drive_qr_login(
    config: AppConfig,
    pending_logins: PendingQrLoginMap,
    request: TelegramQrLoginStartRequest,
    setup_id: String,
    command_tx: Sender<TelegramQrLoginCommand>,
    command_rx: Receiver<TelegramQrLoginCommand>,
    first_response_tx: &mut Option<Sender<Result<TelegramQrLoginStatusResponse, TelegramError>>>,
) -> Result<(), TelegramError> {
    let library = TdJsonLibrary::load(config.tdjson_path())?;
    let client = library.create_client()?;
    let database_directory = tdlib_database_directory(&request);
    let files_directory = database_directory.join("files");
    std::fs::create_dir_all(&files_directory).map_err(|error| {
        TelegramError::TdlibRuntime(format!(
            "failed to create TDLib data directory `{}`: {error}",
            files_directory.display()
        ))
    })?;

    let _ = client.execute_json(&json!({
        "@type": "setLogVerbosityLevel",
        "new_verbosity_level": 1
    }));
    client.send_json(&json!({
        "@type": "getAuthorizationState",
        "@extra": "hermes-initial-authorization-state"
    }))?;

    let started_at = Instant::now();
    let mut tdlib_parameters_sent = false;
    let mut database_encryption_key_checked = false;
    let mut qr_requested = false;
    let mut password_check_in_flight = false;

    loop {
        if drain_qr_login_commands(&client, &command_rx)? {
            password_check_in_flight = true;
            mark_pending_status(
                &pending_logins,
                &setup_id,
                TelegramQrLoginStatus::WaitingPassword,
                "Checking Telegram password.",
                QR_POLL_AFTER_MS,
            )?;
        }

        if first_response_tx.is_some() && started_at.elapsed() > QR_FIRST_LINK_TIMEOUT {
            return Err(TelegramError::TdlibRuntime(format!(
                "Telegram TDLib did not enter QR confirmation state within {} seconds",
                QR_FIRST_LINK_TIMEOUT.as_secs()
            )));
        }
        if started_at.elapsed() > QR_SESSION_LIFETIME {
            mark_pending_status(
                &pending_logins,
                &setup_id,
                TelegramQrLoginStatus::Expired,
                "Telegram QR login session expired; start a new QR login.",
                0,
            )?;
            let _ = client.send_json(&json!({ "@type": "close" }));
            return Ok(());
        }

        let Some(event) = client.receive_json(1.0)? else {
            continue;
        };

        if is_tdlib_parameters_not_specified_error(&event) {
            if !tdlib_parameters_sent {
                send_tdlib_parameters(&client, &request, &database_directory)?;
                tdlib_parameters_sent = true;
            }
            continue;
        }
        if is_tdlib_database_encryption_key_needed_error(&event) {
            if !database_encryption_key_checked {
                client.send_json(&check_database_encryption_key_request(&request))?;
                database_encryption_key_checked = true;
            }
            continue;
        }

        if let Some(message) = tdlib_error_message(&event) {
            if password_check_in_flight {
                password_check_in_flight = false;
                mark_pending_status(
                    &pending_logins,
                    &setup_id,
                    TelegramQrLoginStatus::WaitingPassword,
                    "Telegram password was rejected. Try again.",
                    QR_POLL_AFTER_MS,
                )?;
                continue;
            }
            return Err(TelegramError::TdlibRuntime(message));
        }

        let Some(authorization_state) = authorization_state(&event) else {
            continue;
        };
        let Some(state_type) = authorization_state.get("@type").and_then(Value::as_str) else {
            continue;
        };

        match state_type {
            "authorizationStateWaitTdlibParameters" => {
                send_tdlib_parameters(&client, &request, &database_directory)?;
                tdlib_parameters_sent = true;
            }
            "authorizationStateWaitEncryptionKey" if !database_encryption_key_checked => {
                client.send_json(&check_database_encryption_key_request(&request))?;
                database_encryption_key_checked = true;
            }
            state if state_allows_qr_request(state) && !qr_requested => {
                client.send_json(&json!({
                    "@type": "requestQrCodeAuthentication",
                    "other_user_ids": [],
                    "@extra": "hermes-request-qr-code-authentication"
                }))?;
                qr_requested = true;
            }
            "authorizationStateWaitOtherDeviceConfirmation" => {
                let link = authorization_state
                    .get("link")
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .ok_or_else(|| {
                        TelegramError::TdlibRuntime(
                            "TDLib QR authorization state did not include a link".to_owned(),
                        )
                    })?;
                let response = qr_waiting_response(&setup_id, &request.account_id, link)?;
                upsert_pending_response(&pending_logins, response.clone(), command_tx.clone())?;
                if let Some(first_response_tx) = first_response_tx.take() {
                    let _ = first_response_tx.send(Ok(response));
                }
            }
            "authorizationStateWaitPassword" => {
                password_check_in_flight = false;
                let password_hint = password_hint(authorization_state);
                let message = password_hint
                    .as_deref()
                    .map(|hint| {
                        format!("Telegram requires your 2-step verification password. Hint: {hint}")
                    })
                    .unwrap_or_else(|| {
                        "Telegram requires your 2-step verification password.".to_owned()
                    });
                if first_response_tx.is_some() {
                    let response =
                        password_waiting_response(&setup_id, &request.account_id, &message);
                    upsert_pending_response(&pending_logins, response.clone(), command_tx.clone())?;
                    if let Some(first_response_tx) = first_response_tx.take() {
                        let _ = first_response_tx.send(Ok(response));
                    }
                } else {
                    mark_pending_status(
                        &pending_logins,
                        &setup_id,
                        TelegramQrLoginStatus::WaitingPassword,
                        &message,
                        QR_POLL_AFTER_MS,
                    )?;
                }
            }
            "authorizationStateReady" => {
                let identity = match fetch_authorized_user_identity(&client) {
                    Ok(identity) => identity,
                    Err(error) => {
                        tracing::warn!(
                            error = %error,
                            "Telegram QR login completed, but TDLib user identity lookup failed"
                        );
                        None
                    }
                };
                let message = if qr_requested {
                    "Telegram QR login confirmed on the other device."
                } else {
                    "Telegram TDLib session is already authorized."
                };
                if first_response_tx.is_some() {
                    let response =
                        ready_response(&setup_id, &request.account_id, message, identity.as_ref());
                    upsert_pending_response(&pending_logins, response.clone(), command_tx.clone())?;
                    if let Some(first_response_tx) = first_response_tx.take() {
                        let _ = first_response_tx.send(Ok(response));
                    }
                } else {
                    mark_pending_ready_status(
                        &pending_logins,
                        &setup_id,
                        message,
                        identity.as_ref(),
                    )?;
                }
                let _ = client.send_json(&json!({ "@type": "close" }));
                return Ok(());
            }
            "authorizationStateClosed" => {
                mark_pending_status(
                    &pending_logins,
                    &setup_id,
                    TelegramQrLoginStatus::Failed,
                    "Telegram TDLib authorization session closed before QR login completed.",
                    0,
                )?;
                return Ok(());
            }
            "authorizationStateClosing" | "authorizationStateLoggingOut" => {
                mark_pending_status(
                    &pending_logins,
                    &setup_id,
                    TelegramQrLoginStatus::Failed,
                    "Telegram TDLib authorization session is closing.",
                    0,
                )?;
                return Ok(());
            }
            unsupported if qr_requested => {
                mark_pending_status(
                    &pending_logins,
                    &setup_id,
                    TelegramQrLoginStatus::Failed,
                    &format!(
                        "Telegram QR login requires unsupported authorization state `{unsupported}`."
                    ),
                    0,
                )?;
                return Ok(());
            }
            _ => {}
        }
    }
}

fn drain_qr_login_commands(
    client: &TdJsonClient,
    command_rx: &Receiver<TelegramQrLoginCommand>,
) -> Result<bool, TelegramError> {
    let mut password_submitted = false;
    loop {
        match command_rx.try_recv() {
            Ok(TelegramQrLoginCommand::CheckPassword(password)) => {
                client.send_json(&json!({
                    "@type": "checkAuthenticationPassword",
                    "password": password,
                    "@extra": "hermes-check-authentication-password"
                }))?;
                password_submitted = true;
            }
            Err(TryRecvError::Empty | TryRecvError::Disconnected) => {
                return Ok(password_submitted);
            }
        }
    }
}

fn send_tdlib_parameters(
    client: &TdJsonClient,
    request: &TelegramQrLoginStartRequest,
    database_directory: &Path,
) -> Result<(), TelegramError> {
    client.send_json(&set_tdlib_parameters_request(request, database_directory)?)
}

fn load_symbol<T: Copy>(
    library: &Library,
    name: &'static [u8],
    candidate: &Path,
) -> Result<T, TelegramError> {
    let symbol = {
        // SAFETY: Symbol type T is the exact C ABI function pointer expected for the
        // named TDLib JSON symbol. The Library is retained for at least as long as
        // copied function pointers can be called.
        unsafe { library.get::<T>(name) }
    }
    .map_err(|error| {
        let symbol_name = name.strip_suffix(b"\0").unwrap_or(name);
        TelegramError::TdlibRuntimeUnavailable(format!(
            "libtdjson `{}` is missing symbol `{}`: {error}",
            candidate.display(),
            String::from_utf8_lossy(symbol_name)
        ))
    })?;
    Ok(*symbol)
}

fn tdjson_library_candidates(configured_path: Option<&Path>) -> Vec<PathBuf> {
    let current_exe_dir = env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf));
    let current_dir = env::current_dir().ok();

    tdjson_library_candidates_with_context(
        configured_path,
        current_exe_dir.as_deref(),
        current_dir.as_deref(),
    )
}

fn tdjson_library_candidates_with_context(
    configured_path: Option<&Path>,
    current_exe_dir: Option<&Path>,
    current_dir: Option<&Path>,
) -> Vec<PathBuf> {
    if let Some(path) = configured_path {
        return vec![path.to_path_buf()];
    }

    let mut candidates = Vec::new();
    add_bundled_tdjson_candidates(&mut candidates, current_exe_dir, current_dir);

    #[cfg(target_os = "macos")]
    {
        push_unique_candidate(&mut candidates, PathBuf::from(tdjson_library_file_name()));
        push_unique_candidate(
            &mut candidates,
            PathBuf::from("/opt/homebrew/opt/tdlib/lib/libtdjson.dylib"),
        );
        push_unique_candidate(
            &mut candidates,
            PathBuf::from("/opt/homebrew/lib/libtdjson.dylib"),
        );
        push_unique_candidate(
            &mut candidates,
            PathBuf::from("/usr/local/opt/tdlib/lib/libtdjson.dylib"),
        );
        push_unique_candidate(
            &mut candidates,
            PathBuf::from("/usr/local/lib/libtdjson.dylib"),
        );
    }
    #[cfg(target_os = "linux")]
    {
        push_unique_candidate(&mut candidates, PathBuf::from(tdjson_library_file_name()));
        push_unique_candidate(
            &mut candidates,
            PathBuf::from("/usr/local/lib/libtdjson.so"),
        );
        push_unique_candidate(&mut candidates, PathBuf::from("/usr/lib/libtdjson.so"));
        push_unique_candidate(
            &mut candidates,
            PathBuf::from("/usr/lib/x86_64-linux-gnu/libtdjson.so"),
        );
    }
    #[cfg(target_os = "windows")]
    {
        push_unique_candidate(&mut candidates, PathBuf::from(tdjson_library_file_name()));
    }

    candidates
}

fn add_bundled_tdjson_candidates(
    candidates: &mut Vec<PathBuf>,
    current_exe_dir: Option<&Path>,
    current_dir: Option<&Path>,
) {
    let library_file_name = tdjson_library_file_name();
    let platform_dir = tdjson_platform_dir();

    if let Some(exe_dir) = current_exe_dir {
        #[cfg(target_os = "macos")]
        if let Some(contents_dir) = exe_dir.parent() {
            add_tdjson_resource_dir_candidates(
                candidates,
                &contents_dir.join("Resources").join("tdlib"),
                platform_dir,
                library_file_name,
            );
        }

        add_tdjson_resource_dir_candidates(
            candidates,
            &exe_dir.join("resources").join("tdlib"),
            platform_dir,
            library_file_name,
        );
        add_tdjson_resource_dir_candidates(
            candidates,
            &exe_dir.join("tdlib"),
            platform_dir,
            library_file_name,
        );
    }

    if let Some(current_dir) = current_dir {
        add_tdjson_resource_dir_candidates(
            candidates,
            &current_dir.join("frontend/src-tauri/resources/tdlib"),
            platform_dir,
            library_file_name,
        );
        add_tdjson_resource_dir_candidates(
            candidates,
            &current_dir.join("resources/tdlib"),
            platform_dir,
            library_file_name,
        );
    }
}

fn add_tdjson_resource_dir_candidates(
    candidates: &mut Vec<PathBuf>,
    tdlib_dir: &Path,
    platform_dir: &str,
    library_file_name: &str,
) {
    push_unique_candidate(
        candidates,
        tdlib_dir.join(platform_dir).join(library_file_name),
    );

    #[cfg(target_os = "macos")]
    push_unique_candidate(
        candidates,
        tdlib_dir.join("macos-universal").join(library_file_name),
    );

    push_unique_candidate(candidates, tdlib_dir.join(library_file_name));
}

fn push_unique_candidate(candidates: &mut Vec<PathBuf>, candidate: PathBuf) {
    if !candidates.contains(&candidate) {
        candidates.push(candidate);
    }
}

fn tdjson_platform_dir() -> &'static str {
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        return "macos-arm64";
    }
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    {
        return "macos-x64";
    }
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        return "linux-x64";
    }
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    {
        return "linux-arm64";
    }
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    {
        return "windows-x64";
    }
    #[cfg(all(target_os = "windows", target_arch = "aarch64"))]
    {
        return "windows-arm64";
    }
    #[allow(unreachable_code)]
    "unknown"
}

fn tdjson_library_file_name() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        return "libtdjson.dylib";
    }
    #[cfg(target_os = "linux")]
    {
        return "libtdjson.so";
    }
    #[cfg(target_os = "windows")]
    {
        return "tdjson.dll";
    }
    #[allow(unreachable_code)]
    "libtdjson"
}

fn set_tdlib_parameters_request(
    request: &TelegramQrLoginStartRequest,
    database_directory: &Path,
) -> Result<Value, TelegramError> {
    let api_id = request.required_api_id()?;
    let api_hash = request.required_api_hash()?;
    let database_directory = database_directory.to_string_lossy().into_owned();
    let files_directory = Path::new(&database_directory)
        .join("files")
        .to_string_lossy()
        .into_owned();

    let parameters = json!({
        "use_test_dc": false,
        "database_directory": database_directory,
        "files_directory": files_directory,
        "use_file_database": true,
        "use_chat_info_database": true,
        "use_message_database": true,
        "use_secret_chats": false,
        "api_id": api_id,
        "api_hash": api_hash,
        "system_language_code": "en",
        "device_model": "Hermes Hub",
        "system_version": std::env::consts::OS,
        "application_version": env!("CARGO_PKG_VERSION"),
        "enable_storage_optimizer": true,
        "ignore_file_names": false
    });

    Ok(json!({
        "@type": "setTdlibParameters",
        "parameters": parameters,
        "@extra": "hermes-set-tdlib-parameters"
    }))
}

fn tdlib_database_encryption_key(request: &TelegramQrLoginStartRequest) -> String {
    request
        .session_encryption_key
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| STANDARD.encode(value.as_bytes()))
        .unwrap_or_default()
}

fn tdlib_database_directory(request: &TelegramQrLoginStartRequest) -> PathBuf {
    request
        .tdlib_data_path
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from("docker/data/telegram").join(safe_path_segment(&request.account_id))
        })
}

fn check_database_encryption_key_request(request: &TelegramQrLoginStartRequest) -> Value {
    json!({
        "@type": "checkDatabaseEncryptionKey",
        "encryption_key": tdlib_database_encryption_key(request),
        "@extra": "hermes-check-database-encryption-key"
    })
}

fn qr_waiting_response(
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

fn password_waiting_response(
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

fn ready_response(
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

fn fetch_authorized_user_identity(
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

fn parse_tdlib_user_identity(user: &Value) -> Option<TelegramQrLoginIdentity> {
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

fn render_qr_svg(link: &str) -> Result<String, TelegramError> {
    let code = QrCode::new(link.as_bytes())
        .map_err(|error| TelegramError::QrGeneration(format!("failed to encode QR: {error}")))?;
    Ok(code
        .render::<svg::Color<'_>>()
        .min_dimensions(240, 240)
        .build())
}

fn upsert_pending_response(
    pending_logins: &PendingQrLoginMap,
    response: TelegramQrLoginStatusResponse,
    command_tx: Sender<TelegramQrLoginCommand>,
) -> Result<(), TelegramError> {
    let mut pending_logins = pending_logins.lock().map_err(|_| {
        TelegramError::TdlibRuntime("Telegram QR login state lock was poisoned".to_owned())
    })?;
    pending_logins.insert(
        response.setup_id.clone(),
        TelegramQrLoginSession {
            response,
            command_tx,
        },
    );
    Ok(())
}

fn mark_pending_status(
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

fn mark_pending_ready_status(
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

fn password_hint(authorization_state: &Value) -> Option<String> {
    authorization_state
        .get("password_hint")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn authorization_state(event: &Value) -> Option<&Value> {
    match event.get("@type").and_then(Value::as_str) {
        Some("updateAuthorizationState") => event.get("authorization_state"),
        Some(value) if value.starts_with("authorizationState") => Some(event),
        _ => None,
    }
}

fn is_tdlib_parameters_not_specified_error(event: &Value) -> bool {
    event.get("@type").and_then(Value::as_str) == Some("error")
        && event.get("code").and_then(Value::as_i64) == Some(400)
        && event.get("message").and_then(Value::as_str) == Some("Parameters aren't specified")
}

fn is_tdlib_database_encryption_key_needed_error(event: &Value) -> bool {
    event.get("@type").and_then(Value::as_str) == Some("error")
        && event.get("code").and_then(Value::as_i64) == Some(400)
        && event
            .get("message")
            .and_then(Value::as_str)
            .is_some_and(|message| {
                message.contains("Database encryption key is needed")
                    && message.contains("checkDatabaseEncryptionKey")
            })
}

fn state_allows_qr_request(state_type: &str) -> bool {
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

fn tdlib_error_message(event: &Value) -> Option<String> {
    if event.get("@type").and_then(Value::as_str) != Some("error") {
        return None;
    }

    let code = event
        .get("code")
        .and_then(Value::as_i64)
        .map(|value| value.to_string())
        .unwrap_or_else(|| "unknown".to_owned());
    let message = event
        .get("message")
        .and_then(Value::as_str)
        .unwrap_or("TDLib returned an error");

    Some(format!("TDLib error {code}: {message}"))
}

fn new_setup_id(account_id: &str) -> String {
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

fn safe_path_segment(value: &str) -> String {
    let sanitized = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_owned();

    if sanitized.is_empty() {
        "account".to_owned()
    } else {
        sanitized
    }
}

fn short_thread_suffix(account_id: &str) -> String {
    safe_path_segment(account_id).chars().take(32).collect()
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::Path;
    use std::sync::{Arc, Mutex, mpsc};

    use base64::Engine as _;
    use base64::engine::general_purpose::STANDARD;
    use serde_json::json;

    use crate::integrations::telegram::client::{
        TelegramError, TelegramQrLoginPasswordRequest, TelegramQrLoginStartRequest,
        TelegramQrLoginStatus, TelegramQrLoginStatusResponse,
    };

    use super::{
        TelegramQrLoginCommand, TelegramQrLoginSession, check_database_encryption_key_request,
        render_qr_svg, set_tdlib_parameters_request,
    };

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_tdjson_candidates_prefer_bundled_tauri_resources() {
        let exe_dir = Path::new("/Applications/Hermes Hub.app/Contents/MacOS");
        let cwd = Path::new("/workspace/hermes-hub");
        let candidates =
            super::tdjson_library_candidates_with_context(None, Some(exe_dir), Some(cwd));
        let bundled_resource = Path::new("/Applications/Hermes Hub.app/Contents/Resources")
            .join("tdlib")
            .join(super::tdjson_platform_dir())
            .join("libtdjson.dylib");
        let dev_resource = cwd
            .join("frontend/src-tauri/resources/tdlib")
            .join(super::tdjson_platform_dir())
            .join("libtdjson.dylib");

        assert_eq!(candidates.first(), Some(&bundled_resource));
        assert!(candidates.contains(&dev_resource));
        assert!(
            candidates
                .iter()
                .position(|candidate| candidate == &bundled_resource)
                < candidates.iter().position(
                    |candidate| candidate == Path::new("/opt/homebrew/lib/libtdjson.dylib")
                )
        );
    }

    #[test]
    fn renders_tdlib_qr_link_as_svg() {
        let svg = render_qr_svg("tg://login?token=test-token").expect("QR SVG");

        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
        assert!(svg.len() > 100);
    }

    #[test]
    fn tdlib_parameters_use_legacy_nested_shape_for_tdlib_1_8_runtime() {
        let request = TelegramQrLoginStartRequest {
            account_id: "telegram-qr".to_owned(),
            display_name: "Telegram QR".to_owned(),
            external_account_id: "qr-login:telegram-qr".to_owned(),
            api_id: Some(12345),
            api_hash: Some("telegram-api-hash".to_owned()),
            session_encryption_key: Some("telegram-session-key".to_owned()),
            tdlib_data_path: Some("docker/data/telegram/telegram-qr".to_owned()),
            transcription_enabled: true,
        };

        let command =
            set_tdlib_parameters_request(&request, Path::new("docker/data/telegram/telegram-qr"))
                .expect("TDLib parameters");

        assert_eq!(command["@type"], "setTdlibParameters");
        assert_eq!(command["parameters"]["api_id"], 12345);
        assert_eq!(command["parameters"]["api_hash"], "telegram-api-hash");
        assert_eq!(command["parameters"]["enable_storage_optimizer"], true);
        assert_eq!(command["parameters"]["ignore_file_names"], false);
        assert_eq!(command["database_encryption_key"], serde_json::Value::Null);
        assert_eq!(
            command["parameters"]["database_encryption_key"],
            serde_json::Value::Null
        );
    }

    #[test]
    fn tdlib_database_key_check_uses_same_base64_key_without_plaintext_secret() {
        let request = TelegramQrLoginStartRequest {
            account_id: "telegram-qr".to_owned(),
            display_name: "Telegram QR".to_owned(),
            external_account_id: "qr-login:telegram-qr".to_owned(),
            api_id: Some(12345),
            api_hash: Some("telegram-api-hash".to_owned()),
            session_encryption_key: Some("telegram-session-key".to_owned()),
            tdlib_data_path: Some("docker/data/telegram/telegram-qr".to_owned()),
            transcription_enabled: true,
        };

        let command = check_database_encryption_key_request(&request);

        assert_eq!(command["@type"], "checkDatabaseEncryptionKey");
        assert_eq!(
            command["encryption_key"],
            STANDARD.encode("telegram-session-key")
        );
        assert_ne!(command["encryption_key"], "telegram-session-key");
    }

    #[test]
    fn recognizes_tdlib_bootstrap_error_events() {
        assert!(super::is_tdlib_parameters_not_specified_error(&json!({
            "@type": "error",
            "code": 400,
            "message": "Parameters aren't specified"
        })));
        assert!(super::is_tdlib_database_encryption_key_needed_error(
            &json!({
                "@type": "error",
                "code": 400,
                "message": "Database encryption key is needed: call checkDatabaseEncryptionKey first"
            })
        ));
    }

    #[test]
    fn parses_tdlib_user_identity_for_qr_account_defaults() {
        let identity = super::parse_tdlib_user_identity(&json!({
            "@type": "user",
            "id": 123456789,
            "usernames": {
                "active_usernames": ["Test_User"]
            }
        }))
        .expect("identity");

        assert_eq!(identity.user_id, "123456789");
        assert_eq!(identity.username.as_deref(), Some("Test_User"));
        assert_eq!(identity.suggested_account_id, "123456789_account_test_user");
        assert_eq!(identity.suggested_display_name, "@Test_User");
        assert_eq!(identity.suggested_external_account_id, "telegram:123456789");
    }

    #[test]
    fn wait_password_state_is_not_a_qr_request_state() {
        assert!(!super::state_allows_qr_request(
            "authorizationStateWaitPassword"
        ));
    }

    #[test]
    fn password_waiting_response_does_not_expose_stale_qr_token() {
        let response = super::password_waiting_response(
            "setup-id",
            "telegram-account",
            "Telegram requires your 2-step verification password.",
        );

        assert_eq!(response.status, TelegramQrLoginStatus::WaitingPassword);
        assert_eq!(response.qr_link, None);
        assert_eq!(response.qr_svg, None);
        assert_eq!(response.poll_after_ms, 2_000);
    }

    #[test]
    fn ready_response_for_existing_tdlib_session_does_not_expose_qr_token() {
        let identity = super::TelegramQrLoginIdentity {
            user_id: "123456789".to_owned(),
            username: Some("Test_User".to_owned()),
            suggested_account_id: "123456789_account_test_user".to_owned(),
            suggested_display_name: "@Test_User".to_owned(),
            suggested_external_account_id: "telegram:123456789".to_owned(),
        };

        let response = super::ready_response(
            "setup-id",
            "telegram-account",
            "Telegram TDLib session is already authorized.",
            Some(&identity),
        );

        assert_eq!(response.status, TelegramQrLoginStatus::Ready);
        assert_eq!(response.qr_link, None);
        assert_eq!(response.qr_svg, None);
        assert_eq!(
            response.suggested_account_id.as_deref(),
            Some("123456789_account_test_user")
        );
        assert_eq!(
            response.suggested_display_name.as_deref(),
            Some("@Test_User")
        );
        assert_eq!(
            response.suggested_external_account_id.as_deref(),
            Some("telegram:123456789")
        );
    }

    #[test]
    fn qr_password_submission_sends_command_to_pending_session() {
        let (command_tx, command_rx) = mpsc::channel();
        let pending = Arc::new(Mutex::new(HashMap::from([(
            "setup-id".to_owned(),
            TelegramQrLoginSession {
                response: test_qr_login_response(TelegramQrLoginStatus::WaitingPassword),
                command_tx,
            },
        )])));

        let login_check_value = "tdlib-check-value".to_owned();

        let response = super::submit_qr_login_password(
            pending,
            "setup-id",
            TelegramQrLoginPasswordRequest {
                password: login_check_value.clone(),
            },
        )
        .expect("password accepted");

        assert_eq!(response.status, TelegramQrLoginStatus::WaitingPassword);
        assert_eq!(
            response.message.as_deref(),
            Some("Checking Telegram password.")
        );
        assert_eq!(
            command_rx.try_recv().expect("password command"),
            TelegramQrLoginCommand::CheckPassword(login_check_value)
        );
    }

    #[test]
    fn qr_password_submission_requires_waiting_password_status() {
        let (command_tx, command_rx) = mpsc::channel();
        let pending = Arc::new(Mutex::new(HashMap::from([(
            "setup-id".to_owned(),
            TelegramQrLoginSession {
                response: test_qr_login_response(TelegramQrLoginStatus::WaitingQrScan),
                command_tx,
            },
        )])));

        let login_check_value = "tdlib-check-value".to_owned();

        let error = super::submit_qr_login_password(
            pending,
            "setup-id",
            TelegramQrLoginPasswordRequest {
                password: login_check_value,
            },
        )
        .expect_err("password must not be accepted before TDLib asks for it");

        assert!(matches!(error, TelegramError::InvalidRequest(_)));
        assert!(command_rx.try_recv().is_err());
    }

    fn test_qr_login_response(status: TelegramQrLoginStatus) -> TelegramQrLoginStatusResponse {
        TelegramQrLoginStatusResponse {
            setup_id: "setup-id".to_owned(),
            account_id: "telegram-account".to_owned(),
            status,
            qr_link: Some("tg://login?token=test-token".to_owned()),
            qr_svg: Some("<svg></svg>".to_owned()),
            telegram_user_id: None,
            telegram_username: None,
            suggested_account_id: None,
            suggested_display_name: None,
            suggested_external_account_id: None,
            expires_at: None,
            poll_after_ms: 2_000,
            message: Some("Waiting".to_owned()),
        }
    }
}
