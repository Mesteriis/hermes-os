//! Detached P-256 signature binding for a recovery-media manifest.

use p256::ecdsa::signature::Verifier;
use p256::ecdsa::{Signature, VerifyingKey};

use super::format::RecoveryMediaManifestV1;

const SIGNATURE_MAGIC: &[u8; 8] = b"HRSIGV1\0";
const MAX_KEY_ID_BYTES: usize = 128;

pub(crate) struct SignedRecoveryMediaManifestV1 {
    verification_key_id: String,
    raw_manifest_bytes: Vec<u8>,
    signature_raw: [u8; 64],
}

impl SignedRecoveryMediaManifestV1 {
    pub(crate) fn new(
        verification_key_id: String,
        raw_manifest_bytes: Vec<u8>,
        signature_raw: [u8; 64],
    ) -> Result<Self, String> {
        if verification_key_id.is_empty()
            || verification_key_id.len() > MAX_KEY_ID_BYTES
            || raw_manifest_bytes.is_empty()
        {
            return Err(invalid_signature());
        }
        Ok(Self {
            verification_key_id,
            raw_manifest_bytes,
            signature_raw,
        })
    }

    pub(crate) fn decode(
        raw_manifest_bytes: Vec<u8>,
        signature_metadata: &[u8],
    ) -> Result<Self, String> {
        if signature_metadata.len() < SIGNATURE_MAGIC.len() + 1 + 64
            || &signature_metadata[..SIGNATURE_MAGIC.len()] != SIGNATURE_MAGIC
        {
            return Err(invalid_signature());
        }
        let key_length = usize::from(signature_metadata[SIGNATURE_MAGIC.len()]);
        let key_start = SIGNATURE_MAGIC.len() + 1;
        let signature_start = key_start
            .checked_add(key_length)
            .ok_or_else(invalid_signature)?;
        if key_length == 0
            || key_length > MAX_KEY_ID_BYTES
            || signature_metadata.len() != signature_start + 64
        {
            return Err(invalid_signature());
        }
        let key_id = std::str::from_utf8(&signature_metadata[key_start..signature_start])
            .map_err(|_| invalid_signature())?
            .to_owned();
        Self::new(
            key_id,
            raw_manifest_bytes,
            signature_metadata[signature_start..]
                .try_into()
                .map_err(|_| invalid_signature())?,
        )
    }

    pub(crate) fn signature_metadata(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(SIGNATURE_MAGIC.len() + 1 + 128 + 64);
        bytes.extend_from_slice(SIGNATURE_MAGIC);
        bytes.push(self.verification_key_id.len() as u8);
        bytes.extend_from_slice(self.verification_key_id.as_bytes());
        bytes.extend_from_slice(&self.signature_raw);
        bytes
    }

    pub(crate) fn raw_manifest_bytes(&self) -> &[u8] {
        &self.raw_manifest_bytes
    }

    pub(crate) fn verify_and_decode(
        &self,
        expected_key_id: &str,
        public_key_sec1: &[u8],
    ) -> Result<RecoveryMediaManifestV1, String> {
        if self.verification_key_id != expected_key_id {
            return Err("recovery media verification key is not pinned".to_owned());
        }
        let key = VerifyingKey::from_sec1_bytes(public_key_sec1)
            .map_err(|_| "recovery media verification key is invalid".to_owned())?;
        let signature = Signature::from_slice(&self.signature_raw)
            .map_err(|_| "recovery media signature is invalid".to_owned())?;
        key.verify(&self.raw_manifest_bytes, &signature)
            .map_err(|_| "recovery media signature verification failed".to_owned())?;
        RecoveryMediaManifestV1::decode(&self.raw_manifest_bytes)
    }
}

fn invalid_signature() -> String {
    "signed recovery media manifest is invalid".to_owned()
}
