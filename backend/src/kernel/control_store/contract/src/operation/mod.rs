//! Idempotent owner-control operation journal contracts.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OperationIdV1([u8; 16]);

impl OperationIdV1 {
    #[must_use]
    pub const fn new(value: [u8; 16]) -> Self {
        Self(value)
    }
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperationTerminalOutcomeV1 {
    Succeeded { response_digest: [u8; 32] },
    Rejected { code: String },
    Failed { code: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperationStatusV1 {
    Admitted,
    Terminal(OperationTerminalOutcomeV1),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperationAdmissionV1 {
    Admitted,
    Duplicate(OperationStatusV1),
}
