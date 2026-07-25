//! Long-lived Telegram process orchestration around the provider runtime.

use std::os::unix::net::UnixStream;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use hermes_blob_client::{
    BlobDataClient, ManagedBlobCustodyTargetV1, ManagedBlobSessionRequestV1,
    request_managed_blob_session_v2,
};
use hermes_communications_ingress::{
    BodyAdmissionFailureV1, BodyBlobReceiptV1, COMMUNICATIONS_BLOB_CUSTODY_TARGET_CAPABILITY_ID,
    COMMUNICATIONS_BLOB_CUSTODY_TARGET_MODULE_ID, COMMUNICATIONS_BLOB_CUSTODY_TARGET_OWNER_ID,
};
use hermes_runtime_protocol::v1::BlobDataOperationV1;
use hermes_runtime_protocol::{
    managed_control::{
        ManagedControlChannelV2, ManagedControlRequestDispatcherV2, ManagedControlTransportErrorV2,
    },
    v1::{
        ManagedRuntimeClientDeliveryResponseV1, ManagedRuntimeControlRequestV1,
        ManagedRuntimeControlResponseV1, ModuleClientResponseV1,
        managed_runtime_control_request_v1::Operation,
        managed_runtime_control_response_v1::Result as ControlResult,
    },
    validation::managed_control::MANAGED_CONTROL_CORRELATION_ID_BYTES,
    validation::module_client::{
        validate_module_client_request_v1, validate_module_client_response_v1,
    },
};
use hermes_telegram_persistence::{TelegramDurablePersistence, TelegramDurablePersistenceError};
use hermes_telegram_tdlib::TdlibAuthorizationUpdate;
use hermes_telegram_tdlib::{TdlibAuthorizationEvent, TdlibError};
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
pub fn serve_admitted_provider_loop(
    admitted: TelegramAdmittedRuntime,
    executor: &tokio::runtime::Runtime,
) -> Result<(), String> {
    let admitted = admitted.into_provider_loop();
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
        handle_client_delivery(&mut control_channel, &mut process, &durable, executor)?;
        let poll = {
            let mut body_admitter =
                |plaintext: &[u8]| admit_telegram_plaintext(&mut control_channel, plaintext);
            executor.block_on(process.poll_once_durable(
                Duration::from_millis(25),
                &durable,
                &mut body_admitter,
            ))
        };
        poll.map_err(|error| format!("Telegram runtime provider loop failed: {error:?}"))?;
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
                        let mut dispatcher = TelegramBusyControlDispatcher;
                        request_managed_blob_session_v2(
                            &mut control_channel,
                            &mut dispatcher,
                            ManagedBlobSessionRequestV1 {
                                capability_id: "blob.content",
                                operation: BlobDataOperationV1::BlobDataOperationReadRangeV1,
                                reference_id: &intent.reference_id,
                                declared_size: intent.declared_size,
                                backup_class: intent.backup_class,
                                receipt_sha256: None,
                                custody_target: None,
                            },
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
        match executor.block_on(
            crate::communications_outbox::relay_communications_outbox_once(
                &durable,
                &event_connection,
                &event_publish_permit,
                published_at_unix_seconds,
            ),
        ) {
            Ok(_) => {}
            Err(
                crate::communications_outbox::TelegramCommunicationsOutboxRelayError::Unavailable,
            ) => {
                std::thread::sleep(Duration::from_millis(25));
            }
            Err(
                crate::communications_outbox::TelegramCommunicationsOutboxRelayError::Persistence,
            ) => {
                return Err("Telegram runtime outbox persistence failed".to_owned());
            }
        }
    }
}

fn admit_telegram_plaintext(
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    plaintext: &[u8],
) -> Result<BodyBlobReceiptV1, BodyAdmissionFailureV1> {
    if plaintext.is_empty() || plaintext.len() > hermes_telegram_api::MAX_TEXT_BYTES {
        return Err(BodyAdmissionFailureV1::SizeLimitExceeded);
    }
    let mut reference_id = [0_u8; 16];
    getrandom::fill(&mut reference_id).map_err(|_| BodyAdmissionFailureV1::SourceUnavailable)?;
    if reference_id.iter().all(|byte| *byte == 0) {
        return Err(BodyAdmissionFailureV1::SourceUnavailable);
    }
    let sha256: [u8; 32] = Sha256::digest(plaintext).into();
    control_channel
        .inner_mut()
        .set_nonblocking(false)
        .map_err(|_| BodyAdmissionFailureV1::SourceUnavailable)?;
    let mut dispatcher = TelegramBusyControlDispatcher;
    let session = request_managed_blob_session_v2(
        control_channel,
        &mut dispatcher,
        ManagedBlobSessionRequestV1 {
            capability_id: "blob.content",
            operation: BlobDataOperationV1::BlobDataOperationWriteV1,
            reference_id: &reference_id,
            declared_size: u64::try_from(plaintext.len())
                .map_err(|_| BodyAdmissionFailureV1::SizeLimitExceeded)?,
            backup_class: 1,
            receipt_sha256: Some(&sha256),
            custody_target: Some(ManagedBlobCustodyTargetV1 {
                owner_id: COMMUNICATIONS_BLOB_CUSTODY_TARGET_OWNER_ID,
                module_id: COMMUNICATIONS_BLOB_CUSTODY_TARGET_MODULE_ID,
                capability_id: COMMUNICATIONS_BLOB_CUSTODY_TARGET_CAPABILITY_ID,
            }),
        },
    );
    let restored = control_channel.inner_mut().set_nonblocking(true);
    let session = session.map_err(|_| BodyAdmissionFailureV1::PolicyRejected)?;
    restored.map_err(|_| BodyAdmissionFailureV1::SourceUnavailable)?;
    let custody_transfer_source_proof = session.custody_transfer_source_proof;
    BlobDataClient::new(session.data_socket_path)
        .and_then(|client| client.write(session.grant, session.channel_binding, plaintext.to_vec()))
        .map_err(|_| BodyAdmissionFailureV1::SourceUnavailable)?;
    Ok(BodyBlobReceiptV1 {
        blob_ref: format!("blob-content:{}", hex_reference_id(&reference_id)),
        reference_id,
        declared_bytes: u64::try_from(plaintext.len())
            .map_err(|_| BodyAdmissionFailureV1::SizeLimitExceeded)?,
        sha256,
        custody_transfer_source_proof,
    })
}

fn hex_reference_id(reference_id: &[u8; 16]) -> String {
    reference_id
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn handle_client_delivery(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    process: &mut TelegramProcessLoop,
    durable: &TelegramDurablePersistence,
    executor: &tokio::runtime::Runtime,
) -> Result<(), String> {
    let Some((correlation_id, control_request)) = channel
        .try_receive_request()
        .map_err(|_| "Telegram runtime control channel is unavailable".to_owned())?
    else {
        return Ok(());
    };
    let request = match control_request.operation {
        Some(Operation::ClientDelivery(delivery)) => match delivery.request {
            Some(request) => request,
            None => {
                write_control_error(
                    channel,
                    correlation_id,
                    "managed_runtime_control_invalid_client_delivery",
                )?;
                return Ok(());
            }
        },
        _ => {
            write_control_error(
                channel,
                correlation_id,
                "managed_runtime_control_unexpected_request",
            )?;
            return Ok(());
        }
    };
    if validate_module_client_request_v1(&request).is_err() {
        write_client_delivery_response(
            channel,
            correlation_id,
            ModuleClientResponseV1 {
                protocol_major: 1,
                request_id: request.request_id,
                response_payload: Vec::new(),
                error_code: "REJECTED".to_owned(),
            },
        )?;
        return Ok(());
    }
    let response = if let Some(runtime) = process.composition_mut().runtime_mut() {
        authorize_media_for_request(channel, runtime, &request)?;
        let payload = executor
            .block_on(client_transport::handle_durable_request(
                runtime,
                durable,
                &request.encode_to_vec(),
            ))
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
    write_client_delivery_response(channel, correlation_id, response)
}

fn authorize_media_for_request<T: hermes_telegram_tdlib::TdlibTransport>(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    runtime: &mut crate::TelegramRuntime<T>,
    request: &hermes_runtime_protocol::v1::ModuleClientRequestV1,
) -> Result<(), String> {
    let Ok(command) = hermes_telegram_api::client_wire::decode_command(&request.request_payload)
    else {
        return Ok(());
    };
    let hermes_telegram_api::TelegramProviderCommand::SendMedia(media) = command else {
        return Ok(());
    };
    let mut dispatcher = TelegramBusyControlDispatcher;
    let session = request_managed_blob_session_v2(
        channel,
        &mut dispatcher,
        ManagedBlobSessionRequestV1 {
            capability_id: "blob.content",
            operation: BlobDataOperationV1::BlobDataOperationReadRangeV1,
            reference_id: &media.blob.reference_id,
            declared_size: media.blob.declared_size,
            backup_class: media.blob.backup_class,
            receipt_sha256: None,
            custody_target: None,
        },
    )
    .map_err(|_| "Telegram Blob session request was denied".to_owned())?;
    runtime
        .authorize_media_session(session, &media.blob)
        .map_err(|_| "Telegram Blob session was rejected".to_owned())
}

struct TelegramBusyControlDispatcher;

impl ManagedControlRequestDispatcherV2<UnixStream> for TelegramBusyControlDispatcher {
    fn dispatch_request(
        &mut self,
        channel: &mut ManagedControlChannelV2<UnixStream>,
        correlation_id: [u8; MANAGED_CONTROL_CORRELATION_ID_BYTES],
        request: ManagedRuntimeControlRequestV1,
    ) -> Result<(), ManagedControlTransportErrorV2> {
        let response = match request.operation {
            Some(Operation::ClientDelivery(delivery)) => match delivery.request {
                Some(request) if validate_module_client_request_v1(&request).is_ok() => {
                    ManagedRuntimeControlResponseV1 {
                        result: Some(ControlResult::ClientDelivery(
                            ManagedRuntimeClientDeliveryResponseV1 {
                                response: Some(ModuleClientResponseV1 {
                                    protocol_major: 1,
                                    request_id: request.request_id,
                                    response_payload: Vec::new(),
                                    error_code: "RUNTIME_BUSY".to_owned(),
                                }),
                            },
                        )),
                        error_code: String::new(),
                    }
                }
                _ => ManagedRuntimeControlResponseV1 {
                    result: None,
                    error_code: "managed_runtime_control_invalid_client_delivery".to_owned(),
                },
            },
            _ => ManagedRuntimeControlResponseV1 {
                result: None,
                error_code: "managed_runtime_control_unexpected_request".to_owned(),
            },
        };
        channel.write_response(correlation_id, response)
    }
}

fn write_client_delivery_response(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    correlation_id: [u8; MANAGED_CONTROL_CORRELATION_ID_BYTES],
    response: ModuleClientResponseV1,
) -> Result<(), String> {
    channel
        .write_response(
            correlation_id,
            ManagedRuntimeControlResponseV1 {
                result: Some(ControlResult::ClientDelivery(
                    ManagedRuntimeClientDeliveryResponseV1 {
                        response: Some(response),
                    },
                )),
                error_code: String::new(),
            },
        )
        .map_err(|_| "Telegram runtime control response failed".to_owned())
}

fn write_control_error(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    correlation_id: [u8; MANAGED_CONTROL_CORRELATION_ID_BYTES],
    error_code: &str,
) -> Result<(), String> {
    channel
        .write_response(
            correlation_id,
            ManagedRuntimeControlResponseV1 {
                result: None,
                error_code: error_code.to_owned(),
            },
        )
        .map_err(|_| "Telegram runtime control response failed".to_owned())
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

#[cfg(test)]
mod control_dispatch_tests {
    use std::os::unix::net::UnixStream;
    use std::thread;

    use hermes_runtime_protocol::managed_control::ManagedControlChannelV2;
    use hermes_runtime_protocol::v1::{
        ContractReferenceV1, ManagedRuntimeClientDeliveryRequestV1, ManagedRuntimeControlAckV1,
        ManagedRuntimeControlRequestV1, ManagedRuntimeControlResponseV1,
        ManagedRuntimeReadyRequestV1, ModuleClientRequestV1,
        managed_runtime_control_frame_v2::Frame, managed_runtime_control_request_v1::Operation,
        managed_runtime_control_response_v1::Result as ControlResult,
    };
    use hermes_runtime_protocol::validation::managed_control::MANAGED_CONTROL_CORRELATION_ID_BYTES;

    use super::TelegramBusyControlDispatcher;

    #[test]
    fn nested_client_delivery_gets_a_correlated_busy_response_without_stealing_platform_reply() {
        let (runtime, kernel) = UnixStream::pair().expect("control pair");
        let kernel = thread::spawn(move || {
            let mut channel = ManagedControlChannelV2::new(kernel);
            let (platform_id, _) = channel.receive_request().expect("platform request");
            channel
                .write_request(
                    [7; MANAGED_CONTROL_CORRELATION_ID_BYTES],
                    ManagedRuntimeControlRequestV1 {
                        operation: Some(Operation::ClientDelivery(
                            ManagedRuntimeClientDeliveryRequestV1 {
                                request: Some(ModuleClientRequestV1 {
                                    protocol_major: 1,
                                    module_id: "telegram".to_owned(),
                                    owner_id: "telegram".to_owned(),
                                    contract: Some(ContractReferenceV1 {
                                        owner: "telegram".to_owned(),
                                        name: "query".to_owned(),
                                        major: 1,
                                        revision: 1,
                                        schema_sha256: vec![1; 32],
                                    }),
                                    request_id: 41,
                                    request_payload: vec![1],
                                }),
                            },
                        )),
                    },
                )
                .expect("client delivery");
            let nested = channel.read_frame().expect("busy response");
            assert_eq!(
                nested.correlation_id,
                vec![7; MANAGED_CONTROL_CORRELATION_ID_BYTES]
            );
            let Some(Frame::Response(response)) = nested.frame else {
                panic!("nested response");
            };
            let Some(ControlResult::ClientDelivery(delivery)) = response.result else {
                panic!("client delivery response");
            };
            assert_eq!(
                delivery.response.expect("module response").error_code,
                "RUNTIME_BUSY"
            );
            channel
                .write_response(
                    platform_id,
                    ManagedRuntimeControlResponseV1 {
                        result: Some(ControlResult::Ack(ManagedRuntimeControlAckV1 {})),
                        error_code: String::new(),
                    },
                )
                .expect("platform response");
        });

        let mut channel = ManagedControlChannelV2::new(runtime);
        let mut dispatcher = TelegramBusyControlDispatcher;
        let response = channel
            .request_next_with_dispatch(
                ManagedRuntimeControlRequestV1 {
                    operation: Some(Operation::Ready(ManagedRuntimeReadyRequestV1::default())),
                },
                &mut dispatcher,
            )
            .expect("correlated platform response");
        assert!(matches!(response.result, Some(ControlResult::Ack(_))));
        kernel.join().expect("kernel join");
    }
}
