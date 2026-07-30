#![forbid(unsafe_code)]

use std::os::unix::net::UnixStream;

use hermes_blob_client::{
    BlobClientError, BlobDataClient, ManagedBlobCustodyReleaseRequestV1,
    ManagedBlobSessionRequestV1, request_managed_blob_custody_release_v2,
    request_managed_blob_session_v2,
};
use hermes_communication_delayed_delivery_execution::{
    BodyCleanupErrorV1, BodyCleanupPortV1, BodyCleanupReasonV1, BodyReadErrorV1, BodyReadPortV1,
    DelayedDeliveryExecutionClaimV1, DeliveryIntentRequestErrorV1, DeliveryIntentRequestPortV1,
};
use hermes_communication_delivery_intent_api::{
    COMMUNICATION_DELIVERY_INTENT_COMMAND_CONTRACT_NAME_V1,
    COMMUNICATION_DELIVERY_INTENT_CONTRACT_MAJOR_V1,
    COMMUNICATION_DELIVERY_INTENT_CONTRACT_REVISION_V1, COMMUNICATION_DELIVERY_INTENT_OWNER_V1,
    COMMUNICATION_DELIVERY_INTENT_SCHEMA_SHA256,
};
use hermes_runtime_protocol::{
    managed_control::{ManagedControlChannelV2, ManagedControlRequestDispatcherV2},
    v1::{
        BlobCustodyReleaseReasonV1, BlobDataOperationV1, ContractReferenceV1,
        ManagedRuntimeControlRequestV1, ManagedRuntimeModuleRequestRequestV1,
        managed_runtime_control_request_v1::Operation,
        managed_runtime_control_response_v1::Result as ControlResult,
    },
    validation::module_request::{
        validate_module_request_request_v1, validate_module_request_response_v1,
    },
};

const REQUIRED_BACKUP_CLASS_V1: u32 = 1;
const DELIVERY_REQUEST_DEADLINE_MILLIS_V1: u32 = 30_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ManagedDelayedDeliveryRuntimePortErrorV1 {
    InvalidCapability,
}

pub struct ManagedDelayedDeliveryRuntimePortV1<'a> {
    channel: &'a mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &'a mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    blob_capability_id: &'a str,
}

impl<'a> ManagedDelayedDeliveryRuntimePortV1<'a> {
    pub fn new(
        channel: &'a mut ManagedControlChannelV2<UnixStream>,
        dispatcher: &'a mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
        blob_capability_id: &'a str,
    ) -> Result<Self, ManagedDelayedDeliveryRuntimePortErrorV1> {
        if blob_capability_id.trim().is_empty()
            || blob_capability_id.len() > 128
            || !blob_capability_id
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
        {
            return Err(ManagedDelayedDeliveryRuntimePortErrorV1::InvalidCapability);
        }
        Ok(Self {
            channel,
            dispatcher,
            blob_capability_id,
        })
    }
}

impl BodyReadPortV1 for ManagedDelayedDeliveryRuntimePortV1<'_> {
    async fn read_once(
        &mut self,
        claim: &DelayedDeliveryExecutionClaimV1,
    ) -> Result<Vec<u8>, BodyReadErrorV1> {
        let session = request_managed_blob_session_v2(
            self.channel,
            self.dispatcher,
            ManagedBlobSessionRequestV1 {
                capability_id: self.blob_capability_id,
                operation: BlobDataOperationV1::BlobDataOperationReadRangeV1,
                reference_id: &claim.body_receipt.reference_id,
                declared_size: claim.body_receipt.declared_bytes,
                backup_class: REQUIRED_BACKUP_CLASS_V1,
                receipt_sha256: Some(&claim.body_receipt.sha256),
                custody_target: None,
            },
        )
        .map_err(body_read_error)?;
        BlobDataClient::new(session.data_socket_path)
            .and_then(|client| {
                client.read_range(
                    session.grant,
                    session.channel_binding,
                    0,
                    claim.body_receipt.declared_bytes,
                )
            })
            .map_err(body_read_error)
    }
}

impl DeliveryIntentRequestPortV1 for ManagedDelayedDeliveryRuntimePortV1<'_> {
    async fn request(
        &mut self,
        request_id: [u8; 16],
        payload: Vec<u8>,
    ) -> Result<Vec<u8>, DeliveryIntentRequestErrorV1> {
        let request = ManagedRuntimeModuleRequestRequestV1 {
            request_id: request_id.to_vec(),
            contract: Some(delivery_intent_command_contract_v1()),
            request_payload: payload,
            deadline_millis: DELIVERY_REQUEST_DEADLINE_MILLIS_V1,
        };
        validate_module_request_request_v1(&request)
            .map_err(|_| DeliveryIntentRequestErrorV1::Protocol)?;
        let response = self
            .channel
            .request_next_with_dispatch(
                ManagedRuntimeControlRequestV1 {
                    operation: Some(Operation::RouteModuleRequest(request)),
                },
                self.dispatcher,
            )
            .map_err(|_| DeliveryIntentRequestErrorV1::Unavailable)?;
        if !response.error_code.is_empty() {
            return Err(DeliveryIntentRequestErrorV1::Unavailable);
        }
        let Some(ControlResult::ModuleRequestRoute(response)) = response.result else {
            return Err(DeliveryIntentRequestErrorV1::Protocol);
        };
        validate_module_request_response_v1(&response)
            .map_err(|_| DeliveryIntentRequestErrorV1::Protocol)?;
        if response.request_id != request_id {
            return Err(DeliveryIntentRequestErrorV1::Protocol);
        }
        if !response.error_code.is_empty() {
            return Err(DeliveryIntentRequestErrorV1::Unavailable);
        }
        Ok(response.response_payload)
    }
}

impl BodyCleanupPortV1 for ManagedDelayedDeliveryRuntimePortV1<'_> {
    async fn request_cleanup(
        &mut self,
        claim: &DelayedDeliveryExecutionClaimV1,
        reason: BodyCleanupReasonV1,
    ) -> Result<(), BodyCleanupErrorV1> {
        let reason = match reason {
            BodyCleanupReasonV1::DeliveryAccepted => {
                BlobCustodyReleaseReasonV1::BlobCustodyReleaseReasonTerminalAcceptedV1
            }
            BodyCleanupReasonV1::DeliveryRejected => {
                BlobCustodyReleaseReasonV1::BlobCustodyReleaseReasonTerminalRejectedV1
            }
        };
        request_managed_blob_custody_release_v2(
            self.channel,
            self.dispatcher,
            ManagedBlobCustodyReleaseRequestV1 {
                operation_id: &claim.delayed_operation_id,
                capability_id: self.blob_capability_id,
                reference_id: &claim.body_receipt.reference_id,
                declared_size: claim.body_receipt.declared_bytes,
                receipt_sha256: &claim.body_receipt.sha256,
                custody_source_proof: &claim.body_receipt.custody_proof,
                reason,
            },
        )
        .map(|_| ())
        .map_err(|_| BodyCleanupErrorV1::Unavailable)
    }
}

fn delivery_intent_command_contract_v1() -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: COMMUNICATION_DELIVERY_INTENT_OWNER_V1.to_owned(),
        name: COMMUNICATION_DELIVERY_INTENT_COMMAND_CONTRACT_NAME_V1.to_owned(),
        major: COMMUNICATION_DELIVERY_INTENT_CONTRACT_MAJOR_V1,
        revision: COMMUNICATION_DELIVERY_INTENT_CONTRACT_REVISION_V1,
        schema_sha256: COMMUNICATION_DELIVERY_INTENT_SCHEMA_SHA256.to_vec(),
    }
}

fn body_read_error(error: BlobClientError) -> BodyReadErrorV1 {
    match error {
        BlobClientError::InvalidSessionRequest
        | BlobClientError::InvalidCustodyReleaseRequest
        | BlobClientError::InvalidSocketPath
        | BlobClientError::InvalidTimeout => BodyReadErrorV1::InvalidReceipt,
        BlobClientError::Rejected(_)
        | BlobClientError::InvalidFrame
        | BlobClientError::InvalidResponse
        | BlobClientError::FrameTooLarge => BodyReadErrorV1::Denied,
        BlobClientError::Connect(_) | BlobClientError::Io(_) | BlobClientError::Unavailable => {
            BodyReadErrorV1::Unavailable
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delivery_contract_is_exact_and_provider_neutral() {
        let contract = delivery_intent_command_contract_v1();
        assert_eq!(contract.owner, "communication_delivery_intent");
        assert_eq!(contract.name, "communication.delivery_intent.command");
        assert_eq!(contract.schema_sha256.len(), 32);
    }

    #[test]
    fn invalid_blob_inputs_never_become_retryable_transport_failures() {
        assert_eq!(
            body_read_error(BlobClientError::InvalidSessionRequest),
            BodyReadErrorV1::InvalidReceipt
        );
        assert_eq!(
            body_read_error(BlobClientError::Unavailable),
            BodyReadErrorV1::Unavailable
        );
    }
}

pub const PACKAGE: &str = "hermes-communication-delayed-delivery-runtime-adapters";
