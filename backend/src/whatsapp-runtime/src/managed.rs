//! Kernel-admitted WhatsApp runtime composition. It receives no browser state
//! or provider credential material; the host owns that boundary.

use std::io::{ErrorKind, Read, Write};
use std::os::unix::{
    fs::PermissionsExt,
    net::{UnixListener, UnixStream},
};
use std::time::Duration;

use hermes_events_jetstream::{
    JetStreamClient, RuntimeJetStreamConnection, RuntimeNatsIdentity, RuntimePublishPermitV1,
    request_managed_runtime_event_access,
};
use hermes_runtime_protocol::v1::{
    DescribeManagedRuntimeRequestV1, ManagedIntegrationHostBridgeConfigurationV1,
    ManagedRuntimeControlRequestV1, ManagedRuntimeControlResponseV1, ManagedRuntimeReadyRequestV1,
    ManagedStorageRuntimeConfigurationV1, managed_runtime_control_request_v1::Operation,
    managed_runtime_control_response_v1::Result as ControlResult,
};
use hermes_runtime_protocol::validation::integration_host_bridge::validate_managed_integration_host_bridge_configuration;
use hermes_storage_protocol::{
    StorageBindingAccessV1, StorageBindingFencesV1, StorageBindingIdentityV1, StorageBindingV1,
    StorageEffectiveBudgetsV1,
};
use hermes_storage_vault::{
    InheritedKernelVaultRouteV1, StorageVaultLeaseAdapterV1, StorageVaultRouteContextV1,
};
use prost::Message;

use crate::{
    WhatsAppCommandQueueError, WhatsAppRuntimeAdmission, WhatsAppRuntimeIdentity,
    accept_host_observation, claim_provider_commands, relay_communications_outbox_once,
};
use hermes_whatsapp_api::{
    WhatsAppProviderCommand,
    host_bridge::{WhatsAppHostBridgeEnvelopeV1, WhatsAppHostBridgeHandshakeV1},
};
use hermes_whatsapp_persistence::WhatsAppDurablePersistence;

const MAX_FRAME_BYTES: usize = 512 * 1024;
const CONTROL_TIMEOUT: Duration = Duration::from_secs(5);

pub struct WhatsAppAdmittedRuntime {
    pub control_channel: UnixStream,
    pub durable: WhatsAppDurablePersistence,
    event_connection: RuntimeJetStreamConnection,
    event_publish_permit: RuntimePublishPermitV1,
    identity: WhatsAppRuntimeIdentity,
    host_bridge_socket_path: String,
    host_bridge_route_binding: [u8; 32],
}

#[derive(Debug, Eq, PartialEq)]
pub enum WhatsAppBootstrapError {
    Admission,
    Control,
    HostBridge,
    Storage,
    Credential,
    Persistence,
    EventHub,
}

#[allow(clippy::too_many_arguments)]
pub async fn open_admitted_runtime(
    mut control_channel: UnixStream,
    descriptor_bytes: Vec<u8>,
    settings_schema_bytes: Vec<u8>,
    admission: &WhatsAppRuntimeAdmission,
    storage_configuration: ManagedStorageRuntimeConfigurationV1,
    host_bridge_configuration: ManagedIntegrationHostBridgeConfigurationV1,
    event_hub_endpoint: &str,
    event_credential_revision: u64,
) -> Result<WhatsAppAdmittedRuntime, WhatsAppBootstrapError> {
    if descriptor_bytes.is_empty()
        || settings_schema_bytes.is_empty()
        || admission.logical_owner_id != "whatsapp"
        || admission.runtime_instance_id.trim().is_empty()
        || event_hub_endpoint.trim().is_empty()
        || event_credential_revision == 0
    {
        return Err(WhatsAppBootstrapError::Admission);
    }
    control_channel
        .set_read_timeout(Some(CONTROL_TIMEOUT))
        .and_then(|_| control_channel.set_write_timeout(Some(CONTROL_TIMEOUT)))
        .map_err(|_| WhatsAppBootstrapError::Control)?;
    write_frame(
        &mut control_channel,
        &ManagedRuntimeControlRequestV1 {
            operation: Some(Operation::Describe(DescribeManagedRuntimeRequestV1 {
                descriptor_bytes,
                settings_schema_bytes,
            })),
        }
        .encode_to_vec(),
    )?;
    let response =
        ManagedRuntimeControlResponseV1::decode(read_frame(&mut control_channel)?.as_slice())
            .map_err(|_| WhatsAppBootstrapError::Control)?;
    let (registration_id, runtime_generation, grant_epoch) = match response.result {
        Some(ControlResult::Describe(value))
            if response.error_code.is_empty()
                && !value.registration_id.is_empty()
                && value.runtime_generation != 0
                && value.grant_epoch != 0 =>
        {
            (
                value.registration_id,
                value.runtime_generation,
                value.grant_epoch,
            )
        }
        _ => return Err(WhatsAppBootstrapError::Admission),
    };
    if registration_id != admission.module_registration_id
        || runtime_generation != admission.runtime_generation
        || grant_epoch != admission.grant_epoch
    {
        return Err(WhatsAppBootstrapError::Admission);
    }
    write_frame(
        &mut control_channel,
        &ManagedRuntimeControlRequestV1 {
            operation: Some(Operation::Ready(ManagedRuntimeReadyRequestV1 {
                registration_id,
                runtime_generation,
                grant_epoch,
            })),
        }
        .encode_to_vec(),
    )?;
    control_channel
        .set_read_timeout(None)
        .and_then(|_| control_channel.set_write_timeout(None))
        .map_err(|_| WhatsAppBootstrapError::Control)?;

    let binding = storage_binding(&storage_configuration, admission)?;
    let (host_bridge_socket_path, host_bridge_route_binding) =
        host_bridge_route(&host_bridge_configuration, admission)?;
    let storage_context = StorageVaultRouteContextV1::new(
        storage_configuration.vault_instance_id.clone(),
        storage_configuration.vault_runtime_generation,
        storage_configuration
            .vault_hpke_public_key_x25519
            .as_slice()
            .try_into()
            .map_err(|_| WhatsAppBootstrapError::Storage)?,
    )
    .map_err(|_| WhatsAppBootstrapError::Storage)?;
    let mut storage_leases = StorageVaultLeaseAdapterV1::new(
        InheritedKernelVaultRouteV1::new(
            control_channel
                .try_clone()
                .map_err(|_| WhatsAppBootstrapError::Control)?,
        ),
        storage_context,
    );
    let lease_id = storage_leases
        .issue_runtime_credential(&binding)
        .await
        .map_err(|_| WhatsAppBootstrapError::Credential)?;
    let password = storage_leases
        .resolve_runtime_credential(&binding, lease_id)
        .await
        .map_err(|_| WhatsAppBootstrapError::Credential)?;
    let password =
        std::str::from_utf8(&password).map_err(|_| WhatsAppBootstrapError::Credential)?;
    let durable = WhatsAppDurablePersistence::connect_runtime(
        &binding,
        &storage_configuration.database_id,
        &storage_configuration.pgbouncer_host,
        storage_configuration.pgbouncer_port,
        password,
    )
    .await
    .map_err(|_| WhatsAppBootstrapError::Persistence)?;
    durable
        .initialize()
        .await
        .map_err(|_| WhatsAppBootstrapError::Persistence)?;

    let event_access = request_managed_runtime_event_access(
        &mut control_channel,
        &admission.logical_owner_id,
        &admission.module_registration_id,
        &admission.runtime_instance_id,
        admission.runtime_generation,
        admission.grant_epoch,
        event_credential_revision,
    )
    .map_err(|_| WhatsAppBootstrapError::EventHub)?;
    let identity = RuntimeNatsIdentity::new(
        admission.runtime_instance_id.clone(),
        admission.runtime_generation,
        admission.grant_epoch,
    )
    .map_err(|_| WhatsAppBootstrapError::EventHub)?;
    let event_publish_permit = event_access
        .publish_permit(
            &admission.module_registration_id,
            &admission.runtime_instance_id,
            admission.runtime_generation,
            admission.grant_epoch,
        )
        .map_err(|_| WhatsAppBootstrapError::EventHub)?;
    let event_connection = JetStreamClient::connect_runtime_with_jwt(
        event_hub_endpoint,
        identity,
        event_access.into_credential(),
    )
    .await
    .map_err(|_| WhatsAppBootstrapError::EventHub)?;
    Ok(WhatsAppAdmittedRuntime {
        control_channel,
        durable,
        event_connection,
        event_publish_permit,
        identity: WhatsAppRuntimeIdentity {
            runtime_instance_id: admission.runtime_instance_id.clone(),
            runtime_generation: admission.runtime_generation,
        },
        host_bridge_socket_path,
        host_bridge_route_binding,
    })
}

impl WhatsAppAdmittedRuntime {
    /// Binds the exact host bridge endpoint staged by Kernel. The caller owns
    /// scheduling and shutdown; this runtime never invents a socket path or
    /// removes an existing endpoint.
    pub fn bind_host_bridge_listener(&self) -> Result<UnixListener, WhatsAppBootstrapError> {
        let listener = UnixListener::bind(&self.host_bridge_socket_path)
            .map_err(|_| WhatsAppBootstrapError::HostBridge)?;
        std::fs::set_permissions(
            &self.host_bridge_socket_path,
            std::fs::Permissions::from_mode(0o600),
        )
        .map_err(|_| WhatsAppBootstrapError::HostBridge)?;
        Ok(listener)
    }

    pub fn serve_host_bridge_once(
        &self,
        listener: &UnixListener,
        handle: &tokio::runtime::Handle,
    ) -> Result<(), WhatsAppBootstrapError> {
        let (stream, _) = listener
            .accept()
            .map_err(|_| WhatsAppBootstrapError::HostBridge)?;
        crate::client_transport::serve_connection(stream, self, handle)
            .map_err(|_| WhatsAppBootstrapError::HostBridge)
    }

    /// Serves one short-lived host connection when one is already pending.
    /// The process root owns scheduling and continues relaying the durable
    /// Communications outbox when no host request is waiting.
    pub fn try_serve_host_bridge_once(
        &self,
        listener: &UnixListener,
        handle: &tokio::runtime::Handle,
    ) -> Result<bool, WhatsAppBootstrapError> {
        let (stream, _) = match listener.accept() {
            Ok(value) => value,
            Err(error) if error.kind() == ErrorKind::WouldBlock => return Ok(false),
            Err(_) => return Err(WhatsAppBootstrapError::HostBridge),
        };
        crate::client_transport::serve_connection(stream, self, handle)
            .map_err(|_| WhatsAppBootstrapError::HostBridge)?;
        Ok(true)
    }

    pub async fn accept_host_observation(
        &self,
        envelope: &WhatsAppHostBridgeEnvelopeV1,
        recorded_at_unix_seconds: i64,
        recorded_at_nanos: i32,
    ) -> Result<(), crate::WhatsAppHostIngressError> {
        accept_host_observation(
            &self.durable,
            &self.identity,
            envelope,
            recorded_at_unix_seconds,
            recorded_at_nanos,
        )
        .await
    }

    pub fn accepts_host_bridge_handshake(&self, handshake: &WhatsAppHostBridgeHandshakeV1) -> bool {
        handshake.route_binding_sha256 == self.host_bridge_route_binding
    }

    pub async fn claim_host_commands(
        &self,
        account_id: &str,
        host_claim_id: &str,
        now_unix_seconds: i64,
        lease_seconds: i64,
        limit: i64,
    ) -> Result<Vec<WhatsAppProviderCommand>, WhatsAppCommandQueueError> {
        claim_provider_commands(
            &self.durable,
            account_id,
            host_claim_id,
            now_unix_seconds,
            lease_seconds,
            limit,
        )
        .await
    }

    pub async fn relay_communications_outbox(
        &self,
        published_at_unix_seconds: i64,
    ) -> Result<usize, crate::WhatsAppCommunicationsOutboxRelayError> {
        relay_communications_outbox_once(
            &self.durable,
            &self.event_connection,
            &self.event_publish_permit,
            published_at_unix_seconds,
        )
        .await
    }
}

fn host_bridge_route(
    configuration: &ManagedIntegrationHostBridgeConfigurationV1,
    admission: &WhatsAppRuntimeAdmission,
) -> Result<(String, [u8; 32]), WhatsAppBootstrapError> {
    validate_managed_integration_host_bridge_configuration(configuration)
        .map_err(|_| WhatsAppBootstrapError::Admission)?;
    if configuration.owner_id != admission.logical_owner_id
        || configuration.registration_id != admission.module_registration_id
        || configuration.runtime_instance_id != admission.runtime_instance_id
        || configuration.runtime_generation != admission.runtime_generation
        || configuration.grant_epoch != admission.grant_epoch
    {
        return Err(WhatsAppBootstrapError::Admission);
    }
    let route_binding = configuration
        .route_binding_sha256
        .as_slice()
        .try_into()
        .map_err(|_| WhatsAppBootstrapError::Admission)?;
    Ok((configuration.socket_path.clone(), route_binding))
}

fn storage_binding(
    configuration: &ManagedStorageRuntimeConfigurationV1,
    admission: &WhatsAppRuntimeAdmission,
) -> Result<StorageBindingV1, WhatsAppBootstrapError> {
    if configuration.runtime_instance_id != admission.runtime_instance_id
        || configuration.logical_owner_id != configuration.owner
        || configuration.storage_bundle_digest.len() != 32
        || configuration.storage_generation == 0
        || configuration.credential_revision == 0
        || configuration.role_epoch == 0
        || configuration.storage_bundle_revision == 0
    {
        return Err(WhatsAppBootstrapError::Admission);
    }
    let identity = StorageBindingIdentityV1::new(
        configuration.storage_instance_id.clone(),
        configuration.database_id.clone(),
        configuration.owner.clone(),
        admission.module_registration_id.clone(),
        configuration.runtime_instance_id.clone(),
    )
    .map_err(|_| WhatsAppBootstrapError::Storage)?;
    let fences = StorageBindingFencesV1::new(
        configuration.storage_generation,
        admission.runtime_generation,
        admission.grant_epoch,
        configuration.role_epoch,
        configuration.credential_revision,
        configuration.storage_bundle_revision,
    )
    .map_err(|_| WhatsAppBootstrapError::Storage)?;
    let budgets = StorageEffectiveBudgetsV1::new(
        u16::try_from(configuration.max_connections)
            .map_err(|_| WhatsAppBootstrapError::Storage)?,
        configuration.statement_timeout_millis,
    )
    .map_err(|_| WhatsAppBootstrapError::Storage)?;
    let access = StorageBindingAccessV1::new(
        configuration.runtime_principal.clone(),
        configuration.pool_alias.clone(),
        budgets,
        configuration
            .storage_bundle_digest
            .as_slice()
            .try_into()
            .map_err(|_| WhatsAppBootstrapError::Storage)?,
    )
    .map_err(|_| WhatsAppBootstrapError::Storage)?;
    StorageBindingV1::new(identity, fences, access).map_err(|_| WhatsAppBootstrapError::Storage)
}

fn write_frame(channel: &mut UnixStream, bytes: &[u8]) -> Result<(), WhatsAppBootstrapError> {
    if bytes.is_empty() || bytes.len() > MAX_FRAME_BYTES {
        return Err(WhatsAppBootstrapError::Control);
    }
    let mut length = u32::try_from(bytes.len()).map_err(|_| WhatsAppBootstrapError::Control)?;
    let mut prefix = Vec::with_capacity(5);
    while length >= 0x80 {
        prefix.push((length as u8 & 0x7f) | 0x80);
        length >>= 7;
    }
    prefix.push(length as u8);
    channel
        .write_all(&prefix)
        .and_then(|_| channel.write_all(bytes))
        .and_then(|_| channel.flush())
        .map_err(|_| WhatsAppBootstrapError::Control)
}

fn read_frame(channel: &mut UnixStream) -> Result<Vec<u8>, WhatsAppBootstrapError> {
    let length =
        usize::try_from(read_varint(channel)?).map_err(|_| WhatsAppBootstrapError::Control)?;
    if length == 0 || length > MAX_FRAME_BYTES {
        return Err(WhatsAppBootstrapError::Control);
    }
    let mut bytes = vec![0_u8; length];
    channel
        .read_exact(&mut bytes)
        .map_err(|_| WhatsAppBootstrapError::Control)?;
    Ok(bytes)
}

fn read_varint(channel: &mut UnixStream) -> Result<u64, WhatsAppBootstrapError> {
    let mut value = 0_u64;
    for index in 0..5 {
        let mut byte = [0_u8; 1];
        channel
            .read_exact(&mut byte)
            .map_err(|_| WhatsAppBootstrapError::Control)?;
        value |= u64::from(byte[0] & 0x7f) << (index * 7);
        if byte[0] & 0x80 == 0 {
            return Ok(value);
        }
    }
    Err(WhatsAppBootstrapError::Control)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn admission() -> WhatsAppRuntimeAdmission {
        WhatsAppRuntimeAdmission {
            logical_owner_id: "whatsapp".to_owned(),
            module_registration_id: "whatsapp_runtime".to_owned(),
            runtime_instance_id: "whatsapp_runtime_1".to_owned(),
            runtime_generation: 2,
            grant_epoch: 3,
        }
    }

    #[test]
    fn accepts_only_a_kernel_fenced_whatsapp_host_route() {
        let configuration = ManagedIntegrationHostBridgeConfigurationV1 {
            major: 1,
            kernel_instance_id: "kernel_1".to_owned(),
            owner_id: "whatsapp".to_owned(),
            registration_id: "whatsapp_runtime".to_owned(),
            runtime_instance_id: "whatsapp_runtime_1".to_owned(),
            runtime_generation: 2,
            grant_epoch: 3,
            socket_path: "/private/tmp/hermes/whatsapp.sock".to_owned(),
            route_binding_sha256: vec![1; 32],
        };

        assert_eq!(
            host_bridge_route(&configuration, &admission()),
            Ok((configuration.socket_path.clone(), [1; 32]))
        );
    }

    #[test]
    fn rejects_a_stale_host_route() {
        let configuration = ManagedIntegrationHostBridgeConfigurationV1 {
            major: 1,
            kernel_instance_id: "kernel_1".to_owned(),
            owner_id: "whatsapp".to_owned(),
            registration_id: "whatsapp_runtime".to_owned(),
            runtime_instance_id: "whatsapp_runtime_1".to_owned(),
            runtime_generation: 1,
            grant_epoch: 3,
            socket_path: "/private/tmp/hermes/whatsapp.sock".to_owned(),
            route_binding_sha256: vec![1; 32],
        };

        assert!(matches!(
            host_bridge_route(&configuration, &admission()),
            Err(WhatsAppBootstrapError::Admission)
        ));
    }
}
