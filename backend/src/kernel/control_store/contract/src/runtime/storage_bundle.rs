//! Immutable canonical owner-local Storage migration bundle retained by Kernel.

use sha2::{Digest, Sha256};

const MAX_STORAGE_BUNDLE_BYTES: usize = 4 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlatformStorageBundleV1 {
    owner_id: String,
    revision: u64,
    digest: [u8; 32],
    canonical_bytes: Vec<u8>,
}

impl PlatformStorageBundleV1 {
    pub fn new(
        owner_id: impl Into<String>,
        revision: u64,
        digest: [u8; 32],
        canonical_bytes: Vec<u8>,
    ) -> Result<Self, ()> {
        let owner_id = owner_id.into();
        (valid_owner_id(&owner_id)
            && revision > 0
            && digest != [0; 32]
            && !canonical_bytes.is_empty()
            && canonical_bytes.len() <= MAX_STORAGE_BUNDLE_BYTES
            && Sha256::digest(&canonical_bytes).as_slice() == digest)
            .then_some(Self {
                owner_id,
                revision,
                digest,
                canonical_bytes,
            })
            .ok_or(())
    }

    pub fn owner_id(&self) -> &str {
        &self.owner_id
    }
    pub const fn revision(&self) -> u64 {
        self.revision
    }
    pub const fn digest(&self) -> &[u8; 32] {
        &self.digest
    }
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }
}

fn valid_owner_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 96
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}
