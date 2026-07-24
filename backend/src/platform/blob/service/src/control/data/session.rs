//! Verifies one-use, Kernel-signed Blob data sessions before any content access.

use std::collections::HashMap;

use hermes_blob_protocol::{BlobAccessFenceV1, BlobBackupClassV1, BlobQuotaGrantV1, BlobRefV1};
use hermes_runtime_protocol::v1::{
    BlobBackupClassV1 as WireBackupClass, BlobCustodyTransferGrantV1, BlobDataOperationV1,
    BlobDataSessionGrantV1,
};
use p256::ecdsa::{Signature, VerifyingKey, signature::Verifier};
use prost::Message;
use sha2::{Digest, Sha256};

const SESSION_TTL_LIMIT_MS: u64 = 30_000;
const MAX_ACTIVE_SESSIONS: usize = 4_096;

pub(super) struct VerifiedBlobDataSessionV1 {
    reference: BlobRefV1,
    access: BlobAccessFenceV1,
    quota: BlobQuotaGrantV1,
    key_revision: u64,
}

pub(super) struct VerifiedBlobCustodyTransferV1 {
    source_reference: BlobRefV1,
    source_access: BlobAccessFenceV1,
    source_key_revision: u64,
    target_reference: BlobRefV1,
    target_access: BlobAccessFenceV1,
    target_quota: BlobQuotaGrantV1,
    target_key_revision: u64,
    expected_plaintext_sha256: [u8; 32],
}

impl VerifiedBlobCustodyTransferV1 {
    pub(super) fn source_reference(&self) -> &BlobRefV1 {
        &self.source_reference
    }
    pub(super) fn source_access(&self) -> &BlobAccessFenceV1 {
        &self.source_access
    }
    pub(super) const fn source_key_revision(&self) -> u64 {
        self.source_key_revision
    }
    pub(super) fn target_reference(&self) -> &BlobRefV1 {
        &self.target_reference
    }
    pub(super) fn target_access(&self) -> &BlobAccessFenceV1 {
        &self.target_access
    }
    pub(super) fn target_quota(&self) -> &BlobQuotaGrantV1 {
        &self.target_quota
    }
    pub(super) const fn target_key_revision(&self) -> u64 {
        self.target_key_revision
    }
    pub(super) const fn expected_plaintext_sha256(&self) -> &[u8; 32] {
        &self.expected_plaintext_sha256
    }
}

impl VerifiedBlobDataSessionV1 {
    pub(super) fn reference(&self) -> &BlobRefV1 {
        &self.reference
    }
    pub(super) fn access(&self) -> &BlobAccessFenceV1 {
        &self.access
    }
    pub(super) fn quota(&self) -> &BlobQuotaGrantV1 {
        &self.quota
    }
    pub(super) const fn key_revision(&self) -> u64 {
        self.key_revision
    }
}

pub(crate) struct BlobDataSessionVerifierV1 {
    kernel_instance_id: String,
    blob_runtime_generation: u64,
    key: VerifyingKey,
    consumed: HashMap<[u8; 16], u64>,
}

impl BlobDataSessionVerifierV1 {
    pub(crate) fn new(
        instance_id: String,
        blob_runtime_generation: u64,
        key_sec1: &[u8],
    ) -> Result<Self, ()> {
        if blob_runtime_generation == 0 {
            return Err(());
        }
        let key = VerifyingKey::from_sec1_bytes(key_sec1).map_err(|_| ())?;
        Ok(Self {
            kernel_instance_id: instance_id,
            blob_runtime_generation,
            key,
            consumed: HashMap::new(),
        })
    }

    pub(super) fn verify(
        &mut self,
        grant: BlobDataSessionGrantV1,
        binding: &[u8],
        expected_operation: BlobDataOperationV1,
        now_unix_ms: u64,
    ) -> Result<VerifiedBlobDataSessionV1, ()> {
        self.prune(now_unix_ms);
        let session_id: [u8; 16] = grant.session_id.as_slice().try_into().map_err(|_| ())?;
        if self.consumed.len() >= MAX_ACTIVE_SESSIONS || self.consumed.contains_key(&session_id) {
            denied("reused_session");
            return Err(());
        }
        validate_signed_grant(
            &self.kernel_instance_id,
            self.blob_runtime_generation,
            &self.key,
            &grant,
            binding,
            expected_operation,
            now_unix_ms,
        )?;
        let verified = decode_grant(&grant).map_err(|_| denied("grant_shape"))?;
        self.consumed.insert(session_id, grant.expires_at_unix_ms);
        Ok(verified)
    }

    pub(super) fn verify_custody_transfer(
        &mut self,
        grant: BlobCustodyTransferGrantV1,
        binding: &[u8],
        now_unix_ms: u64,
    ) -> Result<VerifiedBlobCustodyTransferV1, ()> {
        self.prune(now_unix_ms);
        let session_id: [u8; 16] = grant.session_id.as_slice().try_into().map_err(|_| ())?;
        if self.consumed.len() >= MAX_ACTIVE_SESSIONS || self.consumed.contains_key(&session_id) {
            return Err(());
        }
        validate_signed_transfer(
            &self.kernel_instance_id,
            self.blob_runtime_generation,
            &self.key,
            &grant,
            binding,
            now_unix_ms,
        )?;
        let verified = decode_transfer(&grant)?;
        self.consumed.insert(session_id, grant.expires_at_unix_ms);
        Ok(verified)
    }

    fn prune(&mut self, now_unix_ms: u64) {
        self.consumed.retain(|_, expiry| *expiry > now_unix_ms);
    }
}

fn validate_signed_transfer(
    instance_id: &str,
    blob_runtime_generation: u64,
    key: &VerifyingKey,
    grant: &BlobCustodyTransferGrantV1,
    binding: &[u8],
    now: u64,
) -> Result<(), ()> {
    if grant.major != 1
        || grant.kernel_instance_id != instance_id
        || grant.blob_runtime_generation != blob_runtime_generation
        || grant.session_id.len() != 16
        || grant.session_id.iter().all(|byte| *byte == 0)
        || grant.channel_binding_sha256.len() != 32
        || binding.len() != 32
        || grant.evidence_id.len() != 16
        || grant.evidence_id.iter().all(|byte| *byte == 0)
        || grant.evidence_envelope_sha256.len() != 32
        || grant.expires_at_unix_ms <= now
        || grant.expires_at_unix_ms > now.checked_add(SESSION_TTL_LIMIT_MS).ok_or(())?
        || grant.kernel_authorization_signature_raw.len() != 64
    {
        return Err(());
    }
    if Sha256::digest(binding).as_slice() != grant.channel_binding_sha256.as_slice() {
        return Err(());
    }
    let signature =
        Signature::from_slice(&grant.kernel_authorization_signature_raw).map_err(|_| ())?;
    let mut unsigned = grant.clone();
    unsigned.kernel_authorization_signature_raw.clear();
    let mut message = b"hermes.blob-custody-transfer.v1\0".to_vec();
    message.extend_from_slice(&unsigned.encode_to_vec());
    key.verify(&message, &signature).map_err(|_| ())
}

fn validate_signed_grant(
    instance_id: &str,
    blob_runtime_generation: u64,
    key: &VerifyingKey,
    grant: &BlobDataSessionGrantV1,
    binding: &[u8],
    expected: BlobDataOperationV1,
    now: u64,
) -> Result<(), ()> {
    if grant.major != 1 { return Err(denied("major")); }
    if grant.kernel_instance_id != instance_id { return Err(denied("kernel_instance")); }
    if grant.blob_runtime_generation != blob_runtime_generation { return Err(denied("runtime_generation")); }
    if grant.session_id.len() != 16 || grant.session_id.iter().all(|byte| *byte == 0) { return Err(denied("session_id")); }
    if grant.channel_binding_sha256.len() != 32 || binding.len() != 32 { return Err(denied("binding_shape")); }
    if grant.expires_at_unix_ms <= now { return Err(denied("expired")); }
    if grant.expires_at_unix_ms > now.checked_add(SESSION_TTL_LIMIT_MS).ok_or(())? { return Err(denied("ttl")); }
    if BlobDataOperationV1::try_from(grant.operation).ok() != Some(expected) { return Err(denied("operation")); }
    if grant.kernel_authorization_signature_raw.len() != 64 { return Err(denied("signature_shape")); }
    if Sha256::digest(binding).as_slice() != grant.channel_binding_sha256.as_slice() {
        return Err(denied("binding"));
    }
    let signature =
        Signature::from_slice(&grant.kernel_authorization_signature_raw)
            .map_err(|_| denied("signature_encoding"))?;
    let mut unsigned = grant.clone();
    unsigned.kernel_authorization_signature_raw.clear();
    let mut message = b"hermes.blob-data-session.v1\0".to_vec();
    message.extend_from_slice(&unsigned.encode_to_vec());
    key.verify(&message, &signature)
        .map_err(|_| denied("signature"))
}

fn denied(stage: &str) {
    if std::env::var_os("HERMES_DEVELOPER_VERBOSE").is_some() {
        eprintln!("developer_blob_session_denied stage={stage}");
    }
}

fn decode_grant(grant: &BlobDataSessionGrantV1) -> Result<VerifiedBlobDataSessionV1, ()> {
    let reference_id = grant.reference_id.as_slice().try_into().map_err(|_| denied("reference_id"))?;
    let backup_class = match WireBackupClass::try_from(grant.backup_class).map_err(|_| denied("backup_class"))? {
        WireBackupClass::BlobBackupClassRequiredV1 => BlobBackupClassV1::Required,
        WireBackupClass::BlobBackupClassRebuildableV1 => BlobBackupClassV1::Rebuildable,
        WireBackupClass::BlobBackupClassExcludedV1 => BlobBackupClassV1::Excluded,
        WireBackupClass::BlobBackupClassUnspecifiedV1 => return Err(denied("backup_class")),
    };
    let reference = BlobRefV1::new(
        reference_id,
        grant.owner_id.clone(),
        grant.declared_size,
        (grant.reference_expires_at_unix_ms != 0).then_some(grant.reference_expires_at_unix_ms),
        backup_class,
    )
    .map_err(|_| denied("reference"))?;
    let access = BlobAccessFenceV1::new(
        grant.owner_id.clone(),
        grant.registration_id.clone(),
        grant.capability_id.clone(),
        grant.runtime_instance_id.clone(),
        grant.runtime_generation,
        grant.grant_epoch,
    )
    .map_err(|_| denied("access_fence"))?;
    let quota = BlobQuotaGrantV1::new(
        grant.owner_id.clone(),
        grant.registration_id.clone(),
        grant.capability_id.clone(),
        grant.grant_epoch,
        grant.quota_max_bytes,
    )
    .map_err(|_| denied("quota"))?;
    if !quota.matches(&access) || grant.key_revision == 0 {
        return Err(denied("quota_match"));
    }
    Ok(VerifiedBlobDataSessionV1 {
        reference,
        access,
        quota,
        key_revision: grant.key_revision,
    })
}

fn decode_transfer(
    grant: &BlobCustodyTransferGrantV1,
) -> Result<VerifiedBlobCustodyTransferV1, ()> {
    let source = grant.source.as_ref().ok_or(())?;
    let backup_class = backup_class(source.backup_class)?;
    let source_reference = BlobRefV1::new(
        source.reference_id.as_slice().try_into().map_err(|_| ())?,
        source.owner_id.clone(),
        source.declared_size,
        (source.reference_expires_at_unix_ms != 0).then_some(source.reference_expires_at_unix_ms),
        backup_class,
    )
    .map_err(|_| ())?;
    let source_access = BlobAccessFenceV1::new(
        source.owner_id.clone(),
        source.registration_id.clone(),
        source.capability_id.clone(),
        source.runtime_instance_id.clone(),
        source.runtime_generation,
        source.grant_epoch,
    )
    .map_err(|_| ())?;
    let target_reference = BlobRefV1::new(
        grant
            .target_reference_id
            .as_slice()
            .try_into()
            .map_err(|_| ())?,
        grant.target_owner_id.clone(),
        source.declared_size,
        (source.reference_expires_at_unix_ms != 0).then_some(source.reference_expires_at_unix_ms),
        backup_class,
    )
    .map_err(|_| ())?;
    let target_access = BlobAccessFenceV1::new(
        grant.target_owner_id.clone(),
        grant.target_registration_id.clone(),
        grant.target_capability_id.clone(),
        grant.target_runtime_instance_id.clone(),
        grant.target_runtime_generation,
        grant.target_grant_epoch,
    )
    .map_err(|_| ())?;
    let target_quota = BlobQuotaGrantV1::new(
        grant.target_owner_id.clone(),
        grant.target_registration_id.clone(),
        grant.target_capability_id.clone(),
        grant.target_grant_epoch,
        grant.target_quota_max_bytes,
    )
    .map_err(|_| ())?;
    if source.owner_id != target_reference.owner_id()
        || !target_quota.matches(&target_access)
        || source.key_revision == 0
        || grant.target_key_revision == 0
    {
        return Err(());
    }
    Ok(VerifiedBlobCustodyTransferV1 {
        source_reference,
        source_access,
        source_key_revision: source.key_revision,
        target_reference,
        target_access,
        target_quota,
        target_key_revision: grant.target_key_revision,
        expected_plaintext_sha256: source
            .receipt_sha256
            .as_slice()
            .try_into()
            .map_err(|_| ())?,
    })
}

fn backup_class(value: i32) -> Result<BlobBackupClassV1, ()> {
    match WireBackupClass::try_from(value).map_err(|_| ())? {
        WireBackupClass::BlobBackupClassRequiredV1 => Ok(BlobBackupClassV1::Required),
        WireBackupClass::BlobBackupClassRebuildableV1 => Ok(BlobBackupClassV1::Rebuildable),
        WireBackupClass::BlobBackupClassExcludedV1 => Ok(BlobBackupClassV1::Excluded),
        WireBackupClass::BlobBackupClassUnspecifiedV1 => Err(()),
    }
}
