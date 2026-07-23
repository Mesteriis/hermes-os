//! Long-lived Telegram process orchestration around the provider runtime.

use std::os::fd::AsRawFd;
use std::os::unix::net::UnixStream;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use hermes_telegram_persistence::{TelegramDurablePersistence, TelegramDurablePersistenceError};
use hermes_telegram_tdlib::TdlibAuthorizationUpdate;
use hermes_telegram_tdlib::{TdlibAuthorizationEvent, TdlibError};
use hermes_runtime_protocol::{
    v1::{ManagedRuntimeClientDeliveryRequestV1, ManagedRuntimeClientDeliveryResponseV1, ModuleClientResponseV1},
    validation::module_client::{validate_module_client_request_v1, validate_module_client_response_v1},
};
use hermes_blob_client::request_managed_blob_session;
use hermes_blob_client::BlobDataClient;
use hermes_communications_ingress::{BodyAdmissionFailureV1, BodyBlobReceiptV1};
use hermes_runtime_protocol::v1::BlobDataOperationV1;
use prost::Message;
use sha2::{Digest, Sha256};

use crate::{
    TelegramDurableProjectionError, TelegramRuntimeComposition,
    bootstrap::{TelegramAdmittedProviderLoop, TelegramAdmittedRuntime},
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
    pub fn authorization_status(
        &self,
    ) -> Option<&hermes_telegram_api::TelegramAuthorizationStatus> {
        self.authorization_status.as_ref()
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
            return Ok(event
                .map(|value| TelegramProcessTick::Authorization(Some(value)))
                .unwrap_or(TelegramProcessTick::Idle));
        }
        if self.composition.has_runtime() {
            let frames = self
                .composition
                .poll_runtime_events(self.provider_cursor.clone())?;
            if let Some(cursor) = frames
                .last()
                .and_then(|frame| frame.provider_cursor.clone())
            {
                self.provider_cursor = Some(cursor);
            }
            return Ok(TelegramProcessTick::Runtime {
                frames: frames.len(),
                provider_cursor: self.provider_cursor.clone(),
            });
        }
        Ok(TelegramProcessTick::Idle)
    }

    pub async fn poll_once_durable<F>(
        &mut self,
        timeout: Duration,
        durable: &TelegramDurablePersistence,
        body_admitter: &mut F,
    ) -> Result<TelegramProcessTick, TelegramDurableProcessError>
    where
        F: FnMut(&[u8]) -> Result<BodyBlobReceiptV1, BodyAdmissionFailureV1>,
    {
        if self.composition.has_pending_authorization() {
            let event = self
                .composition
                .poll_authorization(timeout)
                .map_err(TelegramDurableProcessError::Provider)?;
            if let Some(event) = &event {
                self.authorization_status = Some(authorization_status(event));
            }
            return Ok(event
                .map(|value| TelegramProcessTick::Authorization(Some(value)))
                .unwrap_or(TelegramProcessTick::Idle));
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
                        .persist_provider_frame_durable(durable, frame, body_admitter)
                        .await
                        .map_err(TelegramDurableProcessError::Projection)?;
                }
            }
            if let Some(cursor) = frames
                .last()
                .and_then(|frame| frame.provider_cursor.clone())
            {
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

/// Runs the provider side of an admitted runtime without exposing a private
/// provider client socket. Core capability routing owns client request delivery.
pub fn serve_admitted_provider_loop(admitted: TelegramAdmittedRuntime) -> Result<(), String> {
    let admitted = admitted.into_provider_loop();
    let executor = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .map_err(|error| format!("failed to build Telegram runtime executor: {error}"))?;
    let TelegramAdmittedProviderLoop {
        mut control_channel,
        account_id,
        composition,
        durable,
        event_connection,
        event_publish_permit,
    } = admitted;
    let mut process = TelegramProcessLoop::new(composition);
    let mut restored = false;

    loop {
        handle_client_delivery(&mut control_channel, &mut process, &durable, &executor)?;
        let poll = {
            let mut body_admitter = |plaintext: &[u8]| admit_telegram_plaintext(&mut control_channel, plaintext);
            executor
            .block_on(process.poll_once_durable(Duration::from_millis(25), &durable, &mut body_admitter))
        };
        poll
            .map_err(|error| format!("Telegram runtime provider loop failed: {error:?}"))?;
        if !restored && process.composition().has_runtime() {
            let runtime = process
                .composition_mut()
                .runtime_mut()
                .ok_or_else(|| "Telegram runtime provider disappeared during restore".to_owned())?;
            executor
                .block_on(runtime.restore_account_state_durable(
                    &durable,
                    &account_id,
                    10_000,
                ))
                .map_err(|error| format!("Telegram durable state restore failed: {error:?}"))?;
            restored = true;
        }
        if let Some(runtime) = process.composition_mut().runtime_mut() {
            let now_unix_seconds = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|_| "Telegram runtime clock is unavailable".to_owned())?
                .as_secs();
            executor
                .block_on(runtime.execute_due_durable_operations(
                    &durable,
                    &account_id,
                    now_unix_seconds,
                    16,
                    "telegram-provider-runtime",
                    |intent| {
                        request_managed_blob_session(
                            &mut control_channel,
                            "blob.content",
                            BlobDataOperationV1::BlobDataOperationReadRangeV1,
                            &intent.reference_id,
                            intent.declared_size,
                            intent.backup_class,
                        )
                        .map_err(|_| {
                            TdlibError::Protocol(
                                "Telegram Blob session request was denied".to_owned(),
                            )
                        })
                    },
                ))
                .map_err(|error| format!("Telegram durable execution failed: {error:?}"))?;
        }
        let published_at_unix_seconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| "Telegram runtime clock is unavailable".to_owned())
            .and_then(|duration| {
                i64::try_from(duration.as_secs())
                    .map_err(|_| "Telegram runtime clock is unavailable".to_owned())
            })?;
        executor
            .block_on(crate::communications_outbox::relay_communications_outbox_once(
                &durable,
                &event_connection,
                &event_publish_permit,
                published_at_unix_seconds,
            ))
            .map_err(|error| format!("Telegram runtime outbox relay failed: {error:?}"))?;
    }
}

fn admit_telegram_plaintext(
    control_channel: &mut UnixStream,
    plaintext: &[u8],
) -> Result<BodyBlobReceiptV1, BodyAdmissionFailureV1> {
    if plaintext.is_empty() || plaintext.len() > hermes_telegram_api::MAX_TEXT_BYTES {
        return Err(BodyAdmissionFailureV1::SizeLimitExceeded);
    }
    let mut reference_id = [0_u8; 16];
    getrandom::fill(&mut reference_id).map_err(|_| BodyAdmissionFailureV1::SourceUnavailable)?;
    if reference_id.iter().all(|byte| *byte == 0) { return Err(BodyAdmissionFailureV1::SourceUnavailable); }
    control_channel.set_nonblocking(false).map_err(|_| BodyAdmissionFailureV1::SourceUnavailable)?;
    let session = request_managed_blob_session(
        control_channel,
        "blob.content",
        BlobDataOperationV1::BlobDataOperationWriteV1,
        &reference_id,
        u64::try_from(plaintext.len()).map_err(|_| BodyAdmissionFailureV1::SizeLimitExceeded)?,
        1,
    );
    let restored = control_channel.set_nonblocking(true);
    let session = session.map_err(|_| BodyAdmissionFailureV1::PolicyRejected)?;
    restored.map_err(|_| BodyAdmissionFailureV1::SourceUnavailable)?;
    BlobDataClient::new(session.data_socket_path)
        .and_then(|client| client.write(session.grant, session.channel_binding, plaintext.to_vec()))
        .map_err(|_| BodyAdmissionFailureV1::SourceUnavailable)?;
    let sha256: [u8; 32] = Sha256::digest(plaintext).into();
    Ok(BodyBlobReceiptV1 {
        blob_ref: format!("blob-content:{}", hex_reference_id(&reference_id)),
        reference_id,
        declared_bytes: u64::try_from(plaintext.len()).map_err(|_| BodyAdmissionFailureV1::SizeLimitExceeded)?,
        sha256,
    })
}

fn hex_reference_id(reference_id: &[u8; 16]) -> String {
    reference_id.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn handle_client_delivery(
    channel: &mut UnixStream,
    process: &mut TelegramProcessLoop,
    durable: &TelegramDurablePersistence,
    executor: &tokio::runtime::Runtime,
) -> Result<(), String> {
    let Some(frame) = peek_complete_frame(channel)? else { return Ok(()); };
    let request = ManagedRuntimeClientDeliveryRequestV1::decode(frame.as_slice())
        .map_err(|_| "Telegram runtime client delivery is invalid".to_owned())?
        .request
        .ok_or_else(|| "Telegram runtime client delivery is invalid".to_owned())?;
    validate_module_client_request_v1(&request)
        .map_err(|_| "Telegram runtime client delivery is invalid".to_owned())?;
    if read_frame(channel)? != frame { return Err("Telegram runtime client delivery is invalid".to_owned()); }
    let response = if let Some(runtime) = process.composition_mut().runtime_mut() {
        authorize_media_for_request(channel, runtime, &request)?;
        let payload = executor
            .block_on(client_transport::handle_durable_request(runtime, durable, &request.encode_to_vec()))
            .map_err(|_| "Telegram runtime client request failed".to_owned())?;
        ModuleClientResponseV1::decode(payload.as_slice())
            .map_err(|_| "Telegram runtime client response is invalid".to_owned())?
    } else {
        ModuleClientResponseV1 {
            protocol_major: 1,
            request_id: request.request_id,
            response_payload: Vec::new(),
            error_code: "RUNTIME_UNAVAILABLE".to_owned(),
        }
    };
    validate_module_client_response_v1(&response)
        .map_err(|_| "Telegram runtime client response is invalid".to_owned())?;
    write_frame(channel, &ManagedRuntimeClientDeliveryResponseV1 { response: Some(response) }.encode_to_vec())
}

fn authorize_media_for_request<T: hermes_telegram_tdlib::TdlibTransport>(
    channel: &mut UnixStream,
    runtime: &mut crate::TelegramRuntime<T>,
    request: &hermes_runtime_protocol::v1::ModuleClientRequestV1,
) -> Result<(), String> {
    let Ok(command) = hermes_telegram_api::client_wire::decode_command(&request.request_payload) else {
        return Ok(());
    };
    let hermes_telegram_api::TelegramProviderCommand::SendMedia(media) = command else {
        return Ok(());
    };
    let session = request_managed_blob_session(
        channel,
        "blob.content",
        BlobDataOperationV1::BlobDataOperationReadRangeV1,
        &media.blob.reference_id,
        media.blob.declared_size,
        media.blob.backup_class,
    )
    .map_err(|_| "Telegram Blob session request was denied".to_owned())?;
    runtime
        .authorize_media_session(session, &media.blob)
        .map_err(|_| "Telegram Blob session was rejected".to_owned())
}

fn peek_complete_frame(channel: &UnixStream) -> Result<Option<Vec<u8>>, String> {
    let mut header = [0_u8; 5];
    let length = unsafe { libc::recv(channel.as_raw_fd(), header.as_mut_ptr().cast(), header.len(), libc::MSG_PEEK) };
    if length < 0 {
        return if std::io::Error::last_os_error().kind() == std::io::ErrorKind::WouldBlock { Ok(None) } else { Err("Telegram runtime channel is unavailable".to_owned()) };
    }
    if length == 0 { return Err("Telegram runtime channel is unavailable".to_owned()); }
    let (payload_length, prefix_length) = decode_peeked_length(&header[..usize::try_from(length).map_err(|_| "Telegram runtime frame is invalid".to_owned())?])?;
    if payload_length == 0 || payload_length > 512 * 1024 { return Err("Telegram runtime frame is invalid".to_owned()); }
    let mut framed = vec![0_u8; prefix_length + payload_length];
    let received = unsafe { libc::recv(channel.as_raw_fd(), framed.as_mut_ptr().cast(), framed.len(), libc::MSG_PEEK) };
    if received < 0 { return Err("Telegram runtime channel is unavailable".to_owned()); }
    if usize::try_from(received).map_err(|_| "Telegram runtime frame is invalid".to_owned())? < framed.len() { return Ok(None); }
    Ok(Some(framed[prefix_length..].to_vec()))
}

fn read_frame(channel: &mut UnixStream) -> Result<Vec<u8>, String> {
    let (length, _) = read_length(channel)?;
    if length == 0 || length > 512 * 1024 { return Err("Telegram runtime frame is invalid".to_owned()); }
    let mut bytes = vec![0_u8; length];
    use std::io::Read;
    channel.read_exact(&mut bytes).map_err(|_| "Telegram runtime channel is unavailable".to_owned())?;
    Ok(bytes)
}

fn write_frame(channel: &mut UnixStream, bytes: &[u8]) -> Result<(), String> {
    use std::io::Write;
    if bytes.is_empty() || bytes.len() > 512 * 1024 { return Err("Telegram runtime frame is invalid".to_owned()); }
    let mut length = u32::try_from(bytes.len()).map_err(|_| "Telegram runtime frame is invalid".to_owned())?;
    let mut prefix = Vec::new();
    while length >= 0x80 { prefix.push((length as u8 & 0x7f) | 0x80); length >>= 7; }
    prefix.push(length as u8);
    channel.write_all(&prefix).and_then(|_| channel.write_all(bytes)).and_then(|_| channel.flush()).map_err(|_| "Telegram runtime channel is unavailable".to_owned())
}

fn read_length(channel: &mut UnixStream) -> Result<(usize, usize), String> {
    use std::io::Read;
    let mut value = 0_usize;
    for index in 0..5 {
        let mut byte = [0_u8; 1];
        channel.read_exact(&mut byte).map_err(|_| "Telegram runtime channel is unavailable".to_owned())?;
        value |= usize::from(byte[0] & 0x7f) << (index * 7);
        if byte[0] & 0x80 == 0 { return Ok((value, index + 1)); }
    }
    Err("Telegram runtime frame is invalid".to_owned())
}

fn decode_peeked_length(bytes: &[u8]) -> Result<(usize, usize), String> {
    let mut value = 0_usize;
    for (index, byte) in bytes.iter().copied().enumerate() {
        value |= usize::from(byte & 0x7f) << (index * 7);
        if byte & 0x80 == 0 { return Ok((value, index + 1)); }
    }
    Err("Telegram runtime frame is invalid".to_owned())
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
                TdlibAuthorizationUpdate::WaitingPassword { hint } => {
                    ("waiting_password", hint.clone())
                }
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
