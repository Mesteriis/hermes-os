//! Module registrations, grants, and external runtime attestations.

use hermes_kernel_control_store::{
    ExternalRuntimeAttestation, GrantSet, ModuleBlobQuotaRequestV1, ModuleEventRouteRequestV1,
    ModuleGrantSnapshot, ModuleRegistration, ModuleRegistrationState, ModuleSchedulerJobRequestV1,
    ModuleStorageRequestV1, ModuleVaultPurposeRequestV1,
};
use rusqlite::{Connection, OptionalExtension, params};

use crate::{
    SqliteControlStore, StoreError, module_registration_state_from_str, valid_capability_ids,
    valid_identity_token,
};

use super::{
    blob_request::{insert_blob_quota_requests, validate_blob_quota_requests},
    event_request::{insert_event_route_requests, validate_event_route_requests},
    scheduler_request::{insert_scheduler_job_requests, validate_scheduler_job_requests},
    storage_request::{insert_storage_requests, validate_storage_requests},
    vault_purpose_request::{insert_vault_purpose_requests, validate_vault_purpose_requests},
};

impl SqliteControlStore {
    pub fn create_pending_registration(
        &self,
        registration: &ModuleRegistration,
        requested_capability_ids: &[String],
    ) -> Result<(), StoreError> {
        self.create_pending_registration_with_requests(
            registration,
            requested_capability_ids,
            &[],
            &[],
            &[],
        )
    }

    pub fn create_pending_registration_with_requests(
        &self,
        registration: &ModuleRegistration,
        requested_capability_ids: &[String],
        storage_requests: &[ModuleStorageRequestV1],
        event_requests: &[ModuleEventRouteRequestV1],
        blob_requests: &[ModuleBlobQuotaRequestV1],
    ) -> Result<(), StoreError> {
        self.create_pending_registration_with_descriptor_requests(
            registration,
            requested_capability_ids,
            storage_requests,
            event_requests,
            blob_requests,
            &[],
            &[],
        )
    }

    pub fn create_pending_registration_with_descriptor_requests(
        &self,
        registration: &ModuleRegistration,
        requested_capability_ids: &[String],
        storage_requests: &[ModuleStorageRequestV1],
        event_requests: &[ModuleEventRouteRequestV1],
        blob_requests: &[ModuleBlobQuotaRequestV1],
        scheduler_requests: &[ModuleSchedulerJobRequestV1],
        vault_purpose_requests: &[ModuleVaultPurposeRequestV1],
    ) -> Result<(), StoreError> {
        validate_pending_registration(registration, requested_capability_ids)?;
        validate_storage_requests(registration, requested_capability_ids, storage_requests)?;
        validate_event_route_requests(registration, requested_capability_ids, event_requests)?;
        validate_blob_quota_requests(registration, requested_capability_ids, blob_requests)?;
        validate_scheduler_job_requests(
            registration,
            requested_capability_ids,
            scheduler_requests,
        )?;
        validate_vault_purpose_requests(registration, requested_capability_ids, vault_purpose_requests)?;
        let registration = registration.clone();
        let capabilities = requested_capability_ids.to_vec();
        let storage_requests = storage_requests.to_vec();
        let event_requests = event_requests.to_vec();
        let blob_requests = blob_requests.to_vec();
        let scheduler_requests = scheduler_requests.to_vec();
        let vault_purpose_requests = vault_purpose_requests.to_vec();
        self.with_connection(move |connection| {
            let transaction = connection.transaction()?;
            insert_pending_registration(&transaction, &registration, &capabilities)?;
            insert_storage_requests(&transaction, &storage_requests)?;
            insert_event_route_requests(&transaction, &event_requests)?;
            insert_blob_quota_requests(&transaction, &blob_requests)?;
            insert_scheduler_job_requests(&transaction, &scheduler_requests)?;
            insert_vault_purpose_requests(&transaction, &vault_purpose_requests)?;
            transaction.commit()?;
            Ok(())
        })
    }

    pub fn module_registration(
        &self,
        registration_id: &str,
    ) -> Result<Option<ModuleRegistration>, StoreError> {
        let registration_id = registration_id.to_owned();
        self.with_connection(move |connection| {
            read_module_registration(connection, &registration_id)
        })
    }

    pub fn transition_module_registration(
        &self,
        registration_id: &str,
        next: ModuleRegistrationState,
    ) -> Result<ModuleRegistration, StoreError> {
        let registration_id = registration_id.to_owned();
        self.with_connection(move |connection| {
            let transaction = connection.transaction()?;
            let current = read_required_registration(&transaction, &registration_id)?;
            if !current.state().can_transition_to(next) {
                return Err(StoreError::InvalidModuleRegistrationTransition);
            }
            let transitioned = transition_registration(&transaction, &current, next)?;
            transaction.commit()?;
            Ok(transitioned)
        })
    }

    pub fn approve_module_registration(
        &self,
        registration_id: &str,
        capability_ids: &[String],
    ) -> Result<GrantSet, StoreError> {
        if !valid_capability_ids(capability_ids) {
            return Err(StoreError::InvalidCapabilityGrant);
        }
        let registration_id = registration_id.to_owned();
        let capabilities = capability_ids.to_vec();
        self.with_connection(move |connection| {
            approve_registration(connection, &registration_id, &capabilities)
        })
    }

    pub fn module_grant_snapshot(
        &self,
        registration_id: &str,
    ) -> Result<Option<ModuleGrantSnapshot>, StoreError> {
        let registration_id = registration_id.to_owned();
        self.with_connection(move |connection| {
            let transaction = connection.transaction()?;
            let Some(registration) = read_module_registration(&transaction, &registration_id)?
            else {
                transaction.commit()?;
                return Ok(None);
            };
            let grants = if registration.state() == ModuleRegistrationState::Approved {
                Some(GrantSet::new(
                    &registration_id,
                    registration.grant_epoch(),
                    read_approved_capabilities(&transaction, &registration_id)?,
                ))
            } else {
                None
            };
            transaction.commit()?;
            Ok(Some(ModuleGrantSnapshot::new(registration, grants)))
        })
    }

    pub fn approved_module_grant_snapshots(&self) -> Result<Vec<ModuleGrantSnapshot>, StoreError> {
        self.with_connection(|connection| {
            let transaction = connection.transaction()?;
            let registrations = read_approved_registrations(&transaction)?;
            let snapshots = registrations
                .into_iter()
                .map(|registration| {
                    let grants = GrantSet::new(
                        registration.registration_id(),
                        registration.grant_epoch(),
                        read_approved_capabilities(&transaction, registration.registration_id())?,
                    );
                    Ok(ModuleGrantSnapshot::new(registration, Some(grants)))
                })
                .collect::<Result<Vec<_>, StoreError>>()?;
            transaction.commit()?;
            Ok(snapshots)
        })
    }

    pub fn attest_external_runtime(
        &self,
        attestation: &ExternalRuntimeAttestation,
    ) -> Result<(), StoreError> {
        validate_attestation(attestation)?;
        let attestation = attestation.clone();
        self.with_connection(move |connection| write_attestation(connection, &attestation))
    }

    pub fn effective_external_runtime_attestation(
        &self,
        registration_id: &str,
    ) -> Result<Option<ExternalRuntimeAttestation>, StoreError> {
        let registration_id = registration_id.to_owned();
        self.with_connection(move |connection| {
            read_effective_attestation(connection, &registration_id)
        })
    }
}

fn validate_pending_registration(
    registration: &ModuleRegistration,
    capabilities: &[String],
) -> Result<(), StoreError> {
    let valid = registration.state() == ModuleRegistrationState::Pending
        && registration.grant_epoch() == 1
        && valid_identity_token(registration.registration_id())
        && valid_identity_token(registration.module_id())
        && valid_identity_token(registration.owner_id())
        && valid_capability_ids(capabilities);
    valid
        .then_some(())
        .ok_or(StoreError::InvalidModuleRegistration)
}

fn insert_pending_registration(
    connection: &Connection,
    registration: &ModuleRegistration,
    capabilities: &[String],
) -> Result<(), StoreError> {
    let changed = connection.execute(
        "INSERT OR IGNORE INTO hermes_kernel_module_registration
         (registration_id, module_id, owner_id, descriptor_sha256, state, grant_epoch)
         VALUES (?1, ?2, ?3, ?4, 'pending', 1)",
        params![
            registration.registration_id(),
            registration.module_id(),
            registration.owner_id(),
            registration.descriptor_sha256().as_slice()
        ],
    )?;
    if changed != 1 {
        return Err(StoreError::ModuleRegistrationAlreadyExists);
    }
    for capability in capabilities {
        connection.execute(
            "INSERT INTO hermes_kernel_module_registration_capability
             (registration_id, capability_id, approved) VALUES (?1, ?2, 0)",
            params![registration.registration_id(), capability],
        )?;
    }
    Ok(())
}

fn approve_registration(
    connection: &mut Connection,
    registration_id: &str,
    capabilities: &[String],
) -> Result<GrantSet, StoreError> {
    let transaction = connection.transaction()?;
    let current = read_required_registration(&transaction, registration_id)?;
    if !matches!(
        current.state(),
        ModuleRegistrationState::Pending | ModuleRegistrationState::Suspended
    ) {
        return Err(StoreError::InvalidModuleRegistrationTransition);
    }
    transaction.execute(
        "UPDATE hermes_kernel_module_registration_capability SET approved = 0 WHERE registration_id = ?1",
        [registration_id],
    )?;
    approve_capabilities(&transaction, registration_id, capabilities)?;
    let transitioned =
        transition_registration(&transaction, &current, ModuleRegistrationState::Approved)?;
    let grants = read_approved_capabilities(&transaction, registration_id)?;
    transaction.commit()?;
    Ok(GrantSet::new(
        registration_id,
        transitioned.grant_epoch(),
        grants,
    ))
}

fn approve_capabilities(
    connection: &Connection,
    registration_id: &str,
    capabilities: &[String],
) -> Result<(), StoreError> {
    for capability in capabilities {
        let changed = connection.execute(
            "UPDATE hermes_kernel_module_registration_capability SET approved = 1
             WHERE registration_id = ?1 AND capability_id = ?2",
            params![registration_id, capability],
        )?;
        if changed != 1 {
            return Err(StoreError::InvalidCapabilityGrant);
        }
    }
    Ok(())
}

fn transition_registration(
    connection: &Connection,
    current: &ModuleRegistration,
    next: ModuleRegistrationState,
) -> Result<ModuleRegistration, StoreError> {
    let next_epoch = current
        .grant_epoch()
        .checked_add(1)
        .ok_or(StoreError::RecoveryFenceOverflow)?;
    let changed = connection.execute(
        "UPDATE hermes_kernel_module_registration SET state = ?1, grant_epoch = ?2
         WHERE registration_id = ?3 AND state = ?4 AND grant_epoch = ?5",
        params![
            next.as_str(),
            as_sql(next_epoch)?,
            current.registration_id(),
            current.state().as_str(),
            as_sql(current.grant_epoch())?
        ],
    )?;
    if changed != 1 {
        return Err(StoreError::InvalidModuleRegistrationTransition);
    }
    Ok(ModuleRegistration::new(
        current.registration_id(),
        current.module_id(),
        current.owner_id(),
        *current.descriptor_sha256(),
        next,
        next_epoch,
    ))
}

fn validate_attestation(attestation: &ExternalRuntimeAttestation) -> Result<(), StoreError> {
    let valid = valid_identity_token(attestation.registration_id())
        && valid_identity_token(attestation.runtime_id())
        && attestation.runtime_generation() > 0
        && attestation.grant_epoch() > 0;
    valid
        .then_some(())
        .ok_or(StoreError::InvalidExternalRuntimeAttestation)
}

fn write_attestation(
    connection: &mut Connection,
    attestation: &ExternalRuntimeAttestation,
) -> Result<(), StoreError> {
    let transaction = connection.transaction()?;
    let registration = read_required_registration(&transaction, attestation.registration_id())?;
    if registration.state() != ModuleRegistrationState::Approved
        || registration.grant_epoch() != attestation.grant_epoch()
    {
        return Err(StoreError::StaleExternalRuntimeAttestation);
    }
    let changed = transaction.execute(
        "INSERT INTO hermes_kernel_external_runtime_attestation
         (registration_id, runtime_id, runtime_generation, grant_epoch, distribution_sha256)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(registration_id) DO UPDATE SET runtime_id=excluded.runtime_id,
         runtime_generation=excluded.runtime_generation, grant_epoch=excluded.grant_epoch,
         distribution_sha256=excluded.distribution_sha256
         WHERE excluded.runtime_generation > hermes_kernel_external_runtime_attestation.runtime_generation",
        params![attestation.registration_id(), attestation.runtime_id(), as_sql(attestation.runtime_generation())?, as_sql(attestation.grant_epoch())?, attestation.distribution_sha256().as_slice()],
    )?;
    if changed != 1 {
        return Err(StoreError::StaleExternalRuntimeAttestation);
    }
    transaction.commit()?;
    Ok(())
}

fn read_effective_attestation(
    connection: &mut Connection,
    registration_id: &str,
) -> Result<Option<ExternalRuntimeAttestation>, StoreError> {
    let transaction = connection.transaction()?;
    let registration = read_required_registration(&transaction, registration_id)?;
    if registration.state() != ModuleRegistrationState::Approved {
        return Ok(None);
    }
    let result = transaction
        .query_row(
            "SELECT runtime_id, runtime_generation, grant_epoch, distribution_sha256
         FROM hermes_kernel_external_runtime_attestation
         WHERE registration_id = ?1 AND grant_epoch = ?2",
            params![registration_id, as_sql(registration.grant_epoch())?],
            |row| decode_attestation(row, registration_id),
        )
        .optional()?;
    transaction.commit()?;
    Ok(result)
}

pub(crate) fn read_module_registration(
    connection: &Connection,
    registration_id: &str,
) -> Result<Option<ModuleRegistration>, StoreError> {
    connection
        .query_row(
            "SELECT registration_id, module_id, owner_id, descriptor_sha256, state, grant_epoch
         FROM hermes_kernel_module_registration WHERE registration_id = ?1",
            [registration_id],
            decode_registration,
        )
        .optional()
        .map_err(StoreError::from)
}

pub(crate) fn read_required_registration(
    connection: &Connection,
    registration_id: &str,
) -> Result<ModuleRegistration, StoreError> {
    read_module_registration(connection, registration_id)?
        .ok_or(StoreError::ModuleRegistrationMissing)
}

fn read_approved_registrations(
    connection: &Connection,
) -> Result<Vec<ModuleRegistration>, StoreError> {
    let mut statement = connection.prepare(
        "SELECT registration_id, module_id, owner_id, descriptor_sha256, state, grant_epoch
         FROM hermes_kernel_module_registration WHERE state = 'approved' ORDER BY registration_id",
    )?;
    let rows = statement.query_map([], decode_registration)?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(StoreError::from)
}

pub(crate) fn read_approved_capabilities(
    connection: &Connection,
    registration_id: &str,
) -> Result<Vec<String>, StoreError> {
    let mut statement = connection.prepare(
        "SELECT capability_id FROM hermes_kernel_module_registration_capability
         WHERE registration_id = ?1 AND approved = 1 ORDER BY capability_id",
    )?;
    let rows = statement.query_map([registration_id], |row| row.get::<_, String>(0))?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(StoreError::from)
}

fn decode_registration(row: &rusqlite::Row<'_>) -> Result<ModuleRegistration, rusqlite::Error> {
    let digest: Vec<u8> = row.get(3)?;
    let state = module_registration_state_from_str(&row.get::<_, String>(4)?)
        .ok_or(rusqlite::Error::InvalidQuery)?;
    Ok(ModuleRegistration::new(
        row.get::<_, String>(0)?,
        row.get::<_, String>(1)?,
        row.get::<_, String>(2)?,
        digest
            .try_into()
            .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(3, 32))?,
        state,
        u64::try_from(row.get::<_, i64>(5)?)
            .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(5, 0))?,
    ))
}

fn decode_attestation(
    row: &rusqlite::Row<'_>,
    registration_id: &str,
) -> Result<ExternalRuntimeAttestation, rusqlite::Error> {
    let digest: Vec<u8> = row.get(3)?;
    Ok(ExternalRuntimeAttestation::new(
        registration_id,
        row.get::<_, String>(0)?,
        u64::try_from(row.get::<_, i64>(1)?)
            .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(1, 0))?,
        u64::try_from(row.get::<_, i64>(2)?)
            .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(2, 0))?,
        digest
            .try_into()
            .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(3, 32))?,
    ))
}

fn as_sql(value: u64) -> Result<i64, StoreError> {
    i64::try_from(value).map_err(|_| StoreError::RecoveryFenceOverflow)
}
