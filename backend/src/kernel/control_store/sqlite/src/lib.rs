//! SQLite-backed private Kernel Control Store adapter.

use std::fs::{File, OpenOptions};
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use hermes_kernel_control_store::{ControlStore, ExternalRuntimeAttestation, GrantSet, InitialOwnerIdentity, ModuleRegistration, ModuleRegistrationState, SettingsApplyState, SettingsDesiredSnapshot, SettingsSchemaBinding};
use rusqlite::{Connection, OptionalExtension, backup::Backup, params};
use sha2::{Digest, Sha256};

mod schema;

use schema::{SCHEMA_VERSION, migrate_schema};

#[derive(Debug)]
pub enum StoreError {
    Sqlite(rusqlite::Error),
    Io(std::io::Error),
    MissingMetadata,
    UnsupportedSchema(i64),
    InvalidGeneration,
    RecoveryFenceOverflow,
    InstallationIdentityMismatch,
    InvalidExportDestination,
    IntegrityCheckFailed(String),
    InitialOwnerAlreadyClaimed,
    InvalidInitialOwnerIdentity,
    InvalidModuleRegistration,
    ModuleRegistrationAlreadyExists,
    ModuleRegistrationMissing,
    InvalidModuleRegistrationTransition,
    InvalidCapabilityGrant,
    InvalidExternalRuntimeAttestation,
    StaleExternalRuntimeAttestation,
    InvalidSettingsSchemaBinding,
    SettingsSchemaRevisionCollision,
    SettingsRevisionConflict,
    InvalidSettingsApplyState,
}

impl From<rusqlite::Error> for StoreError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Sqlite(error)
    }
}

impl From<std::io::Error> for StoreError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

pub struct ControlStoreExport {
    instance_id: String,
    generation: u64,
    sha256: [u8; 32],
    sha256_hex: String,
}

impl ControlStoreExport {
    #[must_use]
    pub fn instance_id(&self) -> &str {
        &self.instance_id
    }

    #[must_use]
    pub fn generation(&self) -> u64 {
        self.generation
    }

    #[must_use]
    pub fn sha256_hex(&self) -> &str {
        &self.sha256_hex
    }

    #[must_use]
    pub fn sha256_bytes(&self) -> &[u8; 32] {
        &self.sha256
    }
}

pub struct SqliteControlStore {
    path: PathBuf,
    snapshot: ControlStore,
}

impl SqliteControlStore {
    pub fn create(path: &Path, instance_id: &str, generation: u64) -> Result<Self, StoreError> {
        let generation = i64::try_from(generation).map_err(|_| StoreError::InvalidGeneration)?;
        let connection = Connection::open(path)?;
        configure_writable(&connection)?;
        connection.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS hermes_kernel_control_store_metadata (
                singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
                schema_version INTEGER NOT NULL,
                instance_id TEXT NOT NULL,
                generation INTEGER NOT NULL CHECK (generation >= 1),
                identity_epoch INTEGER NOT NULL CHECK (identity_epoch >= 1),
                grant_epoch INTEGER NOT NULL CHECK (grant_epoch >= 1)
            ) STRICT;
            CREATE TABLE IF NOT EXISTS hermes_kernel_initial_owner_identity (
                singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
                owner_id TEXT NOT NULL,
                device_id TEXT NOT NULL,
                public_key_sec1 BLOB NOT NULL CHECK (length(public_key_sec1) = 65)
            ) STRICT;
            CREATE TABLE IF NOT EXISTS hermes_kernel_module_registration (
                registration_id TEXT PRIMARY KEY,
                module_id TEXT NOT NULL,
                owner_id TEXT NOT NULL,
                descriptor_sha256 BLOB NOT NULL CHECK (length(descriptor_sha256) = 32),
                state TEXT NOT NULL CHECK (state IN ('pending', 'approved', 'suspended', 'revoked', 'blocked_incompatible')),
                grant_epoch INTEGER NOT NULL CHECK (grant_epoch >= 1)
            ) STRICT;
            CREATE TABLE IF NOT EXISTS hermes_kernel_module_registration_capability (
                registration_id TEXT NOT NULL REFERENCES hermes_kernel_module_registration(registration_id) ON DELETE CASCADE,
                capability_id TEXT NOT NULL,
                approved INTEGER NOT NULL CHECK (approved IN (0, 1)),
                PRIMARY KEY (registration_id, capability_id)
            ) STRICT;
            CREATE TABLE IF NOT EXISTS hermes_kernel_external_runtime_attestation (
                registration_id TEXT PRIMARY KEY REFERENCES hermes_kernel_module_registration(registration_id) ON DELETE CASCADE,
                runtime_id TEXT NOT NULL,
                runtime_generation INTEGER NOT NULL CHECK (runtime_generation >= 1),
                grant_epoch INTEGER NOT NULL CHECK (grant_epoch >= 1),
                distribution_sha256 BLOB NOT NULL CHECK (length(distribution_sha256) = 32)
            ) STRICT;
            CREATE TABLE IF NOT EXISTS hermes_kernel_settings_schema_binding (
                registration_id TEXT PRIMARY KEY REFERENCES hermes_kernel_module_registration(registration_id) ON DELETE CASCADE,
                schema_major INTEGER NOT NULL CHECK (schema_major >= 1), schema_revision INTEGER NOT NULL CHECK (schema_revision >= 1),
                schema_sha256 BLOB NOT NULL CHECK (length(schema_sha256) = 32), desired_revision INTEGER NOT NULL CHECK (desired_revision >= 0), effective_revision INTEGER NOT NULL CHECK (effective_revision >= 0),
                apply_state TEXT NOT NULL CHECK (apply_state IN ('current', 'pending_validation', 'pending_apply', 'applying', 'awaiting_external_restart', 'blocked_config')),
                sanitized_reason_code TEXT
            ) STRICT;
            CREATE TABLE IF NOT EXISTS hermes_kernel_settings_desired_snapshot (registration_id TEXT PRIMARY KEY REFERENCES hermes_kernel_settings_schema_binding(registration_id) ON DELETE CASCADE, revision INTEGER NOT NULL CHECK (revision >= 1), snapshot_bytes BLOB NOT NULL) STRICT;
            ",
        )?;
        connection.execute(
            "INSERT INTO hermes_kernel_control_store_metadata \
             (singleton, schema_version, instance_id, generation, identity_epoch, grant_epoch) \
             VALUES (1, ?1, ?2, ?3, 1, 1)",
            params![SCHEMA_VERSION, instance_id, generation],
        )?;
        Ok(Self {
            path: path.to_owned(),
            snapshot: ControlStore::new(instance_id, generation as u64),
        })
    }

    pub fn open(path: &Path) -> Result<Self, StoreError> {
        let connection = Connection::open(path)?;
        configure_writable(&connection)?;
        validate_quick_check(&connection)?;
        for _ in 0..SCHEMA_VERSION {
            let before = connection.query_row("SELECT schema_version FROM hermes_kernel_control_store_metadata WHERE singleton = 1", [], |row| row.get::<_, i64>(0)).optional()?.ok_or(StoreError::MissingMetadata)?;
            migrate_schema(&connection)?;
            if before == SCHEMA_VERSION { break; }
        }
        let metadata = connection
            .query_row(
                "SELECT schema_version, instance_id, generation, identity_epoch, grant_epoch \
                 FROM hermes_kernel_control_store_metadata WHERE singleton = 1",
                [],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, i64>(2)?,
                        row.get::<_, i64>(3)?,
                        row.get::<_, i64>(4)?,
                    ))
                },
            )
            .optional()?
            .ok_or(StoreError::MissingMetadata)?;

        if metadata.0 != SCHEMA_VERSION {
            return Err(StoreError::UnsupportedSchema(metadata.0));
        }

        Ok(Self {
            path: path.to_owned(),
            snapshot: ControlStore::with_recovery_fences(
                metadata.1,
                u64::try_from(metadata.2).map_err(|_| StoreError::InvalidGeneration)?,
                u64::try_from(metadata.3).map_err(|_| StoreError::InvalidGeneration)?,
                u64::try_from(metadata.4).map_err(|_| StoreError::InvalidGeneration)?,
            ),
        })
    }

    #[must_use]
    pub fn snapshot(&self) -> &ControlStore {
        &self.snapshot
    }

    pub fn export_to(&self, destination: &Path) -> Result<ControlStoreExport, StoreError> {
        if destination == self.path {
            return Err(StoreError::InvalidExportDestination);
        }

        let parent = destination
            .parent()
            .ok_or(StoreError::InvalidExportDestination)?;
        let temporary = create_export_temporary_file(parent, destination)?;
        let export_result = self.write_export_to(&temporary);
        if let Err(error) = export_result {
            let _ = std::fs::remove_file(&temporary);
            return Err(error);
        }

        File::open(&temporary)?.sync_all()?;
        std::fs::rename(&temporary, destination)?;
        File::open(parent)?.sync_all()?;

        let digest: [u8; 32] = Sha256::digest(std::fs::read(destination)?).into();
        let sha256_hex = digest.iter().map(|byte| format!("{byte:02x}")).collect();
        Ok(ControlStoreExport {
            instance_id: self.snapshot.instance_id().to_owned(),
            generation: self.snapshot.generation(),
            sha256: digest,
            sha256_hex,
        })
    }

    pub fn advance_recovery_fences(&self) -> Result<ControlStore, StoreError> {
        let next_generation = self
            .snapshot
            .generation()
            .checked_add(1)
            .ok_or(StoreError::RecoveryFenceOverflow)?;
        let next_identity_epoch = self
            .snapshot
            .identity_epoch()
            .checked_add(1)
            .ok_or(StoreError::RecoveryFenceOverflow)?;
        let next_grant_epoch = self
            .snapshot
            .grant_epoch()
            .checked_add(1)
            .ok_or(StoreError::RecoveryFenceOverflow)?;
        let next_generation_sql =
            i64::try_from(next_generation).map_err(|_| StoreError::RecoveryFenceOverflow)?;
        let next_identity_epoch_sql =
            i64::try_from(next_identity_epoch).map_err(|_| StoreError::RecoveryFenceOverflow)?;
        let next_grant_epoch_sql =
            i64::try_from(next_grant_epoch).map_err(|_| StoreError::RecoveryFenceOverflow)?;
        let connection = Connection::open(&self.path)?;
        configure_writable(&connection)?;
        let transaction = connection.unchecked_transaction()?;
        transaction.execute(
            "UPDATE hermes_kernel_control_store_metadata \
             SET generation = ?1, identity_epoch = ?2, grant_epoch = ?3 \
             WHERE singleton = 1",
            params![
                next_generation_sql,
                next_identity_epoch_sql,
                next_grant_epoch_sql
            ],
        )?;
        transaction.commit()?;
        Ok(ControlStore::with_recovery_fences(
            self.snapshot.instance_id(),
            next_generation,
            next_identity_epoch,
            next_grant_epoch,
        ))
    }

    pub fn initial_owner_identity(&self) -> Result<Option<InitialOwnerIdentity>, StoreError> {
        let connection = Connection::open(&self.path)?;
        connection.query_row("SELECT owner_id, device_id, public_key_sec1 FROM hermes_kernel_initial_owner_identity WHERE singleton = 1", [], |row| {
            let key: Vec<u8> = row.get(2)?;
            let public_key_sec1: [u8; 65] = key.try_into().map_err(|_| rusqlite::Error::IntegralValueOutOfRange(2, 65))?;
            Ok(InitialOwnerIdentity::new(row.get::<_, String>(0)?, row.get::<_, String>(1)?, public_key_sec1))
        }).optional().map_err(StoreError::from)
    }

    pub fn claim_initial_owner(&self, identity: &InitialOwnerIdentity) -> Result<(), StoreError> {
        if !valid_identity_token(identity.owner_id()) || !valid_identity_token(identity.device_id()) || identity.public_key_sec1()[0] != 0x04 { return Err(StoreError::InvalidInitialOwnerIdentity); }
        let connection = Connection::open(&self.path)?;
        configure_writable(&connection)?;
        let transaction = connection.unchecked_transaction()?;
        let changed = transaction.execute("INSERT OR IGNORE INTO hermes_kernel_initial_owner_identity (singleton, owner_id, device_id, public_key_sec1) VALUES (1, ?1, ?2, ?3)", params![identity.owner_id(), identity.device_id(), identity.public_key_sec1().as_slice()])?;
        if changed != 1 { return Err(StoreError::InitialOwnerAlreadyClaimed); }
        transaction.commit()?;
        Ok(())
    }

    pub fn create_pending_registration(&self, registration: &ModuleRegistration, requested_capability_ids: &[String]) -> Result<(), StoreError> {
        if registration.state() != ModuleRegistrationState::Pending || registration.grant_epoch() != 1
            || !valid_identity_token(registration.registration_id()) || !valid_identity_token(registration.module_id()) || !valid_identity_token(registration.owner_id())
            || !valid_capability_ids(requested_capability_ids) {
            return Err(StoreError::InvalidModuleRegistration);
        }
        let connection = Connection::open(&self.path)?;
        configure_writable(&connection)?;
        let transaction = connection.unchecked_transaction()?;
        let changed = transaction.execute(
            "INSERT OR IGNORE INTO hermes_kernel_module_registration (registration_id, module_id, owner_id, descriptor_sha256, state, grant_epoch) VALUES (?1, ?2, ?3, ?4, 'pending', 1)",
            params![registration.registration_id(), registration.module_id(), registration.owner_id(), registration.descriptor_sha256().as_slice()],
        )?;
        if changed != 1 { return Err(StoreError::ModuleRegistrationAlreadyExists); }
        for capability_id in requested_capability_ids {
            transaction.execute(
                "INSERT INTO hermes_kernel_module_registration_capability (registration_id, capability_id, approved) VALUES (?1, ?2, 0)",
                params![registration.registration_id(), capability_id],
            )?;
        }
        transaction.commit()?;
        Ok(())
    }

    pub fn module_registration(&self, registration_id: &str) -> Result<Option<ModuleRegistration>, StoreError> {
        let connection = Connection::open(&self.path)?;
        connection.query_row(
            "SELECT registration_id, module_id, owner_id, descriptor_sha256, state, grant_epoch FROM hermes_kernel_module_registration WHERE registration_id = ?1",
            [registration_id],
            |row| {
                let digest: Vec<u8> = row.get(3)?;
                let descriptor_sha256: [u8; 32] = digest.try_into().map_err(|_| rusqlite::Error::IntegralValueOutOfRange(3, 32))?;
                let state = module_registration_state_from_str(&row.get::<_, String>(4)?).ok_or(rusqlite::Error::InvalidQuery)?;
                let epoch = u64::try_from(row.get::<_, i64>(5)?).map_err(|_| rusqlite::Error::IntegralValueOutOfRange(5, 0))?;
                Ok(ModuleRegistration::new(row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, descriptor_sha256, state, epoch))
            },
        ).optional().map_err(StoreError::from)
    }

    pub fn transition_module_registration(&self, registration_id: &str, next: ModuleRegistrationState) -> Result<ModuleRegistration, StoreError> {
        let current = self.module_registration(registration_id)?.ok_or(StoreError::ModuleRegistrationMissing)?;
        if !current.state().can_transition_to(next) { return Err(StoreError::InvalidModuleRegistrationTransition); }
        let next_epoch = current.grant_epoch().checked_add(1).ok_or(StoreError::RecoveryFenceOverflow)?;
        let connection = Connection::open(&self.path)?;
        configure_writable(&connection)?;
        let changed = connection.execute(
            "UPDATE hermes_kernel_module_registration SET state = ?1, grant_epoch = ?2 WHERE registration_id = ?3 AND state = ?4 AND grant_epoch = ?5",
            params![next.as_str(), i64::try_from(next_epoch).map_err(|_| StoreError::RecoveryFenceOverflow)?, registration_id, current.state().as_str(), i64::try_from(current.grant_epoch()).map_err(|_| StoreError::RecoveryFenceOverflow)?],
        )?;
        if changed != 1 { return Err(StoreError::InvalidModuleRegistrationTransition); }
        Ok(ModuleRegistration::new(current.registration_id(), current.module_id(), current.owner_id(), *current.descriptor_sha256(), next, next_epoch))
    }

    pub fn approve_module_registration(&self, registration_id: &str, capability_ids: &[String]) -> Result<GrantSet, StoreError> {
        if !valid_capability_ids(capability_ids) { return Err(StoreError::InvalidCapabilityGrant); }
        let current = self.module_registration(registration_id)?.ok_or(StoreError::ModuleRegistrationMissing)?;
        if !matches!(current.state(), ModuleRegistrationState::Pending | ModuleRegistrationState::Suspended) {
            return Err(StoreError::InvalidModuleRegistrationTransition);
        }
        let connection = Connection::open(&self.path)?;
        configure_writable(&connection)?;
        let transaction = connection.unchecked_transaction()?;
        transaction.execute(
            "UPDATE hermes_kernel_module_registration_capability SET approved = 0 WHERE registration_id = ?1",
            [registration_id],
        )?;
        for capability_id in capability_ids {
            let changed = transaction.execute(
                "UPDATE hermes_kernel_module_registration_capability SET approved = 1 WHERE registration_id = ?1 AND capability_id = ?2",
                params![registration_id, capability_id],
            )?;
            if changed != 1 { return Err(StoreError::InvalidCapabilityGrant); }
        }
        let next_epoch = current.grant_epoch().checked_add(1).ok_or(StoreError::RecoveryFenceOverflow)?;
        let changed = transaction.execute(
            "UPDATE hermes_kernel_module_registration SET state = 'approved', grant_epoch = ?1 WHERE registration_id = ?2 AND state = ?3 AND grant_epoch = ?4",
            params![i64::try_from(next_epoch).map_err(|_| StoreError::RecoveryFenceOverflow)?, registration_id, current.state().as_str(), i64::try_from(current.grant_epoch()).map_err(|_| StoreError::RecoveryFenceOverflow)?],
        )?;
        if changed != 1 { return Err(StoreError::InvalidModuleRegistrationTransition); }
        let grants = read_approved_capabilities(&transaction, registration_id)?;
        transaction.commit()?;
        Ok(GrantSet::new(registration_id, next_epoch, grants))
    }

    pub fn effective_grant_set(&self, registration_id: &str) -> Result<Option<GrantSet>, StoreError> {
        let registration = self.module_registration(registration_id)?.ok_or(StoreError::ModuleRegistrationMissing)?;
        if registration.state() != ModuleRegistrationState::Approved { return Ok(None); }
        let connection = Connection::open(&self.path)?;
        Ok(Some(GrantSet::new(registration_id, registration.grant_epoch(), read_approved_capabilities(&connection, registration_id)?)))
    }

    pub fn attest_external_runtime(&self, attestation: &ExternalRuntimeAttestation) -> Result<(), StoreError> {
        if !valid_identity_token(attestation.registration_id())
            || !valid_identity_token(attestation.runtime_id())
            || attestation.runtime_generation() == 0
            || attestation.grant_epoch() == 0 {
            return Err(StoreError::InvalidExternalRuntimeAttestation);
        }
        let registration = self.module_registration(attestation.registration_id())?
            .ok_or(StoreError::ModuleRegistrationMissing)?;
        if registration.state() != ModuleRegistrationState::Approved
            || registration.grant_epoch() != attestation.grant_epoch() {
            return Err(StoreError::StaleExternalRuntimeAttestation);
        }
        let connection = Connection::open(&self.path)?;
        configure_writable(&connection)?;
        let changed = connection.execute(
            "INSERT INTO hermes_kernel_external_runtime_attestation (registration_id, runtime_id, runtime_generation, grant_epoch, distribution_sha256) VALUES (?1, ?2, ?3, ?4, ?5) ON CONFLICT(registration_id) DO UPDATE SET runtime_id = excluded.runtime_id, runtime_generation = excluded.runtime_generation, grant_epoch = excluded.grant_epoch, distribution_sha256 = excluded.distribution_sha256 WHERE excluded.runtime_generation > hermes_kernel_external_runtime_attestation.runtime_generation",
            params![attestation.registration_id(), attestation.runtime_id(), i64::try_from(attestation.runtime_generation()).map_err(|_| StoreError::InvalidExternalRuntimeAttestation)?, i64::try_from(attestation.grant_epoch()).map_err(|_| StoreError::InvalidExternalRuntimeAttestation)?, attestation.distribution_sha256().as_slice()],
        )?;
        if changed != 1 { return Err(StoreError::StaleExternalRuntimeAttestation); }
        Ok(())
    }

    pub fn effective_external_runtime_attestation(&self, registration_id: &str) -> Result<Option<ExternalRuntimeAttestation>, StoreError> {
        let registration = self.module_registration(registration_id)?.ok_or(StoreError::ModuleRegistrationMissing)?;
        if registration.state() != ModuleRegistrationState::Approved { return Ok(None); }
        let connection = Connection::open(&self.path)?;
        connection.query_row(
            "SELECT runtime_id, runtime_generation, grant_epoch, distribution_sha256 FROM hermes_kernel_external_runtime_attestation WHERE registration_id = ?1 AND grant_epoch = ?2",
            params![registration_id, i64::try_from(registration.grant_epoch()).map_err(|_| StoreError::InvalidGeneration)?],
            |row| {
                let digest: Vec<u8> = row.get(3)?;
                let distribution_sha256: [u8; 32] = digest.try_into().map_err(|_| rusqlite::Error::IntegralValueOutOfRange(3, 32))?;
                let runtime_generation = u64::try_from(row.get::<_, i64>(1)?).map_err(|_| rusqlite::Error::IntegralValueOutOfRange(1, 0))?;
                let grant_epoch = u64::try_from(row.get::<_, i64>(2)?).map_err(|_| rusqlite::Error::IntegralValueOutOfRange(2, 0))?;
                Ok(ExternalRuntimeAttestation::new(registration_id, row.get::<_, String>(0)?, runtime_generation, grant_epoch, distribution_sha256))
            },
        ).optional().map_err(StoreError::from)
    }

    pub fn register_settings_schema(&self, binding: &SettingsSchemaBinding) -> Result<(), StoreError> {
        if !valid_identity_token(binding.registration_id()) || binding.schema_major() == 0 || binding.schema_revision() == 0 || !valid_settings_binding_state(binding) { return Err(StoreError::InvalidSettingsSchemaBinding); }
        let registration = self.module_registration(binding.registration_id())?.ok_or(StoreError::ModuleRegistrationMissing)?;
        if registration.state() != ModuleRegistrationState::Approved { return Err(StoreError::InvalidSettingsSchemaBinding); }
        let connection = Connection::open(&self.path)?; configure_writable(&connection)?;
        let changed = connection.execute("INSERT INTO hermes_kernel_settings_schema_binding (registration_id, schema_major, schema_revision, schema_sha256, desired_revision, effective_revision, apply_state, sanitized_reason_code) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8) ON CONFLICT(registration_id) DO UPDATE SET schema_major=excluded.schema_major, schema_revision=excluded.schema_revision, schema_sha256=excluded.schema_sha256, desired_revision=excluded.desired_revision, effective_revision=excluded.effective_revision, apply_state=excluded.apply_state, sanitized_reason_code=excluded.sanitized_reason_code WHERE excluded.schema_major > hermes_kernel_settings_schema_binding.schema_major OR (excluded.schema_major = hermes_kernel_settings_schema_binding.schema_major AND excluded.schema_revision > hermes_kernel_settings_schema_binding.schema_revision)", params![binding.registration_id(), binding.schema_major(), binding.schema_revision(), binding.schema_sha256().as_slice(), i64::try_from(binding.desired_revision()).map_err(|_| StoreError::InvalidSettingsSchemaBinding)?, i64::try_from(binding.effective_revision()).map_err(|_| StoreError::InvalidSettingsSchemaBinding)?, binding.apply_state().as_str(), binding.sanitized_reason_code()])?;
        if changed != 1 { return Err(StoreError::SettingsSchemaRevisionCollision); }
        Ok(())
    }

    pub fn settings_schema_binding(&self, registration_id: &str) -> Result<Option<SettingsSchemaBinding>, StoreError> {
        let connection = Connection::open(&self.path)?;
        connection.query_row("SELECT schema_major, schema_revision, schema_sha256, desired_revision, effective_revision, apply_state, sanitized_reason_code FROM hermes_kernel_settings_schema_binding WHERE registration_id=?1", [registration_id], |row| { let digest: Vec<u8> = row.get(2)?; let digest: [u8; 32] = digest.try_into().map_err(|_| rusqlite::Error::IntegralValueOutOfRange(2,32))?; let apply_state = settings_apply_state_from_str(&row.get::<_, String>(5)?).ok_or(rusqlite::Error::InvalidQuery)?; Ok(SettingsSchemaBinding::new(registration_id, row.get(0)?, row.get(1)?, digest, u64::try_from(row.get::<_, i64>(3)?).map_err(|_| rusqlite::Error::IntegralValueOutOfRange(3,0))?, u64::try_from(row.get::<_, i64>(4)?).map_err(|_| rusqlite::Error::IntegralValueOutOfRange(4,0))?, apply_state, row.get(6)?)) }).optional().map_err(StoreError::from)
    }
    pub fn commit_desired_settings_snapshot(&self, update: &SettingsDesiredSnapshot) -> Result<u64, StoreError> {
        if update.snapshot_bytes.is_empty() || update.snapshot_bytes.len() > 256 * 1024 { return Err(StoreError::InvalidSettingsSchemaBinding); }
        let mut connection = Connection::open(&self.path)?; configure_writable(&connection)?;
        let next = update.expected_revision.checked_add(1).ok_or(StoreError::RecoveryFenceOverflow)?;
        let transaction = connection.transaction()?;
        let changed = transaction.execute("UPDATE hermes_kernel_settings_schema_binding SET desired_revision=?1, apply_state='pending_validation', sanitized_reason_code=NULL WHERE registration_id=?2 AND desired_revision=?3", params![i64::try_from(next).map_err(|_| StoreError::RecoveryFenceOverflow)?, update.registration_id, i64::try_from(update.expected_revision).map_err(|_| StoreError::RecoveryFenceOverflow)?])?;
        if changed != 1 { return Err(StoreError::SettingsRevisionConflict); }
        transaction.execute("INSERT INTO hermes_kernel_settings_desired_snapshot (registration_id, revision, snapshot_bytes) VALUES (?1, ?2, ?3) ON CONFLICT(registration_id) DO UPDATE SET revision=excluded.revision, snapshot_bytes=excluded.snapshot_bytes", params![update.registration_id, i64::try_from(next).map_err(|_| StoreError::RecoveryFenceOverflow)?, update.snapshot_bytes])?;
        transaction.commit()?;
        Ok(next)
    }

    pub fn desired_settings_snapshot(&self, registration_id: &str) -> Result<Option<(u64, Vec<u8>)>, StoreError> {
        let connection = Connection::open(&self.path)?;
        connection.query_row("SELECT revision, snapshot_bytes FROM hermes_kernel_settings_desired_snapshot WHERE registration_id=?1", [registration_id], |row| {
            Ok((u64::try_from(row.get::<_, i64>(0)?).map_err(|_| rusqlite::Error::IntegralValueOutOfRange(0, 0))?, row.get(1)?))
        }).optional().map_err(StoreError::from)
    }

    pub fn transition_settings_apply_state(&self, registration_id: &str, revision: u64, next: SettingsApplyState, sanitized_reason_code: Option<&str>) -> Result<(), StoreError> {
        if !valid_identity_token(registration_id) || !valid_sanitized_reason_code(sanitized_reason_code) || next == SettingsApplyState::Current { return Err(StoreError::InvalidSettingsApplyState); }
        if next == SettingsApplyState::BlockedConfig && sanitized_reason_code.is_none() { return Err(StoreError::InvalidSettingsApplyState); }
        if next != SettingsApplyState::BlockedConfig && sanitized_reason_code.is_some() { return Err(StoreError::InvalidSettingsApplyState); }
        let mut connection = Connection::open(&self.path)?; configure_writable(&connection)?;
        let transaction = connection.transaction()?;
        let (desired_revision, effective_revision, current): (u64, u64, SettingsApplyState) = transaction.query_row("SELECT desired_revision, effective_revision, apply_state FROM hermes_kernel_settings_schema_binding WHERE registration_id=?1", [registration_id], |row| {
            let current = settings_apply_state_from_str(&row.get::<_, String>(2)?).ok_or(rusqlite::Error::InvalidQuery)?;
            Ok((u64::try_from(row.get::<_, i64>(0)?).map_err(|_| rusqlite::Error::IntegralValueOutOfRange(0, 0))?, u64::try_from(row.get::<_, i64>(1)?).map_err(|_| rusqlite::Error::IntegralValueOutOfRange(1, 0))?, current))
        }).optional()?.ok_or(StoreError::SettingsRevisionConflict)?;
        if desired_revision != revision || effective_revision >= revision || !current.can_transition_to(next) { return Err(StoreError::SettingsRevisionConflict); }
        let changed = transaction.execute("UPDATE hermes_kernel_settings_schema_binding SET apply_state=?1, sanitized_reason_code=?2 WHERE registration_id=?3 AND desired_revision=?4 AND apply_state=?5", params![next.as_str(), sanitized_reason_code, registration_id, i64::try_from(revision).map_err(|_| StoreError::RecoveryFenceOverflow)?, current.as_str()])?;
        if changed != 1 { return Err(StoreError::SettingsRevisionConflict); }
        transaction.commit()?;
        Ok(())
    }

    pub fn confirm_effective_settings_revision(&self, registration_id: &str, revision: u64) -> Result<(), StoreError> {
        let connection = Connection::open(&self.path)?; configure_writable(&connection)?;
        let changed = connection.execute("UPDATE hermes_kernel_settings_schema_binding SET effective_revision=?1, apply_state='current', sanitized_reason_code=NULL WHERE registration_id=?2 AND desired_revision=?1 AND effective_revision < ?1 AND apply_state IN ('applying', 'awaiting_external_restart')", params![i64::try_from(revision).map_err(|_| StoreError::RecoveryFenceOverflow)?, registration_id])?;
        if changed != 1 { return Err(StoreError::SettingsRevisionConflict); }
        Ok(())
    }

    pub fn restore_from(
        source: &Path,
        destination: &Path,
        expected_instance_id: &str,
    ) -> Result<ControlStore, StoreError> {
        let source_store = Self::open(source)?;
        if source_store.snapshot().instance_id() != expected_instance_id {
            return Err(StoreError::InstallationIdentityMismatch);
        }
        source_store.export_to(destination)?;
        Self::open(destination)?.advance_recovery_fences()
    }

    fn write_export_to(&self, destination: &Path) -> Result<(), StoreError> {
        let source =
            Connection::open_with_flags(&self.path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        configure_read_only(&source)?;
        let mut output = Connection::open(destination)?;
        configure_writable(&output)?;
        let backup = Backup::new(&source, &mut output)?;
        backup.run_to_completion(32, Duration::from_millis(10), None)?;
        drop(backup);
        output.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;
        drop(output);
        Ok(())
    }
}

fn valid_identity_token(value: &str) -> bool { !value.is_empty() && value.len() <= 128 && value.is_ascii() }

fn module_registration_state_from_str(value: &str) -> Option<ModuleRegistrationState> {
    match value {
        "pending" => Some(ModuleRegistrationState::Pending),
        "approved" => Some(ModuleRegistrationState::Approved),
        "suspended" => Some(ModuleRegistrationState::Suspended),
        "revoked" => Some(ModuleRegistrationState::Revoked),
        "blocked_incompatible" => Some(ModuleRegistrationState::BlockedIncompatible),
        _ => None,
    }
}

fn settings_apply_state_from_str(value: &str) -> Option<SettingsApplyState> {
    match value {
        "current" => Some(SettingsApplyState::Current),
        "pending_validation" => Some(SettingsApplyState::PendingValidation),
        "pending_apply" => Some(SettingsApplyState::PendingApply),
        "applying" => Some(SettingsApplyState::Applying),
        "awaiting_external_restart" => Some(SettingsApplyState::AwaitingExternalRestart),
        "blocked_config" => Some(SettingsApplyState::BlockedConfig),
        _ => None,
    }
}

fn valid_sanitized_reason_code(value: Option<&str>) -> bool {
    value.is_none_or(|code| !code.is_empty() && code.len() <= 128 && code.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-')))
}

fn valid_settings_binding_state(binding: &SettingsSchemaBinding) -> bool {
    binding.effective_revision() <= binding.desired_revision()
        && valid_sanitized_reason_code(binding.sanitized_reason_code())
        && match binding.apply_state() {
            SettingsApplyState::Current => binding.desired_revision() == binding.effective_revision() && binding.sanitized_reason_code().is_none(),
            SettingsApplyState::BlockedConfig => binding.sanitized_reason_code().is_some(),
            SettingsApplyState::PendingValidation | SettingsApplyState::PendingApply | SettingsApplyState::Applying | SettingsApplyState::AwaitingExternalRestart => binding.desired_revision() > binding.effective_revision() && binding.sanitized_reason_code().is_none(),
        }
}

fn valid_capability_ids(capability_ids: &[String]) -> bool {
    capability_ids.windows(2).all(|pair| pair[0] < pair[1])
        && capability_ids.iter().all(|capability_id| valid_identity_token(capability_id))
}

fn read_approved_capabilities(connection: &Connection, registration_id: &str) -> Result<Vec<String>, StoreError> {
    let mut statement = connection.prepare("SELECT capability_id FROM hermes_kernel_module_registration_capability WHERE registration_id = ?1 AND approved = 1 ORDER BY capability_id")?;
    let rows = statement.query_map([registration_id], |row| row.get::<_, String>(0))?;
    rows.collect::<Result<Vec<_>, _>>().map_err(StoreError::from)
}

fn create_export_temporary_file(parent: &Path, destination: &Path) -> Result<PathBuf, StoreError> {
    for attempt in 0..16 {
        let name = destination
            .file_name()
            .ok_or(StoreError::InvalidExportDestination)?
            .to_string_lossy();
        let temporary = parent.join(format!(".{name}.{}.{}.tmp", std::process::id(), attempt));
        match OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&temporary)
        {
            Ok(file) => {
                drop(file);
                return Ok(temporary);
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error.into()),
        }
    }
    Err(StoreError::InvalidExportDestination)
}

fn configure_writable(connection: &Connection) -> Result<(), rusqlite::Error> {
    connection.execute_batch(
        "
        PRAGMA journal_mode = DELETE;
        PRAGMA synchronous = FULL;
        PRAGMA foreign_keys = ON;
        PRAGMA trusted_schema = OFF;
        ",
    )
}

fn configure_read_only(connection: &Connection) -> Result<(), rusqlite::Error> {
    connection.execute_batch(
        "
        PRAGMA foreign_keys = ON;
        PRAGMA trusted_schema = OFF;
        ",
    )
}

fn validate_quick_check(connection: &Connection) -> Result<(), StoreError> {
    let result: String = connection.query_row("PRAGMA quick_check", [], |row| row.get(0))?;
    if result == "ok" {
        Ok(())
    } else {
        Err(StoreError::IntegrityCheckFailed(result))
    }
}
