// Owner-approved browser enrollment contract.
use crate::{
    BrowserDevicePrincipalV1, BrowserPairingAuthority, GatewayIdentityFenceV1, valid_id,
    valid_rp_id,
};

/// A browser credential accepted only from a current, owner-approved pairing.
///
/// The Gateway session package constructs this after WebAuthn verification. The
/// authority persists it atomically against the captured identity fence; it
/// must not treat any caller-selected device or credential values as trusted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserEnrollmentV1 {
    owner_id: String,
    device_id: String,
    rp_id: String,
    credential_id: Vec<u8>,
    cose_public_key: Vec<u8>,
    browser_key_public_key: Vec<u8>,
    sign_count: u32,
    backup_eligible: bool,
    backup_state: bool,
    identity_fence: GatewayIdentityFenceV1,
}

impl BrowserEnrollmentV1 {
    pub fn new(
        owner_id: impl Into<String>,
        device_id: impl Into<String>,
        rp_id: impl Into<String>,
        credential_id: Vec<u8>,
        cose_public_key: Vec<u8>,
        browser_key_public_key: Vec<u8>,
        sign_count: u32,
        backup_eligible: bool,
        backup_state: bool,
        identity_fence: GatewayIdentityFenceV1,
    ) -> Result<Self, String> {
        let owner_id = owner_id.into();
        let device_id = device_id.into();
        let rp_id = rp_id.into();
        (valid_id(&owner_id)
            && valid_id(&device_id)
            && valid_rp_id(&rp_id)
            && !credential_id.is_empty()
            && credential_id.len() <= 1024
            && (16..=1024).contains(&cose_public_key.len())
            && valid_browser_key_public_key(&browser_key_public_key)
            && (!backup_state || backup_eligible))
            .then_some(Self {
                owner_id,
                device_id,
                rp_id,
                credential_id,
                cose_public_key,
                browser_key_public_key,
                sign_count,
                backup_eligible,
                backup_state,
                identity_fence,
            })
            .ok_or_else(|| "browser enrollment is invalid".to_owned())
    }
    #[must_use]
    pub fn owner_id(&self) -> &str {
        &self.owner_id
    }
    #[must_use]
    pub fn device_id(&self) -> &str {
        &self.device_id
    }
    #[must_use]
    pub fn rp_id(&self) -> &str {
        &self.rp_id
    }
    #[must_use]
    pub fn credential_id(&self) -> &[u8] {
        &self.credential_id
    }
    #[must_use]
    pub fn cose_public_key(&self) -> &[u8] {
        &self.cose_public_key
    }
    #[must_use]
    pub fn browser_key_public_key(&self) -> &[u8] {
        &self.browser_key_public_key
    }
    #[must_use]
    pub const fn sign_count(&self) -> u32 {
        self.sign_count
    }
    #[must_use]
    pub const fn backup_eligible(&self) -> bool {
        self.backup_eligible
    }
    #[must_use]
    pub const fn backup_state(&self) -> bool {
        self.backup_state
    }
    #[must_use]
    pub const fn identity_fence(&self) -> &GatewayIdentityFenceV1 {
        &self.identity_fence
    }
}

fn valid_browser_key_public_key(value: &[u8]) -> bool {
    value.len() == 65 && value.first() == Some(&4)
}

pub trait BrowserEnrollmentAuthority: BrowserPairingAuthority {
    fn admit_browser_device(
        &self,
        enrollment: &BrowserEnrollmentV1,
    ) -> Result<BrowserDevicePrincipalV1, String>;
}
