//! Persistence placeholder for ADR-0239 communications domain.

pub const PACKAGE: &str = "hermes-communications-persistence";

use std::collections::HashSet;

use hermes_communications_domain::CommunicationSummary;

#[derive(Default)]
pub struct CommunicationsPersistence {
    ids: HashSet<String>,
}

impl CommunicationsPersistence {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn has(&self, operation_id: &str) -> bool {
        self.ids.contains(operation_id)
    }

    pub fn persist(
        &mut self,
        summary: &CommunicationSummary,
    ) -> Result<(), CommunicationsPersistenceError> {
        if self.ids.contains(&summary.operation_id) {
            return Err(CommunicationsPersistenceError::DuplicateOperation);
        }
        self.ids.insert(summary.operation_id.clone());
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommunicationsPersistenceError {
    DuplicateOperation,
}
