//! SQLite persistence for descriptor-declared Blob quota requests.

use std::collections::BTreeSet;

use hermes_kernel_control_store::{ModuleBlobQuotaRequestV1, ModuleRegistration};
use rusqlite::{Connection, OptionalExtension, params};

use crate::{SqliteControlStore, StoreError, valid_capability_ids};

const MAX_BLOB_QUOTA_BYTES: u64 = 1 << 40;

impl SqliteControlStore {
    pub fn module_blob_quota_request(
        &self,
        registration_id: &str,
        capability_id: &str,
    ) -> Result<Option<ModuleBlobQuotaRequestV1>, StoreError> {
        let registration_id = registration_id.to_owned();
        let capability_id = capability_id.to_owned();
        self.with_connection(move |connection| {
            read_blob_quota_request(connection, &registration_id, &capability_id)
        })
    }
}

pub(crate) fn validate_blob_quota_requests(
    registration: &ModuleRegistration,
    capabilities: &[String],
    requests: &[ModuleBlobQuotaRequestV1],
) -> Result<(), StoreError> {
    let requested = capabilities
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let mut seen = BTreeSet::new();
    let valid = requests.iter().all(|request| {
        request.registration_id() == registration.registration_id()
            && request.owner_id() == registration.owner_id()
            && requested.contains(request.capability_id())
            && valid_capability_ids(&[request.capability_id().to_owned()])
            && (1..=MAX_BLOB_QUOTA_BYTES).contains(&request.max_bytes())
            && seen.insert(request.capability_id())
    });
    valid
        .then_some(())
        .ok_or(StoreError::InvalidModuleBlobQuotaRequest)
}

pub(crate) fn insert_blob_quota_requests(
    connection: &Connection,
    requests: &[ModuleBlobQuotaRequestV1],
) -> Result<(), StoreError> {
    for request in requests {
        let quota = i64::try_from(request.max_bytes())
            .map_err(|_| StoreError::InvalidModuleBlobQuotaRequest)?;
        connection.execute(
            "INSERT INTO hermes_kernel_module_blob_quota_request
             (registration_id, capability_id, owner_id, max_bytes)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                request.registration_id(),
                request.capability_id(),
                request.owner_id(),
                quota,
            ],
        )?;
    }
    Ok(())
}

fn read_blob_quota_request(
    connection: &Connection,
    registration_id: &str,
    capability_id: &str,
) -> Result<Option<ModuleBlobQuotaRequestV1>, StoreError> {
    connection
        .query_row(
            "SELECT owner_id, max_bytes
             FROM hermes_kernel_module_blob_quota_request
             WHERE registration_id = ?1 AND capability_id = ?2",
            params![registration_id, capability_id],
            |row| {
                let max_bytes = u64::try_from(row.get::<_, i64>(1)?)
                    .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(1, 0))?;
                Ok(ModuleBlobQuotaRequestV1::new(
                    registration_id,
                    capability_id,
                    row.get::<_, String>(0)?,
                    max_bytes,
                ))
            },
        )
        .optional()
        .map_err(StoreError::from)
}
