//! Structural limits for opaque Vault ciphertext routing metadata.

use crate::v1::{VaultCiphertextRouteDirectionV1, VaultCiphertextRouteV1};

pub const MAX_VAULT_CIPHERTEXT_BYTES: usize = 256 * 1024;
const ID_BYTES: usize = 256;
const REQUEST_ID_BYTES: usize = 16;
const SHA256_BYTES: usize = 32;
const HPKE_TAG_BYTES: usize = 16;
const ES256_SIGNATURE_BYTES: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VaultCiphertextRouteValidationError {
    InvalidVersion,
    InvalidIdentity,
    InvalidFence,
    InvalidDirection,
    InvalidBinding,
    InvalidCiphertext,
}

pub fn validate_vault_ciphertext_route_v1(
    route: &VaultCiphertextRouteV1,
) -> Result<(), VaultCiphertextRouteValidationError> {
    if route.major != 1 {
        return Err(VaultCiphertextRouteValidationError::InvalidVersion);
    }
    if !valid_id(&route.registration_id) || !valid_id(&route.runtime_instance_id) {
        return Err(VaultCiphertextRouteValidationError::InvalidIdentity);
    }
    if route.vault_runtime_generation == 0 || route.grant_epoch == 0 {
        return Err(VaultCiphertextRouteValidationError::InvalidFence);
    }
    if VaultCiphertextRouteDirectionV1::try_from(route.direction).ok()
        != Some(VaultCiphertextRouteDirectionV1::ToVault)
    {
        return Err(VaultCiphertextRouteValidationError::InvalidDirection);
    }
    if route.request_id.len() != REQUEST_ID_BYTES
        || route.operation_digest_sha256.len() != SHA256_BYTES
        || route.hpke_encapped_key.len() != SHA256_BYTES
        || route.hpke_authentication_tag.len() != HPKE_TAG_BYTES
        || route.response_recipient_hpke_public_key_x25519.len() != SHA256_BYTES
    {
        return Err(VaultCiphertextRouteValidationError::InvalidBinding);
    }
    if route.ciphertext.is_empty() || route.ciphertext.len() > MAX_VAULT_CIPHERTEXT_BYTES {
        return Err(VaultCiphertextRouteValidationError::InvalidCiphertext);
    }
    if !route.kernel_authorization_signature_raw.is_empty()
        && (!valid_id(&route.kernel_instance_id)
            || route.kernel_authorization_signature_raw.len() != ES256_SIGNATURE_BYTES)
    {
        return Err(VaultCiphertextRouteValidationError::InvalidBinding);
    }
    Ok(())
}

fn valid_id(value: &str) -> bool {
    !value.is_empty() && value.len() <= ID_BYTES && value.is_ascii()
}
