//! Persistence placeholder for ADR-0239 mail slice.

pub const PACKAGE: &str = "hermes-mail-persistence";

use std::collections::HashMap;

use hermes_mail_core::MailStatePolicy;

#[derive(Clone, Debug, PartialEq)]
pub struct PersistedMailConnection {
    pub id: String,
    pub host: String,
    pub port: u16,
    pub username: String,
}

#[derive(Clone, Debug)]
pub struct PersistedMailOperation {
    pub operation_id: String,
    pub window_size: u32,
}

#[derive(Default)]
pub struct MailPersistence {
    connections: HashMap<String, PersistedMailConnection>,
    operations: HashMap<String, PersistedMailOperation>,
    policy: MailStatePolicy,
}

impl MailPersistence {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn policy(&self) -> MailStatePolicy {
        self.policy.clone()
    }

    pub fn put_connection(&mut self, connection: PersistedMailConnection) {
        self.connections.insert(connection.id.clone(), connection);
    }

    pub fn get_connection(&self, connection_id: &str) -> Option<&PersistedMailConnection> {
        self.connections.get(connection_id)
    }

    pub fn put_operation(&mut self, operation: PersistedMailOperation) {
        self.operations
            .insert(operation.operation_id.clone(), operation);
    }

    pub fn operation_count(&self) -> usize {
        self.operations.len()
    }
}
