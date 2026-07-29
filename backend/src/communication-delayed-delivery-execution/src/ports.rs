#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DelayedDeliveryBodyReceiptV1 {
    pub reference_id: [u8; 16],
    pub declared_bytes: u64,
    pub sha256: [u8; 32],
    pub custody_proof: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DelayedDeliveryDurableMessageV1 {
    pub message_id: [u8; 16],
    pub contract_kind: &'static str,
    pub envelope_sha256: [u8; 32],
    pub envelope_bytes: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SchedulerExecutionFenceV1 {
    pub run_id: [u8; 16],
    pub schedule_revision: u64,
    pub lease_epoch: u64,
    pub lease_expires_at_unix_millis: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DelayedDeliveryExecutionClaimV1 {
    pub logical_owner_id: String,
    pub delayed_operation_id: [u8; 16],
    pub delivery_operation_id: [u8; 16],
    pub conversation_id: [u8; 16],
    pub reply_to_message_id: Option<[u8; 16]>,
    pub body_receipt: DelayedDeliveryBodyReceiptV1,
    pub fence: SchedulerExecutionFenceV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClaimDueExecutionV1 {
    pub logical_owner_id: String,
    pub delayed_operation_id: [u8; 16],
    pub command_message_id: [u8; 16],
    pub command_envelope_sha256: [u8; 32],
    pub fence: SchedulerExecutionFenceV1,
    pub acceptance_receipt: DelayedDeliveryDurableMessageV1,
    pub claimed_at_unix_millis: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClaimDueExecutionOutcomeV1 {
    Claimed(DelayedDeliveryExecutionClaimV1),
    Duplicate(DelayedDeliveryExecutionClaimV1),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MarkDeliveryAcceptedV1 {
    pub claim: DelayedDeliveryExecutionClaimV1,
    pub terminal_receipt: DelayedDeliveryDurableMessageV1,
    pub accepted_at_unix_millis: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MarkDeliveryFailedV1 {
    pub claim: DelayedDeliveryExecutionClaimV1,
    pub error_code: u16,
    pub terminal_receipt: DelayedDeliveryDurableMessageV1,
    pub failed_at_unix_millis: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExecutionStoreErrorV1 {
    InvalidInput,
    Unavailable,
    Conflict,
    ClaimLost,
    NotFound,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BodyReadErrorV1 {
    Denied,
    Unavailable,
    InvalidReceipt,
}

#[allow(async_fn_in_trait)]
pub trait BodyReadPortV1 {
    async fn read_once(
        &mut self,
        claim: &DelayedDeliveryExecutionClaimV1,
    ) -> Result<Vec<u8>, BodyReadErrorV1>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BodyCleanupErrorV1 {
    Unavailable,
}

#[allow(async_fn_in_trait)]
pub trait BodyCleanupPortV1 {
    async fn request_cleanup(
        &mut self,
        claim: &DelayedDeliveryExecutionClaimV1,
    ) -> Result<(), BodyCleanupErrorV1>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchedulerTerminalOutcomeV1 {
    Succeeded,
    Failed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchedulerReceiptErrorV1 {
    Unavailable,
    InvalidEnvelope,
}

pub trait SchedulerReceiptFactoryPortV1 {
    fn terminal_receipt(
        &mut self,
        claim: &DelayedDeliveryExecutionClaimV1,
        outcome: SchedulerTerminalOutcomeV1,
        observed_at_unix_millis: u64,
    ) -> Result<DelayedDeliveryDurableMessageV1, SchedulerReceiptErrorV1>;
}

#[allow(async_fn_in_trait)]
pub trait ExecutionStorePortV1 {
    async fn claim_due(
        &mut self,
        command: &ClaimDueExecutionV1,
    ) -> Result<ClaimDueExecutionOutcomeV1, ExecutionStoreErrorV1>;

    async fn mark_accepted(
        &mut self,
        command: &MarkDeliveryAcceptedV1,
    ) -> Result<(), ExecutionStoreErrorV1>;

    async fn mark_failed(
        &mut self,
        command: &MarkDeliveryFailedV1,
    ) -> Result<(), ExecutionStoreErrorV1>;
}

pub(crate) fn receipt_matches_body(receipt: &DelayedDeliveryBodyReceiptV1, body: &[u8]) -> bool {
    u64::try_from(body.len()) == Ok(receipt.declared_bytes)
}
