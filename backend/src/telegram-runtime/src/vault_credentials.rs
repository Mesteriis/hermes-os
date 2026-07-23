//! Telegram admission glue over the shared ciphertext-only Vault route.

use std::os::unix::net::UnixStream;

use hermes_storage_protocol::StorageBindingV1;
use hermes_storage_vault::{
    InheritedKernelVaultRouteV1, StorageCredentialLeaseErrorV1, StorageVaultLeaseAdapterV1,
    StorageVaultRouteContextV1,
};
use zeroize::Zeroizing;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TelegramCredentialRouteError {
    InvalidContext,
    InvalidLease,
    Unavailable,
    Rejected,
}

pub async fn resolve_storage_credential(
    channel: UnixStream,
    binding: &StorageBindingV1,
    context: StorageVaultRouteContextV1,
) -> Result<Zeroizing<Vec<u8>>, StorageCredentialLeaseErrorV1> {
    let mut leases = StorageVaultLeaseAdapterV1::new(InheritedKernelVaultRouteV1::new(channel), context);
    let lease_id = leases.issue_runtime_credential(binding).await?;
    leases.resolve_runtime_credential(binding, lease_id).await
}
