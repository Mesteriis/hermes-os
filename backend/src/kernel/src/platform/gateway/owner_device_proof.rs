//! Shared active-owner-device proof for public Core Gateway control ceremonies.

use hermes_gateway_runtime::OwnerBrowserPrincipalV1;
use hermes_kernel_control_store::BrowserDeviceStateV1;
use hermes_kernel_control_store_sqlite::SqliteControlStore;
use p256::ecdsa::{Signature, VerifyingKey, signature::Verifier};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OwnerDeviceProofErrorV1 {
    InvalidArgument,
    PermissionDenied,
    Internal,
}

pub(crate) fn validate_active_principal(
    store: &SqliteControlStore,
    principal: &OwnerBrowserPrincipalV1,
) -> Result<(), OwnerDeviceProofErrorV1> {
    let owner = store
        .initial_owner_identity()
        .map_err(|_| OwnerDeviceProofErrorV1::Internal)?
        .ok_or(OwnerDeviceProofErrorV1::PermissionDenied)?;
    let device = store
        .browser_device_identity(principal.device_id())
        .map_err(|_| OwnerDeviceProofErrorV1::Internal)?
        .ok_or(OwnerDeviceProofErrorV1::PermissionDenied)?;
    let enrollment = device.enrollment();
    if owner.owner_id() != principal.owner_id()
        || enrollment.owner_id() != principal.owner_id()
        || enrollment.device_id() != principal.device_id()
        || device.state() != BrowserDeviceStateV1::Active
        || device.identity_epoch()
            != store
                .current_identity_epoch()
                .map_err(|_| OwnerDeviceProofErrorV1::Internal)?
    {
        return Err(OwnerDeviceProofErrorV1::PermissionDenied);
    }
    Ok(())
}

pub(crate) fn verify_fresh_proof(
    store: &SqliteControlStore,
    principal: &OwnerBrowserPrincipalV1,
    challenge_bytes: &[u8; 32],
    signature_raw: &[u8],
) -> Result<(), OwnerDeviceProofErrorV1> {
    validate_active_principal(store, principal)?;
    let device = store
        .browser_device_identity(principal.device_id())
        .map_err(|_| OwnerDeviceProofErrorV1::Internal)?
        .ok_or(OwnerDeviceProofErrorV1::PermissionDenied)?;
    let verifying_key = VerifyingKey::from_sec1_bytes(device.enrollment().browser_key_public_key())
        .map_err(|_| OwnerDeviceProofErrorV1::PermissionDenied)?;
    let signature = Signature::from_slice(signature_raw)
        .map_err(|_| OwnerDeviceProofErrorV1::InvalidArgument)?;
    verifying_key
        .verify(challenge_bytes, &signature)
        .map_err(|_| OwnerDeviceProofErrorV1::PermissionDenied)
}
