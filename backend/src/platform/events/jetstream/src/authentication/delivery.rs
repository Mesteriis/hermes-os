//! Event Hub-specific construction of the public runtime credential fence.

use hermes_events_protocol::{
    NatsRuntimeCredentialDeliveryBindingV1, NatsRuntimeCredentialDeliveryErrorV1,
    NatsRuntimeCredentialRecipientPublicKeyV1,
};

use crate::vault::NatsRuntimeCredentialFenceV1;

pub fn bind_runtime_credential_delivery(
    fence: &NatsRuntimeCredentialFenceV1,
    request_id: [u8; 16],
    recipient_public_key: NatsRuntimeCredentialRecipientPublicKeyV1,
) -> Result<NatsRuntimeCredentialDeliveryBindingV1, NatsRuntimeCredentialDeliveryErrorV1> {
    NatsRuntimeCredentialDeliveryBindingV1::new(
        fence.logical_owner_id().to_owned(),
        fence.registration_id().to_owned(),
        fence.runtime_instance_id().to_owned(),
        fence.runtime_generation(),
        fence.grant_epoch(),
        fence.credential_revision(),
        request_id,
        recipient_public_key,
    )
}
