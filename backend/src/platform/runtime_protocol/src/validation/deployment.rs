//! Structural validation for pre-gate deployment and enrollment declarations.

use crate::v1::{
    DeploymentProfileV1, DeviceProofV1, DistributionArtifactV1, InitialOwnerEnrollmentTransportV1,
    RuntimeLifecycleV1, device_proof_v1, distribution_artifact_v1,
    initial_owner_enrollment_transport_v1,
};

pub const MACOS_TAURI_TARGET: &str = "aarch64-apple-darwin";
const MAX_ENDPOINT_BYTES: usize = 1024;
const MAX_INHERITED_FD: u32 = 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeploymentValidationError {
    InvalidProfile,
    InvalidLifecycle,
    InvalidArtifact,
    InvalidDeviceProof,
    InvalidEnrollmentTransport,
}

pub fn validate_deployment_binding(
    profile: i32,
    lifecycle: i32,
    artifact: &DistributionArtifactV1,
) -> Result<(), DeploymentValidationError> {
    let profile = DeploymentProfileV1::try_from(profile)
        .map_err(|_| DeploymentValidationError::InvalidProfile)?;
    let lifecycle = RuntimeLifecycleV1::try_from(lifecycle)
        .map_err(|_| DeploymentValidationError::InvalidLifecycle)?;
    match (profile, lifecycle, artifact.artifact.as_ref()) {
        (
            DeploymentProfileV1::MacosTauriEmbedded,
            RuntimeLifecycleV1::ManagedChild,
            Some(distribution_artifact_v1::Artifact::NativeExecutable(native)),
        ) if native.target_triple == MACOS_TAURI_TARGET && valid_digest(&native.sha256) => Ok(()),
        (
            DeploymentProfileV1::LinuxDockerServer,
            RuntimeLifecycleV1::ExternalCompose,
            Some(distribution_artifact_v1::Artifact::OciImage(image)),
        ) if valid_oci_repository(&image.repository) && valid_digest(&image.sha256) => Ok(()),
        (DeploymentProfileV1::Unspecified, _, _)
        | (_, RuntimeLifecycleV1::Unspecified, _)
        | (_, _, None) => Err(DeploymentValidationError::InvalidArtifact),
        _ => Err(DeploymentValidationError::InvalidLifecycle),
    }
}

pub fn validate_initial_owner_enrollment_transport(
    transport: &InitialOwnerEnrollmentTransportV1,
) -> Result<(), DeploymentValidationError> {
    match transport.transport.as_ref() {
        Some(initial_owner_enrollment_transport_v1::Transport::InheritedFd(enrollment))
            if enrollment.inherited_fd > 2 && enrollment.inherited_fd <= MAX_INHERITED_FD =>
        {
            validate_device_proof(enrollment.device_proof.as_ref())
        }
        Some(initial_owner_enrollment_transport_v1::Transport::RemotePairing(enrollment))
            if valid_remote_endpoint(&enrollment.endpoint)
                && valid_digest(&enrollment.tls_certificate_sha256)
                && enrollment.one_time_token.len() == 32 =>
        {
            validate_device_proof(enrollment.device_proof.as_ref())
        }
        _ => Err(DeploymentValidationError::InvalidEnrollmentTransport),
    }
}

fn validate_device_proof(proof: Option<&DeviceProofV1>) -> Result<(), DeploymentValidationError> {
    match proof.and_then(|proof| proof.proof.as_ref()) {
        Some(device_proof_v1::Proof::FileEs256(proof))
            if proof.public_key_sec1.len() == 65
                && proof.public_key_sec1.first() == Some(&0x04)
                && proof.signature_raw.len() == 64 =>
        {
            Ok(())
        }
        _ => Err(DeploymentValidationError::InvalidDeviceProof),
    }
}

fn valid_digest(value: &[u8]) -> bool {
    value.len() == 32
}

fn valid_oci_repository(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_ENDPOINT_BYTES
        && value.is_ascii()
        && !value.contains('@')
        && !value.contains(':')
        && !value.contains([' ', '\t', '\n', '\r'])
}

fn valid_remote_endpoint(value: &str) -> bool {
    value.starts_with("https://")
        && value.len() > "https://".len()
        && value.len() <= MAX_ENDPOINT_BYTES
        && value.is_ascii()
        && !value.contains([' ', '\t', '\n', '\r', '#', '?'])
}
