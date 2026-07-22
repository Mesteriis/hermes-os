//! Framed local transport for the Telegram-owned module client port.

use std::io::{ErrorKind, Read, Write};
use std::os::unix::net::UnixStream;

use hermes_telegram_persistence::TelegramDurablePersistence;
use hermes_telegram_tdlib::TdlibTransport;

use crate::{
    TelegramRuntime, TelegramRuntimeComposition,
    client_port::{TelegramClientPort, TelegramClientPortError},
};

const MAX_CLIENT_FRAME_BYTES: usize = 512 * 1024;

pub fn serve_authorization_connection(
    mut stream: UnixStream,
    composition: &mut TelegramRuntimeComposition,
    status: Option<&hermes_telegram_api::TelegramAuthorizationStatus>,
) -> Result<(), TelegramClientTransportError> {
    loop {
        let request = match read_frame(&mut stream) {
            Ok(request) => request,
            Err(TelegramClientTransportError::Io(error)) if error == "eof" => return Ok(()),
            Err(error) => return Err(error),
        };
        let (request_id, payload) = crate::client_port::decode_module_request_payload(&request)
            .map_err(TelegramClientTransportError::Port)?;
        let request = hermes_telegram_api::client_wire::decode_request(&payload).map_err(|_| {
            TelegramClientTransportError::Port(TelegramClientPortError::Protocol(
                "Telegram authorization payload is invalid".to_owned(),
            ))
        })?;
        let response = match request {
            hermes_telegram_api::client_wire::TelegramAuthorizationRequest::Status => {
                hermes_telegram_api::client_wire::TelegramAuthorizationResponse::Status(
                    status
                        .cloned()
                        .unwrap_or(hermes_telegram_api::TelegramAuthorizationStatus {
                            state: "starting".to_owned(),
                            qr_link: None,
                            password_hint: None,
                        }),
                )
            }
            hermes_telegram_api::client_wire::TelegramAuthorizationRequest::SubmitPassword(
                password,
            ) => {
                composition.submit_password(&password).map_err(|error| {
                    TelegramClientTransportError::Port(TelegramClientPortError::Provider(error))
                })?;
                hermes_telegram_api::client_wire::TelegramAuthorizationResponse::PasswordAccepted
            }
        };
        let response_payload = hermes_telegram_api::client_wire::encode_response(&response);
        let encoded =
            crate::client_port::encode_module_response_payload(request_id, response_payload)
                .map_err(TelegramClientTransportError::Port)?;
        write_frame(&mut stream, &encoded)?;
    }
}

#[derive(Debug)]
pub enum TelegramClientTransportError {
    Port(TelegramClientPortError),
    Io(String),
    Frame(String),
    RuntimeUnavailable,
}

pub fn serve_connection<T: TdlibTransport>(
    mut stream: UnixStream,
    runtime: &mut TelegramRuntime<T>,
) -> Result<(), TelegramClientTransportError> {
    loop {
        let request = match read_frame(&mut stream) {
            Ok(request) => request,
            Err(TelegramClientTransportError::Io(error)) if error == "eof" => return Ok(()),
            Err(error) => return Err(error),
        };
        let response = TelegramClientPort::new(runtime)
            .handle_module_request(&request)
            .map_err(TelegramClientTransportError::Port)?;
        write_frame(&mut stream, &response)?;
    }
}

pub fn serve_connection_durable<T: TdlibTransport>(
    mut stream: UnixStream,
    runtime: &mut TelegramRuntime<T>,
    durable: &TelegramDurablePersistence,
    handle: &tokio::runtime::Handle,
) -> Result<(), TelegramClientTransportError> {
    loop {
        let request = match read_frame(&mut stream) {
            Ok(request) => request,
            Err(TelegramClientTransportError::Io(error)) if error == "eof" => return Ok(()),
            Err(error) => return Err(error),
        };
        let response = tokio::task::block_in_place(|| {
            handle.block_on(
                crate::client_port::TelegramClientPort::new(runtime)
                    .handle_module_request_durable(&request, durable),
            )
        })
        .map_err(TelegramClientTransportError::Port)?;
        write_frame(&mut stream, &response)?;
    }
}

fn read_frame(stream: &mut UnixStream) -> Result<Vec<u8>, TelegramClientTransportError> {
    let length = read_length(stream)?;
    if length == 0 || length > MAX_CLIENT_FRAME_BYTES {
        return Err(TelegramClientTransportError::Frame(
            "Telegram client frame length is invalid".to_owned(),
        ));
    }
    let mut bytes = vec![0_u8; length];
    stream.read_exact(&mut bytes).map_err(|error| {
        if error.kind() == ErrorKind::UnexpectedEof {
            TelegramClientTransportError::Io("eof".to_owned())
        } else {
            TelegramClientTransportError::Io("Telegram client transport is unavailable".to_owned())
        }
    })?;
    Ok(bytes)
}

fn write_frame(stream: &mut UnixStream, bytes: &[u8]) -> Result<(), TelegramClientTransportError> {
    if bytes.is_empty() || bytes.len() > MAX_CLIENT_FRAME_BYTES {
        return Err(TelegramClientTransportError::Frame(
            "Telegram client frame length is invalid".to_owned(),
        ));
    }
    let mut length = u32::try_from(bytes.len()).map_err(|_| {
        TelegramClientTransportError::Frame("Telegram client frame length is invalid".to_owned())
    })?;
    let mut prefix = Vec::with_capacity(5);
    while length >= 0x80 {
        prefix.push((length as u8 & 0x7f) | 0x80);
        length >>= 7;
    }
    prefix.push(length as u8);
    stream
        .write_all(&prefix)
        .and_then(|_| stream.write_all(bytes))
        .and_then(|_| stream.flush())
        .map_err(|_| {
            TelegramClientTransportError::Io("Telegram client transport is unavailable".to_owned())
        })
}

fn read_length(stream: &mut UnixStream) -> Result<usize, TelegramClientTransportError> {
    let mut value = 0_u64;
    for index in 0..5 {
        let mut byte = [0_u8; 1];
        stream.read_exact(&mut byte).map_err(|error| {
            if error.kind() == ErrorKind::UnexpectedEof {
                TelegramClientTransportError::Io("eof".to_owned())
            } else {
                TelegramClientTransportError::Io(
                    "Telegram client transport is unavailable".to_owned(),
                )
            }
        })?;
        value |= u64::from(byte[0] & 0x7f) << (index * 7);
        if byte[0] & 0x80 == 0 {
            return usize::try_from(value).map_err(|_| {
                TelegramClientTransportError::Frame(
                    "Telegram client frame length is invalid".to_owned(),
                )
            });
        }
    }
    Err(TelegramClientTransportError::Frame(
        "Telegram client frame length is invalid".to_owned(),
    ))
}
