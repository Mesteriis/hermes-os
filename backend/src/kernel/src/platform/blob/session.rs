//! Issues one-use Blob data sessions from approved capability quotas.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use hermes_kernel_control_store_sqlite::SqliteControlStore;
use hermes_runtime_protocol::v1::{
    BlobCustodySourceProofV1, BlobCustodyTransferGrantV1, BlobDataOperationV1, BlobDataSessionGrantV1, ManagedRuntimeBlobSessionDeliveryV1,
    ManagedRuntimeBlobSessionRequestV1,
};
use prost::Message;
use p256::ecdsa::{Signature, VerifyingKey, signature::Verifier};
use sha2::{Digest, Sha256};

use crate::identity::device::signer::{DeviceSigner, FileDeviceSigner};
use crate::platform::blob::{catalog, launch, status};
use crate::runtime::lifecycle::control::{
    ManagedRuntimeBlobSessionHandler, ManagedRuntimeExpectation,
};
use crate::runtime::lifecycle::supervisor::ManagedRuntimeRelayPort;

const MAX_SESSION_TTL_SECONDS: u32 = 30;
const CUSTODY_SOURCE_PROOF_TTL_MS: u64 = 24 * 60 * 60 * 1_000;

/// Kernel authority for an exact direct Blob data operation.
pub(crate) struct BlobSessionHandlerV1 {
    store: Arc<SqliteControlStore>,
    relay: ManagedRuntimeRelayPort,
    data_dir: PathBuf,
}

impl BlobSessionHandlerV1 {
    #[must_use]
    pub(crate) fn new(
        store: Arc<SqliteControlStore>,
        relay: ManagedRuntimeRelayPort,
        data_dir: PathBuf,
    ) -> Self {
        Self { store, relay, data_dir }
    }
}

impl ManagedRuntimeBlobSessionHandler for BlobSessionHandlerV1 {
    fn issue_blob_session(
        &self,
        expectation: &ManagedRuntimeExpectation,
        request: ManagedRuntimeBlobSessionRequestV1,
    ) -> Result<ManagedRuntimeBlobSessionDeliveryV1, String> {
        let entry = catalog::resolve(&*self.store)?
            .into_iter()
            .find(|entry| {
                entry.registration_id() == expectation.registration_id()
                    && entry.capability_id() == request.capability_id
                    && entry.grant_epoch() == expectation.grant_epoch()
            })
            .ok_or_else(|| "managed runtime Blob session request is denied".to_owned())?;
        let operation = i32::try_from(request.operation)
            .ok()
            .and_then(|value| BlobDataOperationV1::try_from(value).ok())
            .filter(|value| {
                matches!(
                    value,
                    BlobDataOperationV1::BlobDataOperationWriteV1
                        | BlobDataOperationV1::BlobDataOperationReadRangeV1
                        | BlobDataOperationV1::BlobDataOperationCustodyTransferV1
                )
            })
            .ok_or_else(|| "managed runtime Blob session request is denied".to_owned())?;
        if (operation == BlobDataOperationV1::BlobDataOperationWriteV1
            && !request.receipt_sha256.is_empty()
            && (request.receipt_sha256.len() != 32
                || request.receipt_sha256.iter().all(|byte| *byte == 0)))
            || (operation != BlobDataOperationV1::BlobDataOperationWriteV1
                && operation != BlobDataOperationV1::BlobDataOperationCustodyTransferV1
                && !request.receipt_sha256.is_empty())
        {
            return Err("managed runtime Blob session request is denied".to_owned());
        }
        if entry.request().owner_id().is_empty()
            || request.declared_size == 0
            || request.declared_size > entry.request().max_bytes()
        {
            return Err("managed runtime Blob session request is denied".to_owned());
        }
        if operation == BlobDataOperationV1::BlobDataOperationCustodyTransferV1 {
            return self.issue_custody_transfer(expectation, request, entry);
        }
        if !request.custody_source_proof.is_empty()
            || !request.evidence_id.is_empty()
            || !request.evidence_envelope_sha256.is_empty()
        {
            return Err("managed runtime Blob session request is denied".to_owned());
        }
        let now = now_unix_ms()?;
        let expires_at_unix_ms = now
            .checked_add(u64::from(request.ttl_seconds) * 1_000)
            .ok_or_else(|| "managed runtime Blob session request is denied".to_owned())?;
        let blob = status::read_current(&self.store, &self.relay)?;
        let mut session_id = [0_u8; 16];
        getrandom::fill(&mut session_id)
            .map_err(|_| "managed runtime Blob session request is unavailable".to_owned())?;
        if session_id.iter().all(|byte| *byte == 0) {
            return Err("managed runtime Blob session request is unavailable".to_owned());
        }
        let mut grant = BlobDataSessionGrantV1 {
            major: 1,
            kernel_instance_id: self.store.snapshot().instance_id().to_owned(),
            session_id: session_id.to_vec(),
            channel_binding_sha256: request.channel_binding_sha256,
            owner_id: entry.request().owner_id().to_owned(),
            registration_id: expectation.registration_id().to_owned(),
            capability_id: request.capability_id,
            runtime_instance_id: expectation.runtime_instance_id().to_owned(),
            runtime_generation: expectation.runtime_generation(),
            grant_epoch: expectation.grant_epoch(),
            // Blob content keys rotate with the only current durable access revision.
            key_revision: expectation.grant_epoch(),
            quota_max_bytes: entry.request().max_bytes(),
            reference_id: request.reference_id,
            declared_size: request.declared_size,
            reference_expires_at_unix_ms: 0,
            backup_class: i32::try_from(request.backup_class)
                .map_err(|_| "managed runtime Blob session request is denied".to_owned())?,
            operation: operation as i32,
            expires_at_unix_ms,
            kernel_authorization_signature_raw: Vec::new(),
            blob_runtime_generation: blob.runtime_generation(),
        };
        let signer = FileDeviceSigner::open_for_instance(&self.data_dir)?;
        let mut message = b"hermes.blob-data-session.v1\0".to_vec();
        message.extend_from_slice(&grant.encode_to_vec());
        grant.kernel_authorization_signature_raw = signer.sign(&message).to_vec();
        let custody_transfer_source_proof = if request.receipt_sha256.is_empty() {
            Vec::new()
        } else {
            issue_custody_source_proof(&signer, &grant, &request.receipt_sha256, now)?
        };
        Ok(ManagedRuntimeBlobSessionDeliveryV1 {
            data_socket_path: launch::data_socket_path(&self.data_dir).display().to_string(),
            grant: Some(grant),
            custody_transfer_source_proof,
            custody_transfer_grant: None,
        })
    }
}

impl BlobSessionHandlerV1 {
    fn issue_custody_transfer(
        &self,
        expectation: &ManagedRuntimeExpectation,
        request: ManagedRuntimeBlobSessionRequestV1,
        target: crate::platform::blob::catalog::BlobQuotaCatalogEntryV1,
    ) -> Result<ManagedRuntimeBlobSessionDeliveryV1, String> {
        if request.receipt_sha256.len() != 32
            || request.reference_id.len() != 16
            || request.evidence_id.len() != 16
            || request.evidence_envelope_sha256.len() != 32
            || request.custody_source_proof.is_empty()
            || request.custody_source_proof.len() > 2_048
            || request.declared_size == 0
            || !matches!(request.backup_class, 1 | 2 | 3)
        {
            return Err("managed runtime Blob custody transfer is denied".to_owned());
        }
        let now = now_unix_ms()?;
        let signer = FileDeviceSigner::open_for_instance(&self.data_dir)?;
        let source = verify_custody_source_proof(
            &request.custody_source_proof,
            &signer.public_key_sec1(),
            self.store.snapshot().instance_id(),
            now,
        )?;
        if source.reference_id != request.reference_id
            || source.declared_size != request.declared_size
            || source.receipt_sha256 != request.receipt_sha256
            || source.owner_id.as_str() != target.request().owner_id()
            || !catalog::resolve(&*self.store)?.iter().any(|entry| {
                entry.registration_id() == source.registration_id.as_str()
                    && entry.capability_id() == source.capability_id.as_str()
                    && entry.grant_epoch() == source.grant_epoch
                    && entry.request().owner_id() == source.owner_id.as_str()
            })
        {
            return Err("managed runtime Blob custody transfer is denied".to_owned());
        }
        let blob = status::read_current(&self.store, &self.relay)?;
        let mut session_id = [0_u8; 16];
        getrandom::fill(&mut session_id)
            .map_err(|_| "managed runtime Blob custody transfer is unavailable".to_owned())?;
        let target_reference_id = transfer_target_reference(&request.custody_source_proof, &request.evidence_id, &request.evidence_envelope_sha256);
        let expires_at_unix_ms = now.checked_add(u64::from(request.ttl_seconds) * 1_000)
            .ok_or_else(|| "managed runtime Blob custody transfer is unavailable".to_owned())?;
        let mut grant = BlobCustodyTransferGrantV1 {
            major: 1,
            kernel_instance_id: self.store.snapshot().instance_id().to_owned(),
            session_id: session_id.to_vec(),
            channel_binding_sha256: request.channel_binding_sha256,
            evidence_id: request.evidence_id,
            evidence_envelope_sha256: request.evidence_envelope_sha256,
            source: Some(source),
            target_owner_id: target.request().owner_id().to_owned(),
            target_registration_id: expectation.registration_id().to_owned(),
            target_capability_id: request.capability_id,
            target_runtime_instance_id: expectation.runtime_instance_id().to_owned(),
            target_runtime_generation: expectation.runtime_generation(),
            target_grant_epoch: expectation.grant_epoch(),
            target_key_revision: expectation.grant_epoch(),
            target_quota_max_bytes: target.request().max_bytes(),
            target_reference_id: target_reference_id.to_vec(),
            expires_at_unix_ms,
            blob_runtime_generation: blob.runtime_generation(),
            kernel_authorization_signature_raw: Vec::new(),
        };
        let mut message = b"hermes.blob-custody-transfer.v1\0".to_vec();
        message.extend_from_slice(&grant.encode_to_vec());
        grant.kernel_authorization_signature_raw = signer.sign(&message).to_vec();
        Ok(ManagedRuntimeBlobSessionDeliveryV1 {
            data_socket_path: launch::data_socket_path(&self.data_dir).display().to_string(),
            grant: None,
            custody_transfer_source_proof: Vec::new(),
            custody_transfer_grant: Some(grant),
        })
    }
}

pub(crate) fn valid_request(request: &ManagedRuntimeBlobSessionRequestV1) -> bool {
    request.request_id.len() == 16
        && request.request_id.iter().any(|byte| *byte != 0)
        && !request.capability_id.is_empty()
        && request.capability_id.len() <= 128
        && request.channel_binding_sha256.len() == 32
        && request.reference_id.len() == 16
        && request.reference_id.iter().any(|byte| *byte != 0)
        && request.declared_size > 0
        && (1..=3).contains(&request.backup_class)
        && (1..=MAX_SESSION_TTL_SECONDS).contains(&request.ttl_seconds)
        && (request.receipt_sha256.is_empty()
            || (request.receipt_sha256.len() == 32
                && request.receipt_sha256.iter().any(|byte| *byte != 0)))
}

fn verify_custody_source_proof(
    encoded: &[u8],
    public_key_sec1: &[u8; 65],
    kernel_instance_id: &str,
    now_unix_ms: u64,
) -> Result<BlobCustodySourceProofV1, String> {
    let proof = BlobCustodySourceProofV1::decode(encoded)
        .map_err(|_| "managed runtime Blob custody transfer is denied".to_owned())?;
    if proof.major != 1
        || proof.kernel_instance_id != kernel_instance_id
        || proof.owner_id.is_empty()
        || proof.registration_id.is_empty()
        || proof.capability_id.is_empty()
        || proof.runtime_instance_id.is_empty()
        || proof.runtime_generation == 0
        || proof.grant_epoch == 0
        || proof.key_revision == 0
        || proof.reference_id.len() != 16
        || proof.reference_id.iter().all(|byte| *byte == 0)
        || proof.declared_size == 0
        || proof.receipt_sha256.len() != 32
        || proof.expires_at_unix_ms <= now_unix_ms
        || proof.issued_at_unix_ms > now_unix_ms
        || proof.kernel_authorization_signature_raw.len() != 64
    {
        return Err("managed runtime Blob custody transfer is denied".to_owned());
    }
    let signature = Signature::from_slice(&proof.kernel_authorization_signature_raw)
        .map_err(|_| "managed runtime Blob custody transfer is denied".to_owned())?;
    let key = VerifyingKey::from_sec1_bytes(public_key_sec1)
        .map_err(|_| "managed runtime Blob custody transfer is denied".to_owned())?;
    let mut unsigned = proof.clone();
    unsigned.kernel_authorization_signature_raw.clear();
    let mut message = b"hermes.blob-custody-source-proof.v1\0".to_vec();
    message.extend_from_slice(&unsigned.encode_to_vec());
    key.verify(&message, &signature)
        .map_err(|_| "managed runtime Blob custody transfer is denied".to_owned())?;
    Ok(proof)
}

fn transfer_target_reference(source_proof: &[u8], evidence_id: &[u8], envelope_hash: &[u8]) -> [u8; 16] {
    let mut digest = Sha256::new();
    digest.update(b"hermes.blob-custody-target-reference.v1\0");
    digest.update(source_proof);
    digest.update(evidence_id);
    digest.update(envelope_hash);
    let hash: [u8; 32] = digest.finalize().into();
    let mut reference_id = [0_u8; 16];
    reference_id.copy_from_slice(&hash[..16]);
    if reference_id.iter().all(|byte| *byte == 0) {
        reference_id[0] = 1;
    }
    reference_id
}

fn issue_custody_source_proof(
    signer: &FileDeviceSigner,
    grant: &BlobDataSessionGrantV1,
    receipt_sha256: &[u8],
    now_unix_ms: u64,
) -> Result<Vec<u8>, String> {
    let expires_at_unix_ms = now_unix_ms
        .checked_add(CUSTODY_SOURCE_PROOF_TTL_MS)
        .ok_or_else(|| "managed runtime Blob session request is unavailable".to_owned())?;
    let mut proof = BlobCustodySourceProofV1 {
        major: 1,
        kernel_instance_id: grant.kernel_instance_id.clone(),
        owner_id: grant.owner_id.clone(),
        registration_id: grant.registration_id.clone(),
        capability_id: grant.capability_id.clone(),
        runtime_instance_id: grant.runtime_instance_id.clone(),
        runtime_generation: grant.runtime_generation,
        grant_epoch: grant.grant_epoch,
        key_revision: grant.key_revision,
        reference_id: grant.reference_id.clone(),
        declared_size: grant.declared_size,
        receipt_sha256: receipt_sha256.to_vec(),
        issued_at_unix_ms: now_unix_ms,
        expires_at_unix_ms,
        kernel_authorization_signature_raw: Vec::new(),
        backup_class: grant.backup_class,
        reference_expires_at_unix_ms: grant.reference_expires_at_unix_ms,
    };
    let mut message = b"hermes.blob-custody-source-proof.v1\0".to_vec();
    message.extend_from_slice(&proof.encode_to_vec());
    proof.kernel_authorization_signature_raw = signer.sign(&message).to_vec();
    Ok(proof.encode_to_vec())
}

fn now_unix_ms() -> Result<u64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "managed runtime Blob session request is unavailable".to_owned())?
        .as_millis()
        .try_into()
        .map_err(|_| "managed runtime Blob session request is unavailable".to_owned())
}
