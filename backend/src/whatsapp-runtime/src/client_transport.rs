//! Framed local transport for the WhatsApp-owned client port.

use std::io::{ErrorKind, Read, Write};
use std::os::unix::net::UnixStream;

use crate::{WhatsAppProviderTransport, WhatsAppRuntime, client_port::WhatsAppClientPort};
use hermes_whatsapp_persistence::WhatsAppDurablePersistence;

const MAX_FRAME_BYTES: usize = 512 * 1024;

#[derive(Debug)]
pub enum WhatsAppClientTransportError {
    Port(crate::client_port::WhatsAppClientPortError),
    Io(String),
    Frame(String),
}

pub fn serve_connection<T: WhatsAppProviderTransport>(
    mut stream: UnixStream,
    runtime: &mut WhatsAppRuntime<T>,
) -> Result<(), WhatsAppClientTransportError> {
    loop {
        let request = match read_frame(&mut stream) {
            Ok(bytes) => bytes,
            Err(WhatsAppClientTransportError::Io(error)) if error == "eof" => return Ok(()),
            Err(error) => return Err(error),
        };
        let response = WhatsAppClientPort::new(runtime)
            .handle_module_request(&request)
            .map_err(WhatsAppClientTransportError::Port)?;
        write_frame(&mut stream, &response)?;
    }
}

pub fn serve_connection_durable<T: WhatsAppProviderTransport>(
    mut stream: UnixStream,
    runtime: &mut WhatsAppRuntime<T>,
    durable: &WhatsAppDurablePersistence,
    handle: &tokio::runtime::Handle,
) -> Result<(), WhatsAppClientTransportError> {
    loop {
        let request = match read_frame(&mut stream) {
            Ok(bytes) => bytes,
            Err(WhatsAppClientTransportError::Io(error)) if error == "eof" => return Ok(()),
            Err(error) => return Err(error),
        };
        let response = tokio::task::block_in_place(|| {
            handle.block_on(
                WhatsAppClientPort::new(runtime)
                    .handle_module_request_durable(&request, durable),
            )
        })
            .map_err(WhatsAppClientTransportError::Port)?;
        write_frame(&mut stream, &response)?;
    }
}

fn read_frame(stream: &mut UnixStream) -> Result<Vec<u8>, WhatsAppClientTransportError> {
    let length = read_length(stream)?;
    if length == 0 || length > MAX_FRAME_BYTES {
        return Err(WhatsAppClientTransportError::Frame(
            "WhatsApp client frame length is invalid".to_owned(),
        ));
    }
    let mut bytes = vec![0_u8; length];
    stream.read_exact(&mut bytes).map_err(|error| {
        if error.kind() == ErrorKind::UnexpectedEof {
            WhatsAppClientTransportError::Io("eof".to_owned())
        } else {
            WhatsAppClientTransportError::Io("WhatsApp client transport is unavailable".to_owned())
        }
    })?;
    Ok(bytes)
}

fn write_frame(stream: &mut UnixStream, bytes: &[u8]) -> Result<(), WhatsAppClientTransportError> {
    if bytes.is_empty() || bytes.len() > MAX_FRAME_BYTES {
        return Err(WhatsAppClientTransportError::Frame(
            "WhatsApp client frame length is invalid".to_owned(),
        ));
    }
    let mut length = u32::try_from(bytes.len()).map_err(|_| {
        WhatsAppClientTransportError::Frame("WhatsApp client frame length is invalid".to_owned())
    })?;
    let mut prefix = Vec::with_capacity(5);
    while length >= 0x80 {
        prefix.push((length as u8 & 0x7f) | 0x80);
        length >>= 7;
    }
    prefix.push(length as u8);
    stream.write_all(&prefix).and_then(|_| stream.write_all(bytes)).and_then(|_| stream.flush()).map_err(|_| {
        WhatsAppClientTransportError::Io("WhatsApp client transport is unavailable".to_owned())
    })
}

fn read_length(stream: &mut UnixStream) -> Result<usize, WhatsAppClientTransportError> {
    let mut value = 0_u64;
    for index in 0..5 {
        let mut byte = [0_u8; 1];
        stream.read_exact(&mut byte).map_err(|error| {
            if error.kind() == ErrorKind::UnexpectedEof {
                WhatsAppClientTransportError::Io("eof".to_owned())
            } else {
                WhatsAppClientTransportError::Io("WhatsApp client transport is unavailable".to_owned())
            }
        })?;
        value |= u64::from(byte[0] & 0x7f) << (index * 7);
        if byte[0] & 0x80 == 0 {
            return usize::try_from(value).map_err(|_| {
                WhatsAppClientTransportError::Frame("WhatsApp client frame length is invalid".to_owned())
            });
        }
    }
    Err(WhatsAppClientTransportError::Frame(
        "WhatsApp client frame length is invalid".to_owned(),
    ))
}
