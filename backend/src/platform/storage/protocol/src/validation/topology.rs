//! Validation for the non-secret Storage runtime topology declaration.

use crate::v1::{StorageDeploymentProfileV1, StorageRuntimeTopologyV1};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorageRuntimeTopologyErrorV1 {
    InvalidRevision,
    InvalidIdentity,
    InvalidProfile,
    InvalidArtifactDigest,
    InvalidEndpoint,
}

pub fn validate_storage_runtime_topology(
    topology: &StorageRuntimeTopologyV1,
) -> Result<(), StorageRuntimeTopologyErrorV1> {
    if topology.topology_revision == 0 || topology.storage_generation == 0 {
        return Err(StorageRuntimeTopologyErrorV1::InvalidRevision);
    }
    if !valid_token(&topology.storage_instance_id) || !valid_token(&topology.database_id) {
        return Err(StorageRuntimeTopologyErrorV1::InvalidIdentity);
    }
    if StorageDeploymentProfileV1::try_from(topology.deployment_profile).ok()
        == Some(StorageDeploymentProfileV1::Unspecified)
        || StorageDeploymentProfileV1::try_from(topology.deployment_profile).is_err()
    {
        return Err(StorageRuntimeTopologyErrorV1::InvalidProfile);
    }
    if !valid_digest(&topology.postgres_artifact_sha256)
        || !valid_digest(&topology.pgbouncer_artifact_sha256)
    {
        return Err(StorageRuntimeTopologyErrorV1::InvalidArtifactDigest);
    }
    if !valid_endpoint(&topology.postgres_host, topology.postgres_port)
        || !valid_endpoint(&topology.pgbouncer_host, topology.pgbouncer_port)
        || !valid_endpoint(
            &topology.pgbouncer_postgres_host,
            topology.pgbouncer_postgres_port,
        )
    {
        return Err(StorageRuntimeTopologyErrorV1::InvalidEndpoint);
    }
    Ok(())
}

fn valid_token(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
}

fn valid_digest(value: &[u8]) -> bool {
    value.len() == 32 && value.iter().any(|byte| *byte != 0)
}

fn valid_endpoint(host: &str, port: u32) -> bool {
    port > 0
        && port <= u32::from(u16::MAX)
        && !host.is_empty()
        && host.len() <= 253
        && host.bytes().all(|byte| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_uppercase()
                || byte.is_ascii_digit()
                || matches!(byte, b'.' | b'-' | b':')
        })
}
