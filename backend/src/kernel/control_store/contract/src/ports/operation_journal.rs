use crate::{OperationAdmissionV1, OperationIdV1, OperationStatusV1, OperationTerminalOutcomeV1};

/// Kernel-private idempotency and terminal-outcome journal.
pub trait OperationJournalStore {
    type Error;
    fn admit_operation(
        &self,
        operation_id: OperationIdV1,
        request_digest: [u8; 32],
        deadline_unix_millis: u64,
    ) -> Result<OperationAdmissionV1, Self::Error>;
    fn complete_operation(
        &self,
        operation_id: OperationIdV1,
        request_digest: [u8; 32],
        outcome: &OperationTerminalOutcomeV1,
    ) -> Result<(), Self::Error>;
    fn operation_status(
        &self,
        operation_id: OperationIdV1,
    ) -> Result<Option<OperationStatusV1>, Self::Error>;
}
