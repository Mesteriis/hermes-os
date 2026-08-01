//! Bounded framing for private Blob content requests.

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;

pub(super) const MAX_DATA_FRAME_BYTES: usize = 64 * 1024 * 1024 + 32 * 1024;

pub(super) fn read_frame(stream: &mut UnixStream) -> Result<Vec<u8>, ()> {
    let length = usize::try_from(read_varint(stream)?).map_err(|_| ())?;
    if length == 0 || length > MAX_DATA_FRAME_BYTES {
        return Err(());
    }
    let mut bytes = vec![0; length];
    stream.read_exact(&mut bytes).map_err(|_| ())?;
    Ok(bytes)
}

pub(super) fn write_frame(stream: &mut UnixStream, bytes: &[u8]) -> Result<(), ()> {
    if bytes.is_empty() || bytes.len() > MAX_DATA_FRAME_BYTES {
        return Err(());
    }
    let mut value = u32::try_from(bytes.len()).map_err(|_| ())?;
    let mut prefix = Vec::with_capacity(5);
    while value >= 0x80 {
        prefix.push((value as u8 & 0x7f) | 0x80);
        value >>= 7;
    }
    prefix.push(value as u8);
    stream
        .write_all(&prefix)
        .and_then(|_| stream.write_all(bytes))
        .and_then(|_| stream.flush())
        .map_err(|_| ())
}

fn read_varint(stream: &mut impl Read) -> Result<u64, ()> {
    let mut value = 0_u64;
    for shift in (0..35).step_by(7) {
        let mut byte = [0_u8; 1];
        stream.read_exact(&mut byte).map_err(|_| ())?;
        value |= u64::from(byte[0] & 0x7f) << shift;
        if byte[0] & 0x80 == 0 {
            return Ok(value);
        }
    }
    Err(())
}
