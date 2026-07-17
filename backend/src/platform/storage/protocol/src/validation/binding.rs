//! Validation for protobuf StorageBindingV1 transport messages.

use crate::{
    StorageBindingAccessV1, StorageBindingErrorV1, StorageBindingFencesV1,
    StorageBindingIdentityV1, StorageEffectiveBudgetsV1, v1::StorageBindingV1,
};

pub fn validate_storage_binding_message(
    binding: &StorageBindingV1,
) -> Result<(), StorageBindingErrorV1> {
    StorageBindingIdentityV1::new(
        binding.storage_instance_id.clone(),
        binding.database_id.clone(),
        binding.owner.clone(),
        binding.registration_id.clone(),
        binding.runtime_instance_id.clone(),
    )?;
    StorageBindingFencesV1::new(
        binding.storage_generation,
        binding.runtime_generation,
        binding.grant_epoch,
        binding.role_epoch,
        binding.credential_lease_revision,
        binding.storage_bundle_revision,
    )?;
    let budgets = binding
        .effective_budgets
        .as_ref()
        .ok_or(StorageBindingErrorV1::Budget)?;
    let budgets = StorageEffectiveBudgetsV1::new(
        u16::try_from(budgets.max_connections).map_err(|_| StorageBindingErrorV1::Budget)?,
        budgets.statement_timeout_millis,
    )?;
    let digest = binding
        .storage_bundle_digest
        .as_slice()
        .try_into()
        .map_err(|_| StorageBindingErrorV1::Digest)?;
    StorageBindingAccessV1::new(
        binding.runtime_principal.clone(),
        binding.pool_alias.clone(),
        budgets,
        digest,
    )?;
    Ok(())
}
