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
    BlobCustodyTransferGrantV1, BlobDataCustodyTransferRequestV1, BlobDataReadRangeRequestV1, BlobDataRequestV1, BlobDataResponseV1, BlobDataSessionGrantV1,
    BlobDataWriteRequestV1, BlobDataOperationV1,
    ManagedRuntimeBlobSessionRequestV1, ManagedRuntimeControlRequestV1,
    ManagedRuntimeControlResponseV1, blob_data_request_v1::Operation,
    managed_runtime_control_request_v1::Operation as ControlOperation,
    managed_runtime_control_response_v1::Result as ControlResult,
};
use prost::Message;
use sha2::{Digest, Sha256};

pub const PACKAGE: &str = "hermes-blob-client";
const MAX_FRAME_BYTES: usize = 64 * 1024 * 1024 + 32 * 1024;
const CONTROL_FRAME_BYTES: usize = 512 * 1024;

pub struct ManagedBlobSessionV1 {
    pub data_socket_path: PathBuf,
    pub grant: BlobDataSessionGrantV1,
    pub channel_binding: Vec<u8>,
    pub custody_transfer_source_proof: Vec<u8>,
}

/// Kernel-authorized internal rewrap request. It transports no source bytes,
/// key material, provider identity, or target storage path.
pub struct ManagedBlobCustodyTransferV1 {
    pub data_socket_path: PathBuf,
    pub grant: BlobCustodyTransferGrantV1,
    pub channel_binding: Vec<u8>,
}

pub fn request_managed_blob_custody_transfer(
    channel: &mut UnixStream,
    capability_id: &str,
    source_reference_id: &[u8; 16],
    declared_size: u64,
    receipt_sha256: &[u8; 32],
    custody_source_proof: &[u8],
    evidence_id: &[u8; 16],
    evidence_envelope_sha256: &[u8; 32],
) -> Result<ManagedBlobCustodyTransferV1, BlobClientError> {
    if capability_id.is_empty()
        || capability_id.len() > 128
        || source_reference_id.iter().all(|byte| *byte == 0)
        || declared_size == 0
        || receipt_sha256.iter().all(|byte| *byte == 0)
        || custody_source_proof.is_empty()
        || custody_source_proof.len() > 2_048
        || evidence_id.iter().all(|byte| *byte == 0)
        || evidence_envelope_sha256.iter().all(|byte| *byte == 0)
    {
        return Err(BlobClientError::InvalidSessionRequest);
    }
    let mut request_id = [0_u8; 16];
    let mut channel_binding = vec![0_u8; 32];
    getrandom::fill(&mut request_id).map_err(|_| BlobClientError::Unavailable)?;
    getrandom::fill(&mut channel_binding).map_err(|_| BlobClientError::Unavailable)?;
    if request_id.iter().all(|byte| *byte == 0) || channel_binding.iter().all(|byte| *byte == 0) {
        return Err(BlobClientError::Unavailable);
    }
    let request = ManagedRuntimeControlRequestV1 {
        operation: Some(ControlOperation::IssueBlobSession(ManagedRuntimeBlobSessionRequestV1 {
            request_id: request_id.to_vec(),
            capability_id: capability_id.to_owned(),
            operation: BlobDataOperationV1::BlobDataOperationCustodyTransferV1 as u32,
            channel_binding_sha256: Sha256::digest(&channel_binding).to_vec(),
            reference_id: source_reference_id.to_vec(),
            declared_size,
            backup_class: 1,
            ttl_seconds: 30,
            receipt_sha256: receipt_sha256.to_vec(),
            custody_source_proof: custody_source_proof.to_vec(),
            evidence_id: evidence_id.to_vec(),
            evidence_envelope_sha256: evidence_envelope_sha256.to_vec(),
        })),
    };
    let bytes = request.encode_to_vec();
    if bytes.len() > CONTROL_FRAME_BYTES {
        return Err(BlobClientError::InvalidSessionRequest);
    }
    channel
        .set_read_timeout(Some(Duration::from_secs(5)))
        .and_then(|_| channel.set_write_timeout(Some(Duration::from_secs(5))))
        .map_err(|error| BlobClientError::Io(error.to_string()))?;
    write_frame(channel, &bytes)?;
    let response = ManagedRuntimeControlResponseV1::decode(read_frame(channel)?.as_slice())
        .map_err(|_| BlobClientError::InvalidResponse)?;
    channel
        .set_read_timeout(None)
        .and_then(|_| channel.set_write_timeout(None))
        .map_err(|error| BlobClientError::Io(error.to_string()))?;
    let delivery = match response.result {
        Some(ControlResult::BlobSessionDelivery(delivery)) if response.error_code.is_empty() => delivery,
        _ if response.error_code == "managed_blob_session_unavailable" => {
            return Err(BlobClientError::Unavailable);
        }
        _ => return Err(BlobClientError::Rejected("managed_blob_custody_transfer_denied".to_owned())),
    };
    let grant = delivery.custody_transfer_grant.ok_or(BlobClientError::InvalidResponse)?;
    if delivery.grant.is_some()
        || !delivery.custody_transfer_source_proof.is_empty()
        || !Path::new(&delivery.data_socket_path).is_absolute()
        || grant.evidence_id.as_slice() != evidence_id.as_slice()
        || grant.evidence_envelope_sha256.as_slice() != evidence_envelope_sha256.as_slice()
        || grant.channel_binding_sha256 != Sha256::digest(&channel_binding).as_slice()
        || grant.target_reference_id.len() != 16
        || grant.target_reference_id.iter().all(|byte| *byte == 0)
    {
        return Err(BlobClientError::InvalidResponse);
    }
    Ok(ManagedBlobCustodyTransferV1 {
        data_socket_path: PathBuf::from(delivery.data_socket_path),
        grant,
        channel_binding,
    })
}

pub fn request_managed_blob_session(
    channel: &mut UnixStream,
    capability_id: &str,
    operation: BlobDataOperationV1,
    reference_id: &[u8],
    declared_size: u64,
    backup_class: u32,
    receipt_sha256: Option<&[u8; 32]>,
) -> Result<ManagedBlobSessionV1, BlobClientError> {
    if capability_id.is_empty()
        || capability_id.len() > 128
        || reference_id.len() != 16
        || reference_id.iter().all(|byte| *byte == 0)
        || declared_size == 0
        || !(1..=3).contains(&backup_class)
        || (receipt_sha256.is_some()
            && operation != BlobDataOperationV1::BlobDataOperationWriteV1)
    {
        return Err(BlobClientError::InvalidSessionRequest);
    }
    let mut request_id = [0_u8; 16];
    let mut channel_binding = vec![0_u8; 32];
    getrandom::fill(&mut request_id).map_err(|_| BlobClientError::Unavailable)?;
    getrandom::fill(&mut channel_binding).map_err(|_| BlobClientError::Unavailable)?;
    if request_id.iter().all(|byte| *byte == 0) || channel_binding.iter().all(|byte| *byte == 0) {
        return Err(BlobClientError::Unavailable);
    }
    let request = ManagedRuntimeControlRequestV1 {
        operation: Some(ControlOperation::IssueBlobSession(ManagedRuntimeBlobSessionRequestV1 {
            request_id: request_id.to_vec(),
            capability_id: capability_id.to_owned(),
            operation: operation as u32,
            channel_binding_sha256: Sha256::digest(&channel_binding).to_vec(),
            reference_id: reference_id.to_vec(),
            declared_size,
            backup_class,
            ttl_seconds: 30,
            receipt_sha256: receipt_sha256.map_or_else(Vec::new, |digest| digest.to_vec()),
            custody_source_proof: Vec::new(),
            evidence_id: Vec::new(),
            evidence_envelope_sha256: Vec::new(),
        })),
    };
    let bytes = request.encode_to_vec();
    if bytes.len() > CONTROL_FRAME_BYTES {
        return Err(BlobClientError::InvalidSessionRequest);
    }
    channel
        .set_read_timeout(Some(Duration::from_secs(5)))
        .and_then(|_| channel.set_write_timeout(Some(Duration::from_secs(5))))
        .map_err(|error| BlobClientError::Io(error.to_string()))?;
    write_frame(channel, &bytes)?;
    let response = ManagedRuntimeControlResponseV1::decode(read_frame(channel)?.as_slice())
        .map_err(|_| BlobClientError::InvalidResponse)?;
    channel
        .set_read_timeout(None)
        .and_then(|_| channel.set_write_timeout(None))
        .map_err(|error| BlobClientError::Io(error.to_string()))?;
    let delivery = match response.result {
        Some(ControlResult::BlobSessionDelivery(delivery)) if response.error_code.is_empty() => delivery,
        _ if response.error_code == "managed_blob_session_unavailable" => {
            return Err(BlobClientError::Unavailable);
        }
        _ => return Err(BlobClientError::Rejected("managed_blob_session_denied".to_owned())),
    };
    let grant = delivery.grant.ok_or(BlobClientError::InvalidResponse)?;
    if !Path::new(&delivery.data_socket_path).is_absolute()
        || grant.reference_id != reference_id
        || grant.declared_size != declared_size
        || grant.operation != operation as i32
        || grant.channel_binding_sha256 != Sha256::digest(&channel_binding).as_slice()
        || (receipt_sha256.is_some() && delivery.custody_transfer_source_proof.is_empty())
    {
        return Err(BlobClientError::InvalidResponse);
    }
    Ok(ManagedBlobSessionV1 {
        data_socket_path: PathBuf::from(delivery.data_socket_path),
        grant,
        channel_binding,
        custody_transfer_source_proof: delivery.custody_transfer_source_proof,
    })
}

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

    pub fn custody_transfer(
        &self,
        grant: BlobCustodyTransferGrantV1,
        channel_binding: Vec<u8>,
    ) -> Result<(), BlobClientError> {
        let response = self.request(BlobDataRequestV1 {
            grant: None,
            channel_binding: Vec::new(),
            operation: Some(Operation::CustodyTransfer(BlobDataCustodyTransferRequestV1 {
                grant: Some(grant),
                channel_binding,
            })),
        })?;
        if response.accepted {
            return Ok(());
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
    InvalidSessionRequest,
    Unavailable,
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
