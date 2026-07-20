//! Exact Storage namespace request retained from one registered module descriptor.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleStorageRequestV1 {
    registration_id: String,
    capability_id: String,
    owner_id: String,
    connection_budget: u16,
    statement_timeout_millis: u32,
}

impl ModuleStorageRequestV1 {
    #[must_use]
    pub fn new(
        registration_id: impl Into<String>,
        capability_id: impl Into<String>,
        owner_id: impl Into<String>,
        connection_budget: u16,
        statement_timeout_millis: u32,
    ) -> Self {
        Self {
            registration_id: registration_id.into(),
            capability_id: capability_id.into(),
            owner_id: owner_id.into(),
            connection_budget,
            statement_timeout_millis,
        }
    }

    #[must_use]
    pub fn registration_id(&self) -> &str {
        &self.registration_id
    }

    #[must_use]
    pub fn capability_id(&self) -> &str {
        &self.capability_id
    }

    #[must_use]
    pub fn owner_id(&self) -> &str {
        &self.owner_id
    }

    #[must_use]
    pub const fn connection_budget(&self) -> u16 {
        self.connection_budget
    }

    #[must_use]
    pub const fn statement_timeout_millis(&self) -> u32 {
        self.statement_timeout_millis
    }
}
