//! Bounded protobuf framing for the private test harness sockets.

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;

use prost::Message;

const MAX_FRAME_BYTES: usize = 64 * 1024;

pub(super) fn call<Req, Res>(path: &Path, request: &Req) -> Result<Res, String>
where
    Req: Message,
    Res: Message + Default,
{
    let mut stream = UnixStream::connect(path).map_err(|error| error.to_string())?;
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .and_then(|_| stream.set_write_timeout(Some(std::time::Duration::from_secs(5))))
        .map_err(|error| error.to_string())?;
    write_frame(&mut stream, &request.encode_to_vec())?;
    Res::decode(read_frame(&mut stream)?.as_slice()).map_err(|_| "invalid response".to_owned())
}

fn write_frame(stream: &mut UnixStream, bytes: &[u8]) -> Result<(), String> {
    let mut length = u32::try_from(bytes.len()).map_err(|_| "frame is too large".to_owned())?;
    while length >= 0x80 {
        stream
            .write_all(&[(length as u8 & 0x7f) | 0x80])
            .map_err(|error| error.to_string())?;
        length >>= 7;
    }
    stream
        .write_all(&[length as u8])
        .and_then(|_| stream.write_all(bytes))
        .and_then(|_| stream.flush())
        .map_err(|error| error.to_string())
}

fn read_frame(stream: &mut UnixStream) -> Result<Vec<u8>, String> {
    let mut length = 0_usize;
    for shift in (0..35).step_by(7) {
        let mut byte = [0_u8; 1];
        stream
            .read_exact(&mut byte)
            .map_err(|error| error.to_string())?;
        length |= usize::from(byte[0] & 0x7f) << shift;
        if byte[0] & 0x80 == 0 {
            if length > MAX_FRAME_BYTES {
                return Err("frame is too large".to_owned());
            }
            let mut bytes = vec![0; length];
            stream
                .read_exact(&mut bytes)
                .map_err(|error| error.to_string())?;
            return Ok(bytes);
        }
    }
    Err("frame prefix is invalid".to_owned())
}
