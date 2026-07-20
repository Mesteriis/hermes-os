//! Validation for the public, secret-free Storage runtime status contract.

use crate::{
    StorageBindingErrorV1,
    v1::{
        StorageRuntimeControlRequestV1, StorageRuntimeControlResponseV1, StorageRuntimeStateV1,
        StorageRuntimeStatusV1, storage_runtime_control_request_v1::Operation as RequestOperation,
        storage_runtime_control_response_v1::Result as ResponseResult,
    },
    validation::{validate_storage_binding_message, validate_storage_bundle},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorageRuntimeStatusErrorV1 {
    InvalidState,
    InvalidGeneration,
    InvalidBinding,
    InvalidBlocker,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorageRuntimeControlErrorV1 {
    MissingOperation,
    InvalidStatus,
    InvalidRevocation,
    InvalidApply,
    InvalidError,
}

pub fn validate_storage_runtime_control_request(
    request: &StorageRuntimeControlRequestV1,
) -> Result<(), StorageRuntimeControlErrorV1> {
    match &request.operation {
        Some(RequestOperation::GetStatus(_)) => Ok(()),
        Some(RequestOperation::RevokeBinding(request)) => request
            .binding
            .as_ref()
            .ok_or(StorageRuntimeControlErrorV1::InvalidRevocation)
            .and_then(|binding| {
                validate_storage_binding_message(binding)
                    .map_err(|_| StorageRuntimeControlErrorV1::InvalidRevocation)
            }),
        Some(RequestOperation::ApplyBinding(request)) => request
            .binding
            .as_ref()
            .zip(request.bundle.as_ref())
            .ok_or(StorageRuntimeControlErrorV1::InvalidApply)
            .and_then(|(binding, bundle)| {
                validate_storage_binding_message(binding)
                    .map_err(|_| StorageRuntimeControlErrorV1::InvalidApply)
                    .and_then(|_| {
                        validate_storage_bundle(bundle)
                            .map_err(|_| StorageRuntimeControlErrorV1::InvalidApply)
                    })
            }),
        None => Err(StorageRuntimeControlErrorV1::MissingOperation),
    }
}

pub fn validate_storage_runtime_control_response(
    response: &StorageRuntimeControlResponseV1,
) -> Result<(), StorageRuntimeControlErrorV1> {
    match (&response.result, response.error_code.is_empty()) {
        (Some(ResponseResult::Status(status)), true) => validate_storage_runtime_status(status)
            .map_err(|_| StorageRuntimeControlErrorV1::InvalidStatus),
        (Some(ResponseResult::RevokedBinding(binding)), true) => {
            validate_storage_binding_message(binding)
                .map_err(|_| StorageRuntimeControlErrorV1::InvalidRevocation)
        }
        (Some(ResponseResult::ActiveBinding(binding)), true) => {
            validate_storage_binding_message(binding)
                .map_err(|_| StorageRuntimeControlErrorV1::InvalidApply)
        }
        (None, false) if valid_error_code(&response.error_code) => Ok(()),
        _ => Err(StorageRuntimeControlErrorV1::InvalidError),
    }
}

pub fn validate_storage_runtime_status(
    status: &StorageRuntimeStatusV1,
) -> Result<(), StorageRuntimeStatusErrorV1> {
    if status.runtime_generation == 0 {
        return Err(StorageRuntimeStatusErrorV1::InvalidGeneration);
    }
    let state = StorageRuntimeStateV1::try_from(status.state)
        .map_err(|_| StorageRuntimeStatusErrorV1::InvalidState)?;
    match state {
        StorageRuntimeStateV1::Unspecified => Err(StorageRuntimeStatusErrorV1::InvalidState),
        StorageRuntimeStateV1::Unconfigured | StorageRuntimeStateV1::Stopped => {
            validate_inactive(status)
        }
        StorageRuntimeStateV1::Ready | StorageRuntimeStateV1::Revoking => validate_active(status),
        StorageRuntimeStateV1::Reconciling => validate_reconciling(status),
        StorageRuntimeStateV1::Failed => validate_failed(status),
    }
}

fn validate_inactive(status: &StorageRuntimeStatusV1) -> Result<(), StorageRuntimeStatusErrorV1> {
    if status.storage_generation != 0
        || status.topology_revision != 0
        || status.vault_runtime_generation != 0
        || !status.active_bindings.is_empty()
        || !status.blocker_code.is_empty()
    {
        return Err(StorageRuntimeStatusErrorV1::InvalidGeneration);
    }
    Ok(())
}

fn validate_active(status: &StorageRuntimeStatusV1) -> Result<(), StorageRuntimeStatusErrorV1> {
    if status.storage_generation == 0
        || status.topology_revision == 0
        || status.vault_runtime_generation == 0
        || !status.blocker_code.is_empty()
    {
        return Err(StorageRuntimeStatusErrorV1::InvalidGeneration);
    }
    (!status.active_bindings.is_empty())
        .then_some(())
        .ok_or(StorageRuntimeStatusErrorV1::InvalidBinding)?;
    validate_bindings(&status.active_bindings)
}

fn validate_reconciling(
    status: &StorageRuntimeStatusV1,
) -> Result<(), StorageRuntimeStatusErrorV1> {
    if status.storage_generation == 0
        || status.topology_revision == 0
        || status.vault_runtime_generation == 0
        || !status.blocker_code.is_empty()
    {
        return Err(StorageRuntimeStatusErrorV1::InvalidGeneration);
    }
    validate_bindings(&status.active_bindings)
}

fn validate_failed(status: &StorageRuntimeStatusV1) -> Result<(), StorageRuntimeStatusErrorV1> {
    if status.blocker_code.is_empty() || status.blocker_code.len() > 96 {
        return Err(StorageRuntimeStatusErrorV1::InvalidBlocker);
    }
    if !status.active_bindings.is_empty() {
        return Err(StorageRuntimeStatusErrorV1::InvalidBinding);
    }
    if (status.storage_generation == 0) != (status.topology_revision == 0) {
        return Err(StorageRuntimeStatusErrorV1::InvalidGeneration);
    }
    if status.storage_generation != 0 && status.vault_runtime_generation == 0 {
        return Err(StorageRuntimeStatusErrorV1::InvalidGeneration);
    }
    Ok(())
}

fn validate_bindings(
    bindings: &[crate::v1::StorageBindingV1],
) -> Result<(), StorageRuntimeStatusErrorV1> {
    bindings.iter().try_for_each(|binding| {
        validate_storage_binding_message(binding).map_err(map_binding_error)
    })
}

const fn map_binding_error(_: StorageBindingErrorV1) -> StorageRuntimeStatusErrorV1 {
    StorageRuntimeStatusErrorV1::InvalidBinding
}

fn valid_error_code(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 96
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}
