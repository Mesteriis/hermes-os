//! Bounded binary framing for the inherited control connection.

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;

const MAX_FRAME_BYTES: usize = 512 * 1024;

pub(super) fn read_frame(stream: &mut UnixStream) -> Result<Vec<u8>, String> {
    let length = read_varint(stream)?;
    let length =
        usize::try_from(length).map_err(|_| "Telemetry inherited control is invalid".to_owned())?;
    if length == 0 || length > MAX_FRAME_BYTES {
        return Err("Telemetry inherited control is invalid".to_owned());
    }
    let mut frame = vec![0; length];
    stream
        .read_exact(&mut frame)
        .map_err(|_| "Telemetry inherited control is unavailable".to_owned())?;
    Ok(frame)
}

pub(super) fn write_frame(stream: &mut UnixStream, bytes: &[u8]) -> Result<(), String> {
    if bytes.is_empty() || bytes.len() > MAX_FRAME_BYTES {
        return Err("Telemetry inherited control is invalid".to_owned());
    }
    let mut value = u32::try_from(bytes.len())
        .map_err(|_| "Telemetry inherited control is invalid".to_owned())?;
    let mut prefix = Vec::new();
    while value >= 0x80 {
        prefix.push((value as u8 & 0x7f) | 0x80);
        value >>= 7;
    }
    prefix.push(value as u8);
    stream
        .write_all(&prefix)
        .and_then(|_| stream.write_all(bytes))
        .and_then(|_| stream.flush())
        .map_err(|_| "Telemetry inherited control is unavailable".to_owned())
}

fn read_varint(stream: &mut impl Read) -> Result<u64, String> {
    let mut value = 0_u64;
    for shift in (0..35).step_by(7) {
        let mut byte = [0_u8; 1];
        stream
            .read_exact(&mut byte)
            .map_err(|_| "Telemetry inherited control is unavailable".to_owned())?;
        value |= u64::from(byte[0] & 0x7f) << shift;
        if byte[0] & 0x80 == 0 {
            return Ok(value);
        }
    }
    Err("Telemetry inherited control is invalid".to_owned())
}
