//! Typed plaintext commands carried only inside an authenticated Vault session.

use sha2::{Digest, Sha256};

use crate::{LeaseIdV1, MAX_SESSION_CREDENTIAL_BYTES, SecretClassV1};

const COMMAND_MAJOR: u8 = 1;
const RESOLVE_LEASE_OPERATION: u8 = 1;
const STORE_LEASE_OPERATION: u8 = 2;
const REPLACE_LEASE_OPERATION: u8 = 3;
const RESOLVE_LEASE_BYTES: usize = 35;
const STORE_LEASE_HEADER_BYTES: usize = RESOLVE_LEASE_BYTES;
const REPLACE_LEASE_HEADER_BYTES: usize = STORE_LEASE_HEADER_BYTES + 16;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VaultTransportCommandV1 {
    ResolveLease {
        lease_id: LeaseIdV1,
        secret_class: SecretClassV1,
    },
    StoreLease {
        lease_id: LeaseIdV1,
        secret_class: SecretClassV1,
        payload: Vec<u8>,
    },
    ReplaceLease {
        lease_id: LeaseIdV1,
        secret_class: SecretClassV1,
        prior_record_id: [u8; 16],
        payload: Vec<u8>,
    },
}

impl VaultTransportCommandV1 {
    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        match self {
            Self::ResolveLease {
                lease_id,
                secret_class,
            } => {
                let mut bytes = Vec::with_capacity(RESOLVE_LEASE_BYTES);
                bytes.extend_from_slice(&[COMMAND_MAJOR, RESOLVE_LEASE_OPERATION]);
                bytes.push(secret_class.code() as u8);
                bytes.extend_from_slice(lease_id.as_str().as_bytes());
                bytes
            }
            Self::StoreLease {
                lease_id,
                secret_class,
                payload,
            } => {
                let mut bytes = Vec::with_capacity(STORE_LEASE_HEADER_BYTES + payload.len());
                bytes.extend_from_slice(&[COMMAND_MAJOR, STORE_LEASE_OPERATION]);
                bytes.push(secret_class.code() as u8);
                bytes.extend_from_slice(lease_id.as_str().as_bytes());
                bytes.extend_from_slice(payload);
                bytes
            }
            Self::ReplaceLease {
                lease_id,
                secret_class,
                prior_record_id,
                payload,
            } => {
                let mut bytes = Vec::with_capacity(REPLACE_LEASE_HEADER_BYTES + payload.len());
                bytes.extend_from_slice(&[COMMAND_MAJOR, REPLACE_LEASE_OPERATION]);
                bytes.push(secret_class.code() as u8);
                bytes.extend_from_slice(lease_id.as_str().as_bytes());
                bytes.extend_from_slice(prior_record_id);
                bytes.extend_from_slice(payload);
                bytes
            }
        }
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, VaultTransportCommandError> {
        if bytes.len() < RESOLVE_LEASE_BYTES || bytes.first() != Some(&COMMAND_MAJOR) {
            return Err(VaultTransportCommandError::Malformed);
        }
        match bytes[1] {
            RESOLVE_LEASE_OPERATION if bytes.len() == RESOLVE_LEASE_BYTES => {
                decode_resolve_lease(bytes[2], &bytes[3..])
            }
            STORE_LEASE_OPERATION => decode_store_lease(bytes[2], &bytes[3..]),
            REPLACE_LEASE_OPERATION => decode_replace_lease(bytes[2], &bytes[3..]),
            _ => Err(VaultTransportCommandError::Malformed),
        }
    }

    #[must_use]
    pub fn operation_digest(&self) -> [u8; 32] {
        Sha256::digest(self.encode()).into()
    }
}

fn decode_replace_lease(
    secret_class: u8,
    bytes: &[u8],
) -> Result<VaultTransportCommandV1, VaultTransportCommandError> {
    if bytes.len() <= REPLACE_LEASE_HEADER_BYTES - 3
        || bytes.len() > REPLACE_LEASE_HEADER_BYTES - 3 + MAX_SESSION_CREDENTIAL_BYTES
    {
        return Err(VaultTransportCommandError::Malformed);
    }
    let secret_class = SecretClassV1::from_code(i64::from(secret_class))
        .ok_or(VaultTransportCommandError::Malformed)?;
    let lease_id = decode_lease_id(&bytes[..RESOLVE_LEASE_BYTES - 3])?;
    let prior_record_id = bytes[RESOLVE_LEASE_BYTES - 3..REPLACE_LEASE_HEADER_BYTES - 3]
        .try_into()
        .map_err(|_| VaultTransportCommandError::Malformed)?;
    Ok(VaultTransportCommandV1::ReplaceLease {
        lease_id,
        secret_class,
        prior_record_id,
        payload: bytes[REPLACE_LEASE_HEADER_BYTES - 3..].to_vec(),
    })
}

fn decode_store_lease(
    secret_class: u8,
    bytes: &[u8],
) -> Result<VaultTransportCommandV1, VaultTransportCommandError> {
    let (lease_bytes, payload) = bytes.split_at(RESOLVE_LEASE_BYTES - 3);
    if payload.is_empty() || payload.len() > MAX_SESSION_CREDENTIAL_BYTES {
        return Err(VaultTransportCommandError::Malformed);
    }
    let secret_class = SecretClassV1::from_code(i64::from(secret_class))
        .ok_or(VaultTransportCommandError::Malformed)?;
    let lease_id = decode_lease_id(lease_bytes)?;
    Ok(VaultTransportCommandV1::StoreLease {
        lease_id,
        secret_class,
        payload: payload.to_vec(),
    })
}

fn decode_resolve_lease(
    secret_class: u8,
    bytes: &[u8],
) -> Result<VaultTransportCommandV1, VaultTransportCommandError> {
    let secret_class = SecretClassV1::from_code(i64::from(secret_class))
        .ok_or(VaultTransportCommandError::Malformed)?;
    let lease_id = decode_lease_id(bytes)?;
    Ok(VaultTransportCommandV1::ResolveLease {
        lease_id,
        secret_class,
    })
}

fn decode_lease_id(bytes: &[u8]) -> Result<LeaseIdV1, VaultTransportCommandError> {
    std::str::from_utf8(bytes)
        .ok()
        .and_then(|value| LeaseIdV1::new(value.to_owned()).ok())
        .ok_or(VaultTransportCommandError::Malformed)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VaultTransportCommandError {
    Malformed,
}
