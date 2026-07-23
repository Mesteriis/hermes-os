use std::collections::BTreeSet;

use hermes_kernel_control_store::{ModuleRegistration, ModuleVaultPurposeRequestV1};
use rusqlite::{Connection, params};

use crate::{SqliteControlStore, StoreError, valid_capability_ids, valid_identity_token};

impl SqliteControlStore {
    pub fn module_vault_purpose_requests(
        &self,
        registration_id: &str,
        capability_id: &str,
    ) -> Result<Vec<ModuleVaultPurposeRequestV1>, StoreError> {
        let registration_id = registration_id.to_owned();
        let capability_id = capability_id.to_owned();
        self.with_connection(move |connection| read_vault_purpose_requests(connection, &registration_id, &capability_id))
    }
}

pub(crate) fn validate_vault_purpose_requests(
    registration: &ModuleRegistration,
    capabilities: &[String],
    requests: &[ModuleVaultPurposeRequestV1],
) -> Result<(), StoreError> {
    let capabilities = capabilities.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let mut seen = BTreeSet::new();
    requests.iter().all(|request| {
        request.registration_id() == registration.registration_id()
            && capabilities.contains(request.capability_id())
            && valid_capability_ids(&[request.capability_id().to_owned()])
            && valid_identity_token(request.purpose_id())
            && request.requested_lease_ttl_seconds() > 0
            && (1..=5).contains(&request.secret_class())
            && (1..=6).contains(&request.action())
            && request.target_scope() == 1
            && seen.insert((request.capability_id(), request.purpose_id(), request.secret_class(), request.action()))
    }).then_some(()).ok_or(StoreError::InvalidModuleVaultPurposeRequest)
}

pub(crate) fn insert_vault_purpose_requests(
    connection: &Connection,
    requests: &[ModuleVaultPurposeRequestV1],
) -> Result<(), StoreError> {
    for request in requests {
        connection.execute(
            "INSERT INTO hermes_kernel_module_vault_purpose_request
             (registration_id, capability_id, purpose_id, requested_lease_ttl_seconds, secret_class, action, target_scope)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![request.registration_id(), request.capability_id(), request.purpose_id(),
                i64::from(request.requested_lease_ttl_seconds()), i64::from(request.secret_class()),
                i64::from(request.action()), i64::from(request.target_scope())],
        )?;
    }
    Ok(())
}

fn read_vault_purpose_requests(
    connection: &Connection,
    registration_id: &str,
    capability_id: &str,
) -> Result<Vec<ModuleVaultPurposeRequestV1>, StoreError> {
    let mut statement = connection.prepare(
        "SELECT purpose_id, requested_lease_ttl_seconds, secret_class, action, target_scope
         FROM hermes_kernel_module_vault_purpose_request
         WHERE registration_id = ?1 AND capability_id = ?2
         ORDER BY purpose_id, secret_class, action",
    )?;
    let rows = statement.query_map(params![registration_id, capability_id], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?, row.get::<_, i64>(2)?, row.get::<_, i64>(3)?, row.get::<_, i64>(4)?))
    })?;
    rows.map(|row| {
        let (purpose_id, ttl, secret_class, action, target_scope) = row?;
        Ok(ModuleVaultPurposeRequestV1::new(
            registration_id, capability_id, purpose_id,
            u16::try_from(ttl).map_err(|_| StoreError::InvalidModuleVaultPurposeRequest)?,
            u8::try_from(secret_class).map_err(|_| StoreError::InvalidModuleVaultPurposeRequest)?,
            u8::try_from(action).map_err(|_| StoreError::InvalidModuleVaultPurposeRequest)?,
            u8::try_from(target_scope).map_err(|_| StoreError::InvalidModuleVaultPurposeRequest)?,
        ))
    }).collect()
}
