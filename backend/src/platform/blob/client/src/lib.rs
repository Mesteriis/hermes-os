//! Client for the owner-private Blob data socket.
//!
//! This package transports already-authorized session grants. It does not
//! issue grants, access the Vault, expose filesystem paths, or interpret
//! provider/domain payloads.

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

use hermes_blob_client_contract::{BlobReadError, BlobReadPort};
use hermes_runtime_protocol::v1::{
    BlobDataReadRangeRequestV1, BlobDataRequestV1, BlobDataResponseV1, BlobDataSessionGrantV1,
    BlobDataWriteRequestV1, blob_data_request_v1::Operation,
};
use prost::Message;

pub const PACKAGE: &str = "hermes-blob-client";
const MAX_FRAME_BYTES: usize = 64 * 1024 * 1024 + 32 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BlobDataClient {
    socket_path: PathBuf,
    timeout: Duration,
}

impl BlobDataClient {
    pub fn new(socket_path: impl Into<PathBuf>) -> Result<Self, BlobClientError> {
        let socket_path = socket_path.into();
        if !socket_path.is_absolute() || socket_path.as_os_str().is_empty() {
            return Err(BlobClientError::InvalidSocketPath);
        }
        Ok(Self {
            socket_path,
            timeout: Duration::from_secs(2),
        })
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Result<Self, BlobClientError> {
        if timeout.is_zero() {
            return Err(BlobClientError::InvalidTimeout);
        }
        self.timeout = timeout;
        Ok(self)
    }

    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }

    pub fn write(
        &self,
        grant: BlobDataSessionGrantV1,
        channel_binding: Vec<u8>,
        plaintext: Vec<u8>,
    ) -> Result<(), BlobClientError> {
        let response = self.request(BlobDataRequestV1 {
            grant: Some(grant),
            channel_binding,
            operation: Some(Operation::Write(BlobDataWriteRequestV1 { plaintext })),
        })?;
        if response.accepted {
            return Ok(());
        }
        Err(BlobClientError::Rejected(response.error_code))
    }

    pub fn read_range(
        &self,
        grant: BlobDataSessionGrantV1,
        channel_binding: Vec<u8>,
        start: u64,
        end_exclusive: u64,
    ) -> Result<Vec<u8>, BlobClientError> {
        let response = self.request(BlobDataRequestV1 {
            grant: Some(grant),
            channel_binding,
            operation: Some(Operation::ReadRange(BlobDataReadRangeRequestV1 {
                start,
                end_exclusive,
            })),
        })?;
        if response.accepted {
            return Ok(response.plaintext);
        }
        Err(BlobClientError::Rejected(response.error_code))
    }

    fn request(&self, request: BlobDataRequestV1) -> Result<BlobDataResponseV1, BlobClientError> {
        let bytes = request.encode_to_vec();
        if bytes.is_empty() || bytes.len() > MAX_FRAME_BYTES {
            return Err(BlobClientError::FrameTooLarge);
        }
        let mut stream = UnixStream::connect(&self.socket_path)
            .map_err(|error| BlobClientError::Connect(error.to_string()))?;
        stream
            .set_read_timeout(Some(self.timeout))
            .and_then(|_| stream.set_write_timeout(Some(self.timeout)))
            .map_err(|error| BlobClientError::Io(error.to_string()))?;
        write_frame(&mut stream, &bytes)?;
        let response = read_frame(&mut stream)?;
        BlobDataResponseV1::decode(response.as_slice())
            .map_err(|_| BlobClientError::InvalidResponse)
    }
}

impl BlobReadPort for BlobDataClient {
    fn read_range(
        &mut self,
        grant: BlobDataSessionGrantV1,
        channel_binding: Vec<u8>,
        start: u64,
        end_exclusive: u64,
    ) -> Result<Vec<u8>, BlobReadError> {
        BlobDataClient::read_range(self, grant, channel_binding, start, end_exclusive).map_err(
            |error| match error {
                BlobClientError::Rejected(_) => BlobReadError::Rejected,
                BlobClientError::InvalidResponse => BlobReadError::InvalidResponse,
                _ => BlobReadError::Unavailable,
            },
        )
    }
}

fn write_frame(stream: &mut UnixStream, bytes: &[u8]) -> Result<(), BlobClientError> {
    let mut length = u32::try_from(bytes.len()).map_err(|_| BlobClientError::FrameTooLarge)?;
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
        .map_err(|error| BlobClientError::Io(error.to_string()))
}

fn read_frame(stream: &mut UnixStream) -> Result<Vec<u8>, BlobClientError> {
    let mut length = 0_u64;
    for shift in (0..35).step_by(7) {
        let mut byte = [0_u8; 1];
        stream
            .read_exact(&mut byte)
            .map_err(|error| BlobClientError::Io(error.to_string()))?;
        length |= u64::from(byte[0] & 0x7f) << shift;
        if byte[0] & 0x80 == 0 {
            let length = usize::try_from(length).map_err(|_| BlobClientError::FrameTooLarge)?;
            if length == 0 || length > MAX_FRAME_BYTES {
                return Err(BlobClientError::FrameTooLarge);
            }
            let mut bytes = vec![0; length];
            stream
                .read_exact(&mut bytes)
                .map_err(|error| BlobClientError::Io(error.to_string()))?;
            return Ok(bytes);
        }
    }
    Err(BlobClientError::InvalidFrame)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BlobClientError {
    InvalidSocketPath,
    InvalidTimeout,
    Connect(String),
    Io(String),
    FrameTooLarge,
    InvalidFrame,
    InvalidResponse,
    Rejected(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn socket_path_must_be_absolute() {
        assert_eq!(
            BlobDataClient::new("relative.sock"),
            Err(BlobClientError::InvalidSocketPath)
        );
    }

    #[test]
    fn timeout_must_be_positive() {
        assert_eq!(
            BlobDataClient::new("/tmp/blob.sock")
                .expect("valid path")
                .with_timeout(Duration::ZERO),
            Err(BlobClientError::InvalidTimeout)
        );
    }
}
