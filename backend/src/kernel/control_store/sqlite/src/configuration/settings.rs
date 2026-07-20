//! Kernel-owned Settings Registry records.

use hermes_kernel_control_store::{
    ModuleRegistrationState, SettingsApplyState, SettingsDesiredSnapshot, SettingsSchemaBinding,
    SettingsSchemaBindingInputV1,
};
use rusqlite::{Connection, OptionalExtension, params};

use crate::module_state::registry::read_required_registration;
use crate::{
    SqliteControlStore, StoreError, settings_apply_state_from_str, valid_identity_token,
    valid_sanitized_reason_code, valid_settings_binding_state,
};

const MAX_SETTINGS_BYTES: usize = 256 * 1024;

impl SqliteControlStore {
    pub fn register_settings_schema(
        &self,
        binding: &SettingsSchemaBinding,
    ) -> Result<(), StoreError> {
        validate_binding(binding)?;
        let binding = binding.clone();
        self.with_connection(move |connection| {
            let transaction = connection.transaction()?;
            require_approved_registration(&transaction, binding.registration_id())?;
            write_schema_binding(&transaction, &binding)?;
            transaction.commit()?;
            Ok(())
        })
    }

    pub fn admit_settings_schema(
        &self,
        binding: &SettingsSchemaBinding,
        schema_bytes: &[u8],
    ) -> Result<(), StoreError> {
        validate_binding(binding)?;
        validate_bounded_bytes(schema_bytes)?;
        let binding = binding.clone();
        let schema_bytes = schema_bytes.to_vec();
        self.with_connection(move |connection| {
            let transaction = connection.transaction()?;
            require_approved_registration(&transaction, binding.registration_id())?;
            write_schema_binding(&transaction, &binding)?;
            transaction.execute(
                "INSERT INTO hermes_kernel_settings_schema_artifact (registration_id, schema_bytes)
                 VALUES (?1, ?2) ON CONFLICT(registration_id)
                 DO UPDATE SET schema_bytes=excluded.schema_bytes",
                params![binding.registration_id(), schema_bytes],
            )?;
            transaction.commit()?;
            Ok(())
        })
    }

    pub fn settings_schema_artifact(
        &self,
        registration_id: &str,
    ) -> Result<Option<Vec<u8>>, StoreError> {
        let registration_id = registration_id.to_owned();
        self.with_connection(move |connection| {
            connection
                .query_row(
                    "SELECT schema_bytes FROM hermes_kernel_settings_schema_artifact
                     WHERE registration_id=?1",
                    [&registration_id],
                    |row| row.get(0),
                )
                .optional()
                .map_err(StoreError::from)
        })
    }

    pub fn settings_schema_binding(
        &self,
        registration_id: &str,
    ) -> Result<Option<SettingsSchemaBinding>, StoreError> {
        let registration_id = registration_id.to_owned();
        self.with_connection(move |connection| read_settings_binding(connection, &registration_id))
    }

    pub fn commit_desired_settings_snapshot(
        &self,
        update: &SettingsDesiredSnapshot,
    ) -> Result<u64, StoreError> {
        validate_bounded_bytes(&update.snapshot_bytes)?;
        let update = update.clone();
        self.with_connection(move |connection| {
            let next = update
                .expected_revision
                .checked_add(1)
                .ok_or(StoreError::RecoveryFenceOverflow)?;
            let transaction = connection.transaction()?;
            let changed = transaction.execute(
                "UPDATE hermes_kernel_settings_schema_binding
                 SET desired_revision=?1, apply_state='pending_validation', sanitized_reason_code=NULL
                 WHERE registration_id=?2 AND desired_revision=?3",
                params![as_sql(next)?, update.registration_id, as_sql(update.expected_revision)?],
            )?;
            if changed != 1 {
                return Err(StoreError::SettingsRevisionConflict);
            }
            transaction.execute(
                "INSERT INTO hermes_kernel_settings_desired_snapshot
                 (registration_id, revision, snapshot_bytes) VALUES (?1, ?2, ?3)
                 ON CONFLICT(registration_id) DO UPDATE SET
                 revision=excluded.revision, snapshot_bytes=excluded.snapshot_bytes",
                params![update.registration_id, as_sql(next)?, update.snapshot_bytes],
            )?;
            transaction.commit()?;
            Ok(next)
        })
    }

    pub fn desired_settings_snapshot(
        &self,
        registration_id: &str,
    ) -> Result<Option<(u64, Vec<u8>)>, StoreError> {
        let registration_id = registration_id.to_owned();
        self.with_connection(move |connection| {
            connection
                .query_row(
                    "SELECT revision, snapshot_bytes FROM hermes_kernel_settings_desired_snapshot
                 WHERE registration_id=?1",
                    [&registration_id],
                    |row| Ok((as_u64(row.get(0)?, 0)?, row.get(1)?)),
                )
                .optional()
                .map_err(StoreError::from)
        })
    }

    pub fn transition_settings_apply_state(
        &self,
        registration_id: &str,
        revision: u64,
        next: SettingsApplyState,
        sanitized_reason_code: Option<&str>,
    ) -> Result<(), StoreError> {
        validate_apply_transition(registration_id, next, sanitized_reason_code)?;
        let registration_id = registration_id.to_owned();
        let reason = sanitized_reason_code.map(str::to_owned);
        self.with_connection(move |connection| {
            transition_apply_state(
                connection,
                &registration_id,
                revision,
                next,
                reason.as_deref(),
            )
        })
    }

    pub fn confirm_effective_settings_revision(
        &self,
        registration_id: &str,
        revision: u64,
    ) -> Result<(), StoreError> {
        let registration_id = registration_id.to_owned();
        self.with_connection(move |connection| {
            let changed = connection.execute(
                "UPDATE hermes_kernel_settings_schema_binding
                 SET effective_revision=?1, apply_state='current', sanitized_reason_code=NULL
                 WHERE registration_id=?2 AND desired_revision=?1 AND effective_revision < ?1
                 AND apply_state IN ('applying', 'awaiting_external_restart')",
                params![as_sql(revision)?, registration_id],
            )?;
            if changed == 1 {
                Ok(())
            } else {
                Err(StoreError::SettingsRevisionConflict)
            }
        })
    }
}

fn validate_binding(binding: &SettingsSchemaBinding) -> Result<(), StoreError> {
    let valid = valid_identity_token(binding.registration_id())
        && binding.schema_major() > 0
        && binding.schema_revision() > 0
        && valid_settings_binding_state(binding);
    valid
        .then_some(())
        .ok_or(StoreError::InvalidSettingsSchemaBinding)
}

fn validate_bounded_bytes(bytes: &[u8]) -> Result<(), StoreError> {
    (!bytes.is_empty() && bytes.len() <= MAX_SETTINGS_BYTES)
        .then_some(())
        .ok_or(StoreError::InvalidSettingsSchemaBinding)
}

fn require_approved_registration(
    connection: &Connection,
    registration_id: &str,
) -> Result<(), StoreError> {
    let registration = read_required_registration(connection, registration_id)?;
    if registration.state() == ModuleRegistrationState::Approved {
        Ok(())
    } else {
        Err(StoreError::InvalidSettingsSchemaBinding)
    }
}

fn write_schema_binding(
    connection: &Connection,
    binding: &SettingsSchemaBinding,
) -> Result<(), StoreError> {
    let changed = connection.execute(
        "INSERT INTO hermes_kernel_settings_schema_binding
         (registration_id, schema_major, schema_revision, schema_sha256,
          desired_revision, effective_revision, apply_state, sanitized_reason_code)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT(registration_id) DO UPDATE SET schema_major=excluded.schema_major,
         schema_revision=excluded.schema_revision, schema_sha256=excluded.schema_sha256,
         desired_revision=excluded.desired_revision, effective_revision=excluded.effective_revision,
         apply_state=excluded.apply_state, sanitized_reason_code=excluded.sanitized_reason_code
         WHERE excluded.schema_major > hermes_kernel_settings_schema_binding.schema_major
         OR (excluded.schema_major = hermes_kernel_settings_schema_binding.schema_major
             AND excluded.schema_revision > hermes_kernel_settings_schema_binding.schema_revision)",
        params![
            binding.registration_id(),
            binding.schema_major(),
            binding.schema_revision(),
            binding.schema_sha256().as_slice(),
            as_sql(binding.desired_revision())?,
            as_sql(binding.effective_revision())?,
            binding.apply_state().as_str(),
            binding.sanitized_reason_code()
        ],
    )?;
    if changed == 1 {
        Ok(())
    } else {
        Err(StoreError::SettingsSchemaRevisionCollision)
    }
}

fn read_settings_binding(
    connection: &Connection,
    registration_id: &str,
) -> Result<Option<SettingsSchemaBinding>, StoreError> {
    connection
        .query_row(
            "SELECT schema_major, schema_revision, schema_sha256, desired_revision,
         effective_revision, apply_state, sanitized_reason_code
         FROM hermes_kernel_settings_schema_binding WHERE registration_id=?1",
            [registration_id],
            |row| {
                let digest: Vec<u8> = row.get(2)?;
                let state = settings_apply_state_from_str(&row.get::<_, String>(5)?)
                    .ok_or(rusqlite::Error::InvalidQuery)?;
                Ok(SettingsSchemaBinding::new(SettingsSchemaBindingInputV1 {
                    registration_id: registration_id.to_owned(),
                    schema_major: row.get(0)?,
                    schema_revision: row.get(1)?,
                    schema_sha256: digest
                        .try_into()
                        .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(2, 32))?,
                    desired_revision: as_u64(row.get(3)?, 3)?,
                    effective_revision: as_u64(row.get(4)?, 4)?,
                    apply_state: state,
                    sanitized_reason_code: row.get(6)?,
                }))
            },
        )
        .optional()
        .map_err(StoreError::from)
}

fn validate_apply_transition(
    registration_id: &str,
    next: SettingsApplyState,
    reason: Option<&str>,
) -> Result<(), StoreError> {
    let valid = valid_identity_token(registration_id)
        && valid_sanitized_reason_code(reason)
        && next != SettingsApplyState::Current
        && (next == SettingsApplyState::BlockedConfig) == reason.is_some();
    valid
        .then_some(())
        .ok_or(StoreError::InvalidSettingsApplyState)
}

fn transition_apply_state(
    connection: &mut Connection,
    registration_id: &str,
    revision: u64,
    next: SettingsApplyState,
    reason: Option<&str>,
) -> Result<(), StoreError> {
    let transaction = connection.transaction()?;
    let (desired, effective, current) = read_apply_state(&transaction, registration_id)?;
    if desired != revision || effective >= revision || !current.can_transition_to(next) {
        return Err(StoreError::SettingsRevisionConflict);
    }
    let changed = transaction.execute(
        "UPDATE hermes_kernel_settings_schema_binding
         SET apply_state=?1, sanitized_reason_code=?2
         WHERE registration_id=?3 AND desired_revision=?4 AND apply_state=?5",
        params![
            next.as_str(),
            reason,
            registration_id,
            as_sql(revision)?,
            current.as_str()
        ],
    )?;
    if changed != 1 {
        return Err(StoreError::SettingsRevisionConflict);
    }
    transaction.commit()?;
    Ok(())
}

fn read_apply_state(
    connection: &Connection,
    registration_id: &str,
) -> Result<(u64, u64, SettingsApplyState), StoreError> {
    connection
        .query_row(
            "SELECT desired_revision, effective_revision, apply_state
         FROM hermes_kernel_settings_schema_binding WHERE registration_id=?1",
            [registration_id],
            |row| {
                let state = settings_apply_state_from_str(&row.get::<_, String>(2)?)
                    .ok_or(rusqlite::Error::InvalidQuery)?;
                Ok((as_u64(row.get(0)?, 0)?, as_u64(row.get(1)?, 1)?, state))
            },
        )
        .optional()?
        .ok_or(StoreError::SettingsRevisionConflict)
}

fn as_sql(value: u64) -> Result<i64, StoreError> {
    i64::try_from(value).map_err(|_| StoreError::RecoveryFenceOverflow)
}

fn as_u64(value: i64, index: usize) -> Result<u64, rusqlite::Error> {
    u64::try_from(value).map_err(|_| rusqlite::Error::IntegralValueOutOfRange(index, 0))
}
