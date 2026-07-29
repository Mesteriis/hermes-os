//! Kernel-fenced managed process and owner-local Storage bootstrap.

use std::os::unix::net::UnixStream;

use hermes_communication_delivery_intent_core::PlannedDeliveryIntentV1;
use hermes_communication_delivery_intent_persistence::{
    CommunicationDeliveryIntentPersistenceV1, CreateDeliveryIntentOutcomeV1,
    DeliveryIntentPersistenceErrorV1,
};
use hermes_events_jetstream::{
    JetStreamClient, RuntimeJetStreamConnection, RuntimeNatsIdentity, RuntimePublishPermitV1,
    request_managed_runtime_event_access_v2,
};
use hermes_runtime_protocol::{
    managed_control::{ManagedControlChannelV2, ManagedControlRequestDispatcherV2},
    v1::{
        ManagedRuntimeControlResponseV1, ManagedRuntimeReadyRequestV1,
        ManagedStorageRuntimeConfigurationV1,
    },
};
use hermes_storage_protocol::{
    StorageBindingAccessV1, StorageBindingFencesV1, StorageBindingIdentityV1, StorageBindingV1,
    StorageEffectiveBudgetsV1,
};
use hermes_storage_vault::{
    InheritedKernelVaultRouteV2, StorageVaultLeaseAdapterV1, StorageVaultRouteContextV1,
};

use crate::{
    body_materializer::ManagedDeliveryIntentBodyMaterializerV1,
    coordinator::{DeliveryIntentCoordinatorErrorV1, prepare_create_delivery_intent_v1},
    event_runtime::{ProviderTerminalSubscriptionV1, bind_terminal_subscriptions},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeliveryIntentRuntimeAdmissionV1 {
    pub logical_owner_id: String,
    pub registration_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub grant_epoch: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeliveryIntentRuntimeErrorV1 {
    Admission,
    Coordinator(DeliveryIntentCoordinatorErrorV1),
    Persistence(DeliveryIntentPersistenceErrorV1),
    EventContract,
    Unavailable,
}

pub struct DeliveryIntentManagedRuntimeV1 {
    pub(crate) logical_owner_id: String,
    control_channel: ManagedControlChannelV2<UnixStream>,
    persistence: CommunicationDeliveryIntentPersistenceV1,
    pub(crate) runtime_instance_id: String,
    pub(crate) runtime_generation: u64,
    pub(crate) event_connection: RuntimeJetStreamConnection,
    pub(crate) event_publish_permit: RuntimePublishPermitV1,
    pub(crate) terminal_subscriptions: Vec<ProviderTerminalSubscriptionV1>,
    pub(crate) next_terminal_subscription: usize,
}

impl DeliveryIntentManagedRuntimeV1 {
    pub async fn open(
        control_channel: UnixStream,
        descriptor_bytes: Vec<u8>,
        settings_schema_bytes: Vec<u8>,
        admission: &DeliveryIntentRuntimeAdmissionV1,
        storage_configuration: ManagedStorageRuntimeConfigurationV1,
        event_hub_endpoint: &str,
        event_credential_revision: u64,
    ) -> Result<Self, DeliveryIntentRuntimeErrorV1> {
        validate_admission(admission)?;
        if event_hub_endpoint.trim().is_empty() || event_credential_revision == 0 {
            return Err(DeliveryIntentRuntimeErrorV1::Admission);
        }
        let mut control_channel = ManagedControlChannelV2::new(control_channel);
        authenticate_managed_runtime_v2(
            &mut control_channel,
            descriptor_bytes,
            settings_schema_bytes,
            admission,
        )?;
        let binding = storage_binding(&storage_configuration, admission)?;
        let vault_public_key = storage_configuration
            .vault_hpke_public_key_x25519
            .as_slice()
            .try_into()
            .map_err(|_| DeliveryIntentRuntimeErrorV1::Admission)?;
        let vault_context = StorageVaultRouteContextV1::new(
            storage_configuration.vault_instance_id.clone(),
            storage_configuration.vault_runtime_generation,
            vault_public_key,
        )
        .map_err(|_| DeliveryIntentRuntimeErrorV1::Admission)?;
        let mut leases = StorageVaultLeaseAdapterV1::new(
            InheritedKernelVaultRouteV2::new(control_channel),
            vault_context,
        );
        let password = resolve_storage_runtime_credential(&mut leases, &binding).await?;
        let password =
            std::str::from_utf8(&password).map_err(|_| DeliveryIntentRuntimeErrorV1::Admission)?;
        let persistence = CommunicationDeliveryIntentPersistenceV1::connect_runtime(
            &binding,
            &storage_configuration.database_id,
            &storage_configuration.pgbouncer_host,
            storage_configuration.pgbouncer_port,
            password,
        )
        .await
        .map_err(persistence_error)?;
        persistence
            .verify_storage_ready()
            .await
            .map_err(persistence_error)?;
        let mut control_channel = leases.into_route_port().into_channel();
        let event_access = request_managed_runtime_event_access_v2(
            &mut control_channel,
            &admission.logical_owner_id,
            &admission.registration_id,
            &admission.runtime_instance_id,
            admission.runtime_generation,
            admission.grant_epoch,
            event_credential_revision,
        )
        .map_err(|_| DeliveryIntentRuntimeErrorV1::Unavailable)?;
        let event_identity = RuntimeNatsIdentity::new(
            admission.runtime_instance_id.clone(),
            admission.runtime_generation,
            admission.grant_epoch,
        )
        .map_err(|_| DeliveryIntentRuntimeErrorV1::Admission)?;
        let event_publish_permit = event_access
            .publish_permit(
                &admission.registration_id,
                &admission.runtime_instance_id,
                admission.runtime_generation,
                admission.grant_epoch,
            )
            .map_err(|_| DeliveryIntentRuntimeErrorV1::Admission)?;
        let terminal_subscriptions = bind_terminal_subscriptions(
            event_access
                .subscribe_permits(
                    &admission.registration_id,
                    &admission.runtime_instance_id,
                    admission.runtime_generation,
                    admission.grant_epoch,
                )
                .map_err(|_| DeliveryIntentRuntimeErrorV1::Admission)?,
        )?;
        let event_connection = JetStreamClient::connect_runtime_with_jwt(
            event_hub_endpoint,
            event_identity,
            event_access.into_credential(),
        )
        .await
        .map_err(|_| DeliveryIntentRuntimeErrorV1::Unavailable)?;
        signal_managed_runtime_ready(&mut control_channel, admission)?;
        control_channel
            .inner_mut()
            .set_nonblocking(true)
            .map_err(|_| DeliveryIntentRuntimeErrorV1::Unavailable)?;
        Ok(Self {
            logical_owner_id: admission.logical_owner_id.clone(),
            control_channel,
            persistence,
            runtime_instance_id: admission.runtime_instance_id.clone(),
            runtime_generation: admission.runtime_generation,
            event_connection,
            event_publish_permit,
            terminal_subscriptions,
            next_terminal_subscription: 0,
        })
    }

    pub fn persistence(&self) -> &CommunicationDeliveryIntentPersistenceV1 {
        &self.persistence
    }

    pub async fn create_delivery_intent_v1(
        &mut self,
        planned: PlannedDeliveryIntentV1,
        created_at_unix_seconds: i64,
        dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    ) -> Result<CreateDeliveryIntentOutcomeV1, DeliveryIntentRuntimeErrorV1> {
        let command = {
            let mut materializer = ManagedDeliveryIntentBodyMaterializerV1 {
                control_channel: &mut self.control_channel,
                dispatcher,
            };
            prepare_create_delivery_intent_v1(
                self.logical_owner_id.clone(),
                planned,
                created_at_unix_seconds,
                &mut materializer,
            )
            .map_err(DeliveryIntentRuntimeErrorV1::Coordinator)?
        };
        self.persistence
            .create_intent(&command)
            .await
            .map_err(DeliveryIntentRuntimeErrorV1::Persistence)
    }

    pub fn pump_control_once(&mut self) -> Result<bool, DeliveryIntentRuntimeErrorV1> {
        let Some((correlation_id, _request)) = self
            .control_channel
            .try_receive_request()
            .map_err(|_| DeliveryIntentRuntimeErrorV1::Unavailable)?
        else {
            return Ok(false);
        };
        self.control_channel
            .write_response(
                correlation_id,
                ManagedRuntimeControlResponseV1 {
                    result: None,
                    error_code: "managed_runtime_control_unexpected_request".to_owned(),
                },
            )
            .map_err(|_| DeliveryIntentRuntimeErrorV1::Unavailable)?;
        Ok(true)
    }
}

fn validate_admission(
    admission: &DeliveryIntentRuntimeAdmissionV1,
) -> Result<(), DeliveryIntentRuntimeErrorV1> {
    if admission.logical_owner_id.is_empty()
        || admission.registration_id.is_empty()
        || admission.runtime_instance_id.is_empty()
        || admission.runtime_generation == 0
        || admission.grant_epoch == 0
    {
        return Err(DeliveryIntentRuntimeErrorV1::Admission);
    }
    Ok(())
}

async fn resolve_storage_runtime_credential(
    leases: &mut StorageVaultLeaseAdapterV1<InheritedKernelVaultRouteV2>,
    binding: &StorageBindingV1,
) -> Result<zeroize::Zeroizing<Vec<u8>>, DeliveryIntentRuntimeErrorV1> {
    const MAX_ATTEMPTS: usize = 20;
    for attempt in 0..MAX_ATTEMPTS {
        if let Ok(password) = leases.ensure_runtime_credential(binding).await {
            return Ok(password);
        }
        if attempt + 1 < MAX_ATTEMPTS {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    }
    Err(DeliveryIntentRuntimeErrorV1::Unavailable)
}

fn authenticate_managed_runtime_v2(
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    descriptor_bytes: Vec<u8>,
    settings_schema_bytes: Vec<u8>,
    admission: &DeliveryIntentRuntimeAdmissionV1,
) -> Result<(), DeliveryIntentRuntimeErrorV1> {
    control_channel
        .inner_mut()
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .and_then(|_| {
            control_channel
                .inner_mut()
                .set_write_timeout(Some(std::time::Duration::from_secs(5)))
        })
        .map_err(|_| DeliveryIntentRuntimeErrorV1::Unavailable)?;
    let response = control_channel
        .describe_managed_runtime(descriptor_bytes, settings_schema_bytes)
        .map_err(|_| DeliveryIntentRuntimeErrorV1::Unavailable)?;
    if response.registration_id != admission.registration_id
        || response.runtime_generation != admission.runtime_generation
        || response.grant_epoch != admission.grant_epoch
    {
        return Err(DeliveryIntentRuntimeErrorV1::Admission);
    }
    Ok(())
}

fn signal_managed_runtime_ready(
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    admission: &DeliveryIntentRuntimeAdmissionV1,
) -> Result<(), DeliveryIntentRuntimeErrorV1> {
    control_channel
        .signal_ready(ManagedRuntimeReadyRequestV1 {
            registration_id: admission.registration_id.clone(),
            runtime_generation: admission.runtime_generation,
            grant_epoch: admission.grant_epoch,
        })
        .map_err(|_| DeliveryIntentRuntimeErrorV1::Unavailable)?;
    control_channel
        .inner_mut()
        .set_read_timeout(None)
        .and_then(|_| control_channel.inner_mut().set_write_timeout(None))
        .map_err(|_| DeliveryIntentRuntimeErrorV1::Unavailable)
}

fn storage_binding(
    configuration: &ManagedStorageRuntimeConfigurationV1,
    admission: &DeliveryIntentRuntimeAdmissionV1,
) -> Result<StorageBindingV1, DeliveryIntentRuntimeErrorV1> {
    if configuration.runtime_instance_id != admission.runtime_instance_id
        || configuration.logical_owner_id != configuration.owner
        || configuration.storage_bundle_digest.len() != 32
        || configuration.storage_generation == 0
        || configuration.credential_revision == 0
        || configuration.role_epoch == 0
        || configuration.storage_bundle_revision == 0
    {
        return Err(DeliveryIntentRuntimeErrorV1::Admission);
    }
    let identity = StorageBindingIdentityV1::new(
        configuration.storage_instance_id.clone(),
        configuration.database_id.clone(),
        configuration.owner.clone(),
        admission.registration_id.clone(),
        configuration.runtime_instance_id.clone(),
    )
    .map_err(|_| DeliveryIntentRuntimeErrorV1::Admission)?;
    let fences = StorageBindingFencesV1::new(
        configuration.storage_generation,
        admission.runtime_generation,
        admission.grant_epoch,
        configuration.role_epoch,
        configuration.credential_revision,
        configuration.storage_bundle_revision,
    )
    .map_err(|_| DeliveryIntentRuntimeErrorV1::Admission)?;
    let budgets = StorageEffectiveBudgetsV1::new(
        u16::try_from(configuration.max_connections)
            .map_err(|_| DeliveryIntentRuntimeErrorV1::Admission)?,
        configuration.statement_timeout_millis,
    )
    .map_err(|_| DeliveryIntentRuntimeErrorV1::Admission)?;
    let access = StorageBindingAccessV1::new(
        configuration.runtime_principal.clone(),
        configuration.pool_alias.clone(),
        budgets,
        configuration
            .storage_bundle_digest
            .as_slice()
            .try_into()
            .map_err(|_| DeliveryIntentRuntimeErrorV1::Admission)?,
    )
    .map_err(|_| DeliveryIntentRuntimeErrorV1::Admission)?;
    StorageBindingV1::new(identity, fences, access)
        .map_err(|_| DeliveryIntentRuntimeErrorV1::Admission)
}

fn persistence_error(_: DeliveryIntentPersistenceErrorV1) -> DeliveryIntentRuntimeErrorV1 {
    DeliveryIntentRuntimeErrorV1::Unavailable
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admission_requires_current_runtime_fences() {
        let mut admission = DeliveryIntentRuntimeAdmissionV1 {
            logical_owner_id: "owner:test".to_owned(),
            registration_id: "delivery-intent".to_owned(),
            runtime_instance_id: "delivery-intent-1".to_owned(),
            runtime_generation: 1,
            grant_epoch: 1,
        };
        assert_eq!(validate_admission(&admission), Ok(()));
        admission.grant_epoch = 0;
        assert_eq!(
            validate_admission(&admission),
            Err(DeliveryIntentRuntimeErrorV1::Admission)
        );
    }
}
