//! Bounded framing for one private inherited Blob control channel.

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;

const MAX_FRAME_BYTES: usize = 512 * 1024;

pub(super) fn read_frame(stream: &mut UnixStream) -> Result<Vec<u8>, String> {
    let length = usize::try_from(read_varint(stream)?)
        .map_err(|_| "Blob inherited control frame is invalid".to_owned())?;
    if length == 0 || length > MAX_FRAME_BYTES {
        return Err("Blob inherited control frame is invalid".to_owned());
    }
    let mut bytes = vec![0; length];
    stream
        .read_exact(&mut bytes)
        .map_err(|_| "Blob inherited control channel is unavailable".to_owned())?;
    Ok(bytes)
}

pub(super) fn write_frame(stream: &mut UnixStream, bytes: &[u8]) -> Result<(), String> {
    if bytes.is_empty() || bytes.len() > MAX_FRAME_BYTES {
        return Err("Blob inherited control frame is invalid".to_owned());
    }
    let mut length = u32::try_from(bytes.len())
        .map_err(|_| "Blob inherited control frame is invalid".to_owned())?;
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
        .map_err(|_| "Blob inherited control channel is unavailable".to_owned())
}

fn read_varint(stream: &mut impl Read) -> Result<u64, String> {
    let mut value = 0_u64;
    for shift in (0..35).step_by(7) {
        let mut byte = [0_u8; 1];
        stream
            .read_exact(&mut byte)
            .map_err(|_| "Blob inherited control channel is unavailable".to_owned())?;
        value |= u64::from(byte[0] & 0x7f) << shift;
        if byte[0] & 0x80 == 0 {
            return Ok(value);
        }
    }
    Err("Blob inherited control frame is invalid".to_owned())
}
