// Browser device identity contract.
use crate::valid_id;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatewayIdentityFenceV1 {
    instance_id: String,
    generation: u64,
    identity_epoch: u64,
}

impl GatewayIdentityFenceV1 {
    pub fn new(
        instance_id: impl Into<String>,
        generation: u64,
        identity_epoch: u64,
    ) -> Result<Self, String> {
        let instance_id = instance_id.into();
        (!instance_id.is_empty() && generation > 0 && identity_epoch > 0)
            .then_some(Self {
                instance_id,
                generation,
                identity_epoch,
            })
            .ok_or_else(|| "gateway identity fence is invalid".to_owned())
    }

    #[must_use]
    pub fn instance_id(&self) -> &str {
        &self.instance_id
    }

    #[must_use]
    pub const fn generation(&self) -> u64 {
        self.generation
    }

    #[must_use]
    pub const fn identity_epoch(&self) -> u64 {
        self.identity_epoch
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserDevicePrincipalV1 {
    owner_id: String,
    device_id: String,
}

/// Public-key material resolved only for an already active browser device.
///
/// This crosses the Kernel-to-Gateway authority boundary solely to construct a
/// WebAuthn verification ceremony. It is neither a browser response nor a
/// reusable bearer credential.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserDeviceCredentialV1 {
    credential_id: Vec<u8>,
    cose_public_key: Vec<u8>,
    browser_key_public_key: Vec<u8>,
    sign_count: u32,
    backup_eligible: bool,
    backup_state: bool,
}

impl BrowserDeviceCredentialV1 {
    pub fn new(
        credential_id: Vec<u8>,
        cose_public_key: Vec<u8>,
        browser_key_public_key: Vec<u8>,
        sign_count: u32,
        backup_eligible: bool,
        backup_state: bool,
    ) -> Result<Self, String> {
        (!credential_id.is_empty()
            && credential_id.len() <= 1024
            && (16..=1024).contains(&cose_public_key.len())
            && valid_browser_key_public_key(&browser_key_public_key)
            && (!backup_state || backup_eligible))
            .then_some(Self {
                credential_id,
                cose_public_key,
                browser_key_public_key,
                sign_count,
                backup_eligible,
                backup_state,
            })
            .ok_or_else(|| "browser device credential is invalid".to_owned())
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
}

fn valid_browser_key_public_key(value: &[u8]) -> bool {
    value.len() == 65 && value.first() == Some(&4)
}

impl BrowserDevicePrincipalV1 {
    pub fn new(owner_id: impl Into<String>, device_id: impl Into<String>) -> Result<Self, String> {
        let owner_id = owner_id.into();
        let device_id = device_id.into();
        (valid_id(&owner_id) && valid_id(&device_id))
            .then_some(Self {
                owner_id,
                device_id,
            })
            .ok_or_else(|| "browser device principal is invalid".to_owned())
    }

    #[must_use]
    pub fn owner_id(&self) -> &str {
        &self.owner_id
    }

    #[must_use]
    pub fn device_id(&self) -> &str {
        &self.device_id
    }
}

pub trait BrowserDeviceAuthority {
    fn current_identity_fence(&self) -> Result<GatewayIdentityFenceV1, String>;
    fn active_browser_device(&self, device_id: &str) -> Result<BrowserDevicePrincipalV1, String>;
    fn active_browser_device_by_credential(
        &self,
        credential_id: &[u8],
    ) -> Result<BrowserDevicePrincipalV1, String>;
}

pub trait BrowserAssertionAuthority: BrowserDeviceAuthority {
    fn accept_verified_browser_assertion(
        &self,
        credential_id: &[u8],
        sign_count: u32,
        backup_eligible: bool,
        backup_state: bool,
    ) -> Result<BrowserDevicePrincipalV1, String>;
}

/// Resolves an active public credential before the Gateway creates an
/// authentication ceremony. The browser-supplied credential ID remains
/// untrusted until this authority proves that it is active.
pub trait BrowserAuthenticationAuthority: BrowserAssertionAuthority {
    fn active_browser_credential(
        &self,
        credential_id: &[u8],
    ) -> Result<BrowserDeviceCredentialV1, String>;
}

pub trait BrowserPairingAuthority {
    fn current_identity_fence(&self) -> Result<GatewayIdentityFenceV1, String>;
    fn require_current_owner(&self, owner_id: &str) -> Result<(), String>;
}
