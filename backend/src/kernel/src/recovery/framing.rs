use std::io::{Read, Write};

const MAX_RECOVERY_FRAME_BYTES: usize = 64 * 1024;

pub(super) fn read_recovery_frame(stream: &mut impl Read) -> Result<Vec<u8>, String> {
    let frame_length = read_protobuf_varint(stream)?;
    let frame_length =
        usize::try_from(frame_length).map_err(|_| "recovery IPC frame is too large".to_owned())?;
    if frame_length > MAX_RECOVERY_FRAME_BYTES {
        return Err("recovery IPC frame is too large".to_owned());
    }

    let mut bytes = vec![0_u8; frame_length];
    stream
        .read_exact(&mut bytes)
        .map_err(|error| error.to_string())?;
    Ok(bytes)
}

pub(super) fn write_recovery_frame(stream: &mut impl Write, bytes: &[u8]) -> Result<(), String> {
    let length =
        u32::try_from(bytes.len()).map_err(|_| "recovery IPC response is too large".to_owned())?;
    write_protobuf_varint(stream, length)?;
    stream
        .write_all(bytes)
        .and_then(|_| stream.flush())
        .map_err(|error| error.to_string())
}

fn read_protobuf_varint(stream: &mut impl Read) -> Result<u64, String> {
    let mut value = 0_u64;
    for shift in (0..35).step_by(7) {
        let mut byte = [0_u8; 1];
        stream
            .read_exact(&mut byte)
            .map_err(|error| error.to_string())?;
        value |= u64::from(byte[0] & 0x7f) << shift;
        if byte[0] & 0x80 == 0 {
            return Ok(value);
        }
    }
    Err("invalid recovery IPC frame length".to_owned())
}

fn write_protobuf_varint(stream: &mut impl Write, mut value: u32) -> Result<(), String> {
    while value >= 0x80 {
        stream
            .write_all(&[((value as u8 & 0x7f) | 0x80)])
            .map_err(|error| error.to_string())?;
        value >>= 7;
    }
    stream
        .write_all(&[value as u8])
        .map_err(|error| error.to_string())
}
