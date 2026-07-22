//! Long-lived Telegram process orchestration around the provider runtime.

use std::os::unix::net::UnixStream;
use std::os::unix::net::UnixListener;
use std::path::Path;
use std::time::Duration;

use hermes_telegram_persistence::{
    TelegramDurablePersistence, TelegramDurablePersistenceError,
};
use hermes_telegram_tdlib::{TdlibAuthorizationEvent, TdlibError};
use hermes_telegram_tdlib::TdlibAuthorizationUpdate;

use crate::{
    TelegramDurableProjectionError, TelegramRuntimeComposition,
    bootstrap::TelegramAdmittedRuntime,
    client_transport::{self, TelegramClientTransportError},
};

#[derive(Debug)]
pub enum TelegramProcessTick {
    Authorization(Option<TdlibAuthorizationEvent>),
    Runtime {
        frames: usize,
        provider_cursor: Option<String>,
    },
    Idle,
}

#[derive(Debug)]
pub enum TelegramDurableProcessError {
    Provider(TdlibError),
    Persistence(TelegramDurablePersistenceError),
    Projection(TelegramDurableProjectionError),
}

pub struct TelegramProcessLoop {
    composition: TelegramRuntimeComposition,
    provider_cursor: Option<String>,
    authorization_status: Option<hermes_telegram_api::TelegramAuthorizationStatus>,
}

impl TelegramProcessLoop {
    #[must_use]
    pub fn new(composition: TelegramRuntimeComposition) -> Self {
        Self {
            composition,
            provider_cursor: None,
            authorization_status: None,
        }
    }

    pub fn composition_mut(&mut self) -> &mut TelegramRuntimeComposition {
        &mut self.composition
    }

    #[must_use]
    pub fn composition(&self) -> &TelegramRuntimeComposition {
        &self.composition
    }

    #[must_use]
    pub fn authorization_status(&self) -> Option<&hermes_telegram_api::TelegramAuthorizationStatus> {
        self.authorization_status.as_ref()
    }

    pub fn serve_client_connection(
        &mut self,
        stream: UnixStream,
    ) -> Result<(), TelegramClientTransportError> {
        let runtime = self
            .composition
            .runtime_mut()
            .ok_or(TelegramClientTransportError::RuntimeUnavailable)?;
        client_transport::serve_connection(stream, runtime)
    }

    pub fn serve_client_connection_durable(
        &mut self,
        stream: UnixStream,
        durable: &TelegramDurablePersistence,
        handle: &tokio::runtime::Handle,
    ) -> Result<(), TelegramClientTransportError> {
        let runtime = self
            .composition
            .runtime_mut()
            .ok_or(TelegramClientTransportError::RuntimeUnavailable)?;
        client_transport::serve_connection_durable(stream, runtime, durable, handle)
    }

    pub fn poll_once(&mut self, timeout: Duration) -> Result<TelegramProcessTick, TdlibError> {
        if self.composition.has_pending_authorization() {
            let event = self.composition.poll_authorization(timeout)?;
            if let Some(event) = &event {
                self.authorization_status = Some(authorization_status(event));
            }
            return Ok(event.map(TelegramProcessTick::Authorization).unwrap_or(TelegramProcessTick::Idle));
        }
        if self.composition.has_runtime() {
            let frames = self
                .composition
                .poll_runtime_events(self.provider_cursor.clone())?;
            if let Some(cursor) = frames.last().and_then(|frame| frame.provider_cursor.clone()) {
                self.provider_cursor = Some(cursor);
            }
            return Ok(TelegramProcessTick::Runtime {
                frames: frames.len(),
                provider_cursor: self.provider_cursor.clone(),
            });
        }
        Ok(TelegramProcessTick::Idle)
    }

    pub async fn poll_once_durable(
        &mut self,
        timeout: Duration,
        durable: &TelegramDurablePersistence,
    ) -> Result<TelegramProcessTick, TelegramDurableProcessError> {
        if self.composition.has_pending_authorization() {
            let event = self
                .composition
                .poll_authorization(timeout)
                .map_err(TelegramDurableProcessError::Provider)?;
            if let Some(event) = &event {
                self.authorization_status = Some(authorization_status(event));
            }
            return Ok(event.map(TelegramProcessTick::Authorization).unwrap_or(TelegramProcessTick::Idle));
        }
        if self.composition.has_runtime() {
            let frames = self
                .composition
                .poll_runtime_events(self.provider_cursor.clone())
                .map_err(TelegramDurableProcessError::Provider)?;
            for frame in &frames {
                durable
                    .append_provider_event(frame)
                    .await
                    .map_err(TelegramDurableProcessError::Persistence)?;
                if let Some(runtime) = self.composition.runtime_mut() {
                    runtime
                        .persist_provider_frame_durable(durable, frame)
                        .await
                        .map_err(TelegramDurableProcessError::Projection)?;
                }
            }
            if let Some(cursor) = frames.last().and_then(|frame| frame.provider_cursor.clone()) {
                self.provider_cursor = Some(cursor);
            }
            return Ok(TelegramProcessTick::Runtime {
                frames: frames.len(),
                provider_cursor: self.provider_cursor.clone(),
            });
        }
        Ok(TelegramProcessTick::Idle)
    }

    pub fn run_until<F, H>(
        &mut self,
        timeout: Duration,
        mut should_stop: F,
        mut on_tick: H,
    ) -> Result<(), TdlibError>
    where
        F: FnMut() -> bool,
        H: FnMut(TelegramProcessTick),
    {
        while !should_stop() {
            on_tick(self.poll_once(timeout)?);
        }
        Ok(())
    }
}

/// Runs the already admitted Telegram integration process.
///
/// Admission, credential leases, storage binding, and provider construction
/// happen before this function. This loop owns only the provider runtime,
/// durable client port, and provider event polling.
pub fn serve_admitted_runtime(
    admitted: TelegramAdmittedRuntime,
    socket_path: &Path,
) -> Result<(), String> {
    if !socket_path.is_absolute() {
        return Err("Telegram runtime socket path must be absolute".to_owned());
    }
    if socket_path.exists() {
        std::fs::remove_file(socket_path)
            .map_err(|error| format!("failed to replace Telegram runtime socket: {error}"))?;
    }
    let listener = UnixListener::bind(socket_path)
        .map_err(|error| format!("failed to bind Telegram runtime socket: {error}"))?;
    listener
        .set_nonblocking(true)
        .map_err(|error| format!("failed to configure Telegram runtime socket: {error}"))?;

    let TelegramAdmittedRuntime {
        composition,
        durable,
        account_id,
        ..
    } = admitted;
    let mut process = TelegramProcessLoop::new(composition);
    if let Some(blob_socket) = std::env::var_os("HERMES_BLOB_DATA_SOCKET") {
        process
            .composition_mut()
            .runtime_mut()
            .ok_or_else(|| "Telegram runtime provider is not authorized".to_owned())?
            .configure_blob_materializer(blob_socket, std::env::temp_dir())
            .map_err(|_| "Telegram Blob materializer admission failed".to_owned())?;
    }
    let executor = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .map_err(|error| format!("failed to build Telegram runtime executor: {error}"))?;
    let mut restored = false;

        loop {
            match listener.accept() {
            Ok((stream, _)) => {
                if process.composition().has_runtime() {
                    process
                        .serve_client_connection_durable(stream, &durable, executor.handle())
                        .map_err(|error| format!("Telegram runtime client failed: {error:?}"))?;
                } else {
                    let status = process.authorization_status().cloned();
                    client_transport::serve_authorization_connection(
                        stream,
                        process.composition_mut(),
                        status.as_ref(),
                    )
                    .map_err(|error| format!("Telegram authorization client failed: {error:?}"))?;
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(error) => return Err(format!("Telegram runtime accept failed: {error}")),
        }

        executor
            .block_on(process.poll_once_durable(Duration::from_millis(25), &durable))
            .map_err(|error| format!("Telegram runtime provider loop failed: {error:?}"))?;
        if !restored && process.composition().has_runtime() {
            let runtime = process
                .composition_mut()
                .runtime_mut()
                .ok_or_else(|| "Telegram runtime provider disappeared during restore".to_owned())?;
            executor
                .block_on(runtime.restore_account_state_durable(&durable, &account_id, 10_000))
                .map_err(|error| format!("Telegram durable state restore failed: {error:?}"))?;
            restored = true;
        }
    }
}

fn authorization_status(
    event: &TdlibAuthorizationEvent,
) -> hermes_telegram_api::TelegramAuthorizationStatus {
    match event {
        TdlibAuthorizationEvent::QrLink(link) => hermes_telegram_api::TelegramAuthorizationStatus {
            state: "waiting_qr_scan".to_owned(),
            qr_link: Some(link.clone()),
            password_hint: None,
        },
        TdlibAuthorizationEvent::State(state) => {
            let (state_name, password_hint) = match state {
                TdlibAuthorizationUpdate::WaitingParameters => ("waiting_parameters", None),
                TdlibAuthorizationUpdate::WaitingEncryptionKey => ("waiting_encryption_key", None),
                TdlibAuthorizationUpdate::WaitingQrScan => ("waiting_qr_scan", None),
                TdlibAuthorizationUpdate::WaitingPassword { hint } => ("waiting_password", hint.clone()),
                TdlibAuthorizationUpdate::Ready => ("ready", None),
                TdlibAuthorizationUpdate::Closing => ("closing", None),
                TdlibAuthorizationUpdate::Closed => ("closed", None),
                TdlibAuthorizationUpdate::Error { .. } => ("error", None),
                TdlibAuthorizationUpdate::Other(_) => ("other", None),
            };
            hermes_telegram_api::TelegramAuthorizationStatus {
                state: state_name.to_owned(),
                qr_link: None,
                password_hint,
            }
        }
    }
}
