//! Scheduler-owned PgBouncer connection without a URL-shaped secret carrier.

use std::time::Duration;

use hermes_runtime_protocol::v1::SchedulerRuntimeStorageBindingV1;
use hermes_storage_protocol::{
    StorageBindingAccessV1, StorageBindingFencesV1, StorageBindingIdentityV1, StorageBindingV1,
    StorageEffectiveBudgetsV1,
};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};

use super::runs::SchedulerPostgresStoreV1;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(2);

/// Non-secret PgBouncer endpoint staged by the authenticated Scheduler launch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SchedulerPostgresEndpointV1 {
    host: String,
    port: u16,
}

impl SchedulerPostgresEndpointV1 {
    pub fn new(host: String, port: u32) -> Result<Self, SchedulerStoreConnectionErrorV1> {
        (valid_host(&host) && u16::try_from(port).is_ok_and(|value| value > 0))
            .then_some(Self {
                host,
                port: u16::try_from(port).expect("validated PostgreSQL port fits u16"),
            })
            .ok_or(SchedulerStoreConnectionErrorV1::InvalidEndpoint)
    }
}

impl SchedulerPostgresStoreV1 {
    /// Opens Scheduler's own fenced PgBouncer pool. The password remains only
    /// in this call and is never represented as a URL or retained by the store.
    pub async fn connect_runtime(
        binding: &StorageBindingV1,
        endpoint: &SchedulerPostgresEndpointV1,
        password: &str,
    ) -> Result<Self, SchedulerStoreConnectionErrorV1> {
        if password.is_empty() {
            return Err(SchedulerStoreConnectionErrorV1::CredentialUnavailable);
        }
        let options = PgConnectOptions::new()
            .host(&endpoint.host)
            .port(endpoint.port)
            // PgBouncer admits runtime connections only through the fenced
            // alias. The alias resolves to the physical database inside the
            // private pooler configuration; exposing the physical name here
            // would bypass the per-runtime binding selection.
            .database(binding.access().pool_alias())
            .username(binding.access().runtime_principal())
            .password(password);
        let pool = PgPoolOptions::new()
            .max_connections(u32::from(
                binding.access().effective_budgets().max_connections(),
            ))
            .acquire_timeout(CONNECT_TIMEOUT)
            .connect_with(options)
            .await
            .map_err(|_| SchedulerStoreConnectionErrorV1::Unavailable)?;
        Ok(Self::new(pool))
    }
}

/// Assembles the public Storage binding only from Kernel-staged configuration
/// and the identity authenticated on the inherited Scheduler channel.
pub fn scheduler_storage_binding_from_runtime(
    configuration: &SchedulerRuntimeStorageBindingV1,
    registration_id: String,
    runtime_instance_id: String,
    runtime_generation: u64,
    grant_epoch: u64,
) -> Result<StorageBindingV1, SchedulerStoreConnectionErrorV1> {
    let identity = StorageBindingIdentityV1::new(
        configuration.storage_instance_id.clone(),
        configuration.database_id.clone(),
        configuration.owner.clone(),
        registration_id,
        runtime_instance_id,
    )
    .map_err(|_| SchedulerStoreConnectionErrorV1::InvalidBinding)?;
    let fences = StorageBindingFencesV1::new(
        configuration.storage_generation,
        runtime_generation,
        grant_epoch,
        configuration.role_epoch,
        configuration.credential_revision,
        configuration.storage_bundle_revision,
    )
    .map_err(|_| SchedulerStoreConnectionErrorV1::InvalidBinding)?;
    let budgets = StorageEffectiveBudgetsV1::new(
        u16::try_from(configuration.max_connections)
            .map_err(|_| SchedulerStoreConnectionErrorV1::InvalidBinding)?,
        configuration.statement_timeout_millis,
    )
    .map_err(|_| SchedulerStoreConnectionErrorV1::InvalidBinding)?;
    let digest = configuration
        .storage_bundle_digest
        .as_slice()
        .try_into()
        .map_err(|_| SchedulerStoreConnectionErrorV1::InvalidBinding)?;
    let access = StorageBindingAccessV1::new(
        configuration.runtime_principal.clone(),
        configuration.pool_alias.clone(),
        budgets,
        digest,
    )
    .map_err(|_| SchedulerStoreConnectionErrorV1::InvalidBinding)?;
    StorageBindingV1::new(identity, fences, access)
        .map_err(|_| SchedulerStoreConnectionErrorV1::InvalidBinding)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchedulerStoreConnectionErrorV1 {
    InvalidBinding,
    InvalidEndpoint,
    CredentialUnavailable,
    Unavailable,
}

fn valid_host(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 255
        && value.is_ascii()
        && !value.contains(['/', '@', ':', '?', '#', ' '])
}
