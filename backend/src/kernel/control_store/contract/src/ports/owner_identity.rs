use crate::{
    BrowserDeviceEnrollmentV1, BrowserDeviceIdentityV1, ControlStore, InitialOwnerIdentity,
    ServerBootstrapPairing,
};

pub trait OwnerIdentityStore {
    type Error;

    fn initial_owner_identity(&self) -> Result<Option<InitialOwnerIdentity>, Self::Error>;
    fn claim_initial_owner(&self, identity: &InitialOwnerIdentity) -> Result<(), Self::Error>;
    fn current_identity_epoch(&self) -> Result<u64, Self::Error>;
    fn browser_device_identity(
        &self,
        device_id: &str,
    ) -> Result<Option<BrowserDeviceIdentityV1>, Self::Error>;
    fn browser_device_identity_by_credential_id(
        &self,
        credential_id: &[u8],
    ) -> Result<Option<BrowserDeviceIdentityV1>, Self::Error>;
    fn admit_browser_device(
        &self,
        enrollment: &BrowserDeviceEnrollmentV1,
        expected_identity_epoch: u64,
    ) -> Result<BrowserDeviceIdentityV1, Self::Error>;
    fn record_verified_browser_assertion(
        &self,
        credential_id: &[u8],
        observed_sign_count: u32,
        observed_backup_eligible: bool,
        observed_backup_state: bool,
        expected_identity_epoch: u64,
    ) -> Result<BrowserDeviceIdentityV1, Self::Error>;
    fn revoke_browser_device(
        &self,
        device_id: &str,
        expected_identity_epoch: u64,
    ) -> Result<ControlStore, Self::Error>;
    fn begin_server_bootstrap_pairing(
        &self,
        pairing: &ServerBootstrapPairing,
        now_unix_ms: u64,
    ) -> Result<(), Self::Error>;
    fn claim_initial_owner_from_server_bootstrap_pairing(
        &self,
        identity: &InitialOwnerIdentity,
        presented_token_sha256: &[u8; 32],
        now_unix_ms: u64,
    ) -> Result<(), Self::Error>;
}
