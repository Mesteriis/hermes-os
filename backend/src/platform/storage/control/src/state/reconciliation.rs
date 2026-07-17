//! In-memory desired binding selected by Storage Control.

use hermes_storage_protocol::StorageBindingV1;

use super::StorageReconciliationErrorV1;

#[derive(Default)]
pub struct StorageReconciliationV1 {
    current: Option<StorageBindingV1>,
}

impl StorageReconciliationV1 {
    pub fn accept(
        &mut self,
        binding: StorageBindingV1,
    ) -> Result<(), StorageReconciliationErrorV1> {
        if let Some(current) = self.current.as_ref() {
            ensure_same_runtime(current, &binding)?;
            ensure_newer_generation(current, &binding)?;
        }
        self.current = Some(binding);
        Ok(())
    }

    pub fn binding(&self) -> Option<&StorageBindingV1> {
        self.current.as_ref()
    }
}

fn ensure_same_runtime(
    current: &StorageBindingV1,
    replacement: &StorageBindingV1,
) -> Result<(), StorageReconciliationErrorV1> {
    if current.identity().storage_instance_id() != replacement.identity().storage_instance_id()
        || current.identity().registration_id() != replacement.identity().registration_id()
        || current.identity().runtime_instance_id() != replacement.identity().runtime_instance_id()
    {
        return Err(StorageReconciliationErrorV1::IdentityMismatch);
    }
    Ok(())
}

fn ensure_newer_generation(
    current: &StorageBindingV1,
    replacement: &StorageBindingV1,
) -> Result<(), StorageReconciliationErrorV1> {
    if replacement.fences().storage_generation() <= current.fences().storage_generation() {
        return Err(StorageReconciliationErrorV1::StaleStorageGeneration);
    }
    Ok(())
}
