//! Framed Unix transport for the admitted WhatsApp runtime client port.

use std::io::{ErrorKind, Read, Write};
use std::os::unix::net::UnixStream;
use std::time::{SystemTime, UNIX_EPOCH};

use hermes_whatsapp_api::host_bridge::{
    decode_host_bridge_handshake, encode_host_bridge_handshake_accepted,
};

use crate::{client_port, managed::WhatsAppAdmittedRuntime};

const MAX_FRAME_BYTES: usize = 512 * 1024;

#[derive(Debug)]
pub enum WhatsAppClientTransportError {
    Closed,
    Frame,
    Io,
    Port,
    Handshake,
}

pub fn serve_connection(
    mut stream: UnixStream,
    runtime: &WhatsAppAdmittedRuntime,
    handle: &tokio::runtime::Handle,
) -> Result<(), WhatsAppClientTransportError> {
    accept_host_bridge_handshake(&mut stream, runtime)?;
    loop {
        let request = match read_frame(&mut stream) {
            Ok(request) => request,
            Err(WhatsAppClientTransportError::Closed) => return Ok(()),
            Err(error) => return Err(error),
        };
        let recorded_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| WhatsAppClientTransportError::Io)?;
        let recorded_at_unix_seconds = i64::try_from(recorded_at.as_secs())
            .map_err(|_| WhatsAppClientTransportError::Io)?;
        let recorded_at_nanos = i32::try_from(recorded_at.subsec_nanos())
            .map_err(|_| WhatsAppClientTransportError::Io)?;
        let response = handle
            .block_on(client_port::handle_host_request(
                runtime,
                &request,
                recorded_at_unix_seconds,
                recorded_at_nanos,
            ))
            .map_err(|_| WhatsAppClientTransportError::Port)?;
        write_frame(&mut stream, &response)?;
    }
}

fn accept_host_bridge_handshake(
    stream: &mut UnixStream,
    runtime: &WhatsAppAdmittedRuntime,
) -> Result<(), WhatsAppClientTransportError> {
    let handshake = read_frame(stream)?;
    let handshake = decode_host_bridge_handshake(&handshake)
        .map_err(|_| WhatsAppClientTransportError::Handshake)?;
    if !runtime.accepts_host_bridge_handshake(&handshake) {
        return Err(WhatsAppClientTransportError::Handshake);
    }
    write_frame(stream, &encode_host_bridge_handshake_accepted())
}

fn read_frame(stream: &mut UnixStream) -> Result<Vec<u8>, WhatsAppClientTransportError> {
    let length = read_length(stream)?;
    if length == 0 || length > MAX_FRAME_BYTES {
        return Err(WhatsAppClientTransportError::Frame);
    }
    let mut bytes = vec![0_u8; length];
    stream.read_exact(&mut bytes).map_err(|error| {
        if error.kind() == ErrorKind::UnexpectedEof {
            WhatsAppClientTransportError::Closed
        } else {
            WhatsAppClientTransportError::Io
        }
    })?;
    Ok(bytes)
}

fn write_frame(stream: &mut UnixStream, bytes: &[u8]) -> Result<(), WhatsAppClientTransportError> {
    if bytes.is_empty() || bytes.len() > MAX_FRAME_BYTES {
        return Err(WhatsAppClientTransportError::Frame);
    }
    let mut length = u32::try_from(bytes.len()).map_err(|_| WhatsAppClientTransportError::Frame)?;
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
        .map_err(|_| WhatsAppClientTransportError::Io)
}

fn read_length(stream: &mut UnixStream) -> Result<usize, WhatsAppClientTransportError> {
    let mut value = 0_u64;
    for index in 0..5 {
        let mut byte = [0_u8; 1];
        match stream.read_exact(&mut byte) {
            Ok(()) => {}
            Err(error) if error.kind() == ErrorKind::UnexpectedEof => return Err(WhatsAppClientTransportError::Closed),
            Err(_) => return Err(WhatsAppClientTransportError::Io),
        }
        value |= u64::from(byte[0] & 0x7f) << (index * 7);
        if byte[0] & 0x80 == 0 {
            return usize::try_from(value).map_err(|_| WhatsAppClientTransportError::Frame);
        }
    }
    Err(WhatsAppClientTransportError::Frame)
}
