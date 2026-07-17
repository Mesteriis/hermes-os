use crate::{InitialOwnerIdentity, ServerBootstrapPairing};

pub trait OwnerIdentityStore {
    type Error;

    fn initial_owner_identity(&self) -> Result<Option<InitialOwnerIdentity>, Self::Error>;
    fn claim_initial_owner(&self, identity: &InitialOwnerIdentity) -> Result<(), Self::Error>;
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
