//! Structural validation for Blob managed-runtime configuration and status.

use crate::v1::{
    BlobRuntimeConfigurationV1, BlobRuntimeControlRequestV1, BlobRuntimeControlResponseV1,
    BlobRuntimeStateV1, BlobRuntimeStatusV1,
    blob_runtime_control_request_v1::Operation as RequestOperation,
    blob_runtime_control_response_v1::Result as ResponseResult,
};

const MAX_PATH_BYTES: usize = 4_096;
const MAX_UNIX_SOCKET_PATH_BYTES: usize = 100;
const MAX_BLOB_BYTES: u64 = 64 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlobRuntimeValidationErrorV1 {
    InvalidConfiguration,
    InvalidRequest,
    InvalidResponse,
    InvalidStatus,
}

pub fn validate_blob_runtime_configuration(
    configuration: &BlobRuntimeConfigurationV1,
) -> Result<(), BlobRuntimeValidationErrorV1> {
    if !configuration.data_dir.starts_with('/')
        || configuration.data_dir.len() > MAX_PATH_BYTES
        || !configuration.data_socket_path.starts_with('/')
        || configuration.data_socket_path.len() > MAX_UNIX_SOCKET_PATH_BYTES
        || configuration.maximum_blob_bytes == 0
        || configuration.maximum_blob_bytes > MAX_BLOB_BYTES
        || !valid_id(&configuration.vault_instance_id)
        || configuration.vault_runtime_generation == 0
        || configuration.vault_hpke_public_key_x25519.len() != 32
        || !valid_id(&configuration.kernel_instance_id)
        || configuration.kernel_authorization_public_key_sec1.len() != 65
    {
        return Err(BlobRuntimeValidationErrorV1::InvalidConfiguration);
    }
    Ok(())
}

pub fn validate_blob_runtime_control_request(
    request: &BlobRuntimeControlRequestV1,
) -> Result<(), BlobRuntimeValidationErrorV1> {
    matches!(request.operation, Some(RequestOperation::GetStatus(_)))
        .then_some(())
        .ok_or(BlobRuntimeValidationErrorV1::InvalidRequest)
}

pub fn validate_blob_runtime_control_response(
    response: &BlobRuntimeControlResponseV1,
) -> Result<(), BlobRuntimeValidationErrorV1> {
    match (&response.result, response.error_code.is_empty()) {
        (Some(ResponseResult::Status(status)), true) => validate_blob_runtime_status(status),
        (None, false) if valid_blocker_code(&response.error_code) => Ok(()),
        _ => Err(BlobRuntimeValidationErrorV1::InvalidResponse),
    }
}

pub fn validate_blob_runtime_status(
    status: &BlobRuntimeStatusV1,
) -> Result<(), BlobRuntimeValidationErrorV1> {
    let state = BlobRuntimeStateV1::try_from(status.state)
        .map_err(|_| BlobRuntimeValidationErrorV1::InvalidStatus)?;
    if status.runtime_generation == 0
        || status.vault_runtime_generation == 0
        || status.maximum_blob_bytes == 0
        || status.maximum_blob_bytes > MAX_BLOB_BYTES
    {
        return Err(BlobRuntimeValidationErrorV1::InvalidStatus);
    }
    match state {
        BlobRuntimeStateV1::Ready if status.blocker_code.is_empty() => Ok(()),
        BlobRuntimeStateV1::Blocked if valid_blocker_code(&status.blocker_code) => Ok(()),
        _ => Err(BlobRuntimeValidationErrorV1::InvalidStatus),
    }
}

fn valid_id(value: &str) -> bool {
    !value.is_empty() && value.len() <= 128 && value.is_ascii()
}

fn valid_blocker_code(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 96
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}
