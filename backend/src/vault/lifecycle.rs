use std::sync::{Arc, Mutex};

use zeroize::Zeroize;

use super::HostVault;
use super::constants::{MASTER_KEY_LEN, MIN_ENTROPY_EVENTS, VAULT_VERSION};
use super::crypto::{derive_domain_key, derive_master_key, entropy_progress};
use super::errors::HostVaultError;
use super::files::{ensure_secure_dir, guard_release_dev_mode};
use super::models::{
    EntropyEvent, HostVaultConfig, HostVaultState, SessionKey, VaultMode, VaultStatus,
};

impl HostVault {
    pub fn new(config: HostVaultConfig) -> Result<Self, HostVaultError> {
        guard_release_dev_mode(config.dev_mode)?;
        ensure_secure_dir(&config.home)?;
        if let Some(parent) = config.dev_key_path.parent() {
            ensure_secure_dir(parent)?;
        }

        let vault = Self {
            home: config.home,
            dev_mode: config.dev_mode,
            dev_key_path: config.dev_key_path,
            state: Arc::new(Mutex::new(HostVaultState::Locked)),
            entropy: Arc::new(Mutex::new(Vec::new())),
        };
        vault.initialize_database()?;
        Ok(vault)
    }

    pub fn status(&self) -> Result<VaultStatus, HostVaultError> {
        let initialized = self.has_stored_master_key()?;
        let state = if !initialized {
            VaultMode::Uninitialized
        } else if self.is_unlocked()? {
            VaultMode::Unlocked
        } else {
            VaultMode::Locked
        };
        let entropy_events = self
            .entropy
            .lock()
            .map_err(|_| HostVaultError::StatePoisoned)?
            .len();

        Ok(VaultStatus {
            state,
            needs_entropy: !initialized && entropy_events < MIN_ENTROPY_EVENTS,
            needs_biometric: initialized && !self.dev_mode,
            needs_recovery: !initialized,
            version: VAULT_VERSION,
            recoverable: self.recovery_file_path().exists(),
            entropy_progress: entropy_progress(entropy_events),
        })
    }

    pub fn collect_entropy(
        &self,
        events: Vec<EntropyEvent>,
    ) -> Result<VaultStatus, HostVaultError> {
        if events.is_empty() {
            return Err(HostVaultError::EmptyEntropyBatch);
        }
        let mut entropy = self
            .entropy
            .lock()
            .map_err(|_| HostVaultError::StatePoisoned)?;
        entropy.extend(events);
        drop(entropy);
        self.status()
    }

    pub fn create(&self) -> Result<VaultStatus, HostVaultError> {
        if self.has_stored_master_key()? {
            return Err(HostVaultError::AlreadyInitialized);
        }
        let entropy = self
            .entropy
            .lock()
            .map_err(|_| HostVaultError::StatePoisoned)?;
        if entropy.len() < MIN_ENTROPY_EVENTS {
            return Err(HostVaultError::InsufficientEntropy {
                collected: entropy.len(),
                required: MIN_ENTROPY_EVENTS,
            });
        }

        let mut os_random = [0_u8; 64];
        getrandom::getrandom(&mut os_random).map_err(|_| HostVaultError::Random)?;
        let mut master_key = derive_master_key(&os_random, &entropy)?;
        drop(entropy);

        self.store_master_key(&master_key)?;
        self.set_unlocked(SessionKey::new(master_key))?;
        master_key.zeroize();
        self.write_vault_check()?;
        self.status()
    }

    pub fn unlock(&self) -> Result<VaultStatus, HostVaultError> {
        if !self.has_stored_master_key()? {
            return Err(HostVaultError::Uninitialized);
        }
        let master_key = self.load_master_key()?;
        self.set_unlocked(SessionKey::new(master_key))?;
        self.read_vault_check()?;
        self.status()
    }

    pub fn unlock_existing(&self) -> Result<VaultStatus, HostVaultError> {
        if !self.has_stored_master_key()? {
            return self.status();
        }
        self.unlock()
    }

    pub fn lock(&self) -> Result<VaultStatus, HostVaultError> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| HostVaultError::StatePoisoned)?;
        *state = HostVaultState::Locked;
        drop(state);
        self.status()
    }

    pub(super) fn domain_key(&self, label: &[u8]) -> Result<[u8; MASTER_KEY_LEN], HostVaultError> {
        let key = self.current_master_key()?;
        derive_domain_key(&key, label)
    }

    pub(super) fn current_master_key(&self) -> Result<[u8; MASTER_KEY_LEN], HostVaultError> {
        let state = self
            .state
            .lock()
            .map_err(|_| HostVaultError::StatePoisoned)?;
        match &*state {
            HostVaultState::Unlocked(key) => Ok(key.bytes),
            HostVaultState::Locked => Err(HostVaultError::Locked),
        }
    }

    pub(super) fn set_unlocked(&self, key: SessionKey) -> Result<(), HostVaultError> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| HostVaultError::StatePoisoned)?;
        *state = HostVaultState::Unlocked(key);
        Ok(())
    }

    pub(super) fn is_unlocked(&self) -> Result<bool, HostVaultError> {
        let state = self
            .state
            .lock()
            .map_err(|_| HostVaultError::StatePoisoned)?;
        Ok(matches!(*state, HostVaultState::Unlocked(_)))
    }
}
