use crate::{GrantSet, ModuleRegistration};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleGrantSnapshot {
    registration: ModuleRegistration,
    effective_grants: Option<GrantSet>,
}

impl ModuleGrantSnapshot {
    #[must_use]
    pub fn new(registration: ModuleRegistration, effective_grants: Option<GrantSet>) -> Self {
        Self {
            registration,
            effective_grants,
        }
    }

    #[must_use]
    pub fn registration(&self) -> &ModuleRegistration {
        &self.registration
    }

    #[must_use]
    pub fn effective_grants(&self) -> Option<&GrantSet> {
        self.effective_grants.as_ref()
    }

    #[must_use]
    pub fn into_parts(self) -> (ModuleRegistration, Option<GrantSet>) {
        (self.registration, self.effective_grants)
    }
}
