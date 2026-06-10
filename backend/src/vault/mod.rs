use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use chacha20poly1305::aead::{Aead, AeadCore, KeyInit, OsRng, Payload};
use chacha20poly1305::{Key, XChaCha20Poly1305, XNonce};
use chrono::Utc;
use hkdf::Hkdf;
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};
use thiserror::Error;
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::platform::secrets::{
    ResolvedSecret, SecretReference, SecretResolutionError, SecretResolutionFuture, SecretResolver,
    SecretStoreKind,
};

const VAULT_VERSION: u16 = 1;
const MIN_ENTROPY_EVENTS: usize = 2_000;
const MASTER_KEY_LEN: usize = 32;
const SERVICE_NAME: &str = "hermes-hub";
const KEYCHAIN_USER: &str = "host-vault-master-key";

#[derive(Clone)]
pub struct HostVault {
    home: PathBuf,
    dev_mode: bool,
    dev_key_path: PathBuf,
    state: Arc<Mutex<HostVaultState>>,
    entropy: Arc<Mutex<Vec<EntropyEvent>>>,
}

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

    pub fn store_secret(
        &self,
        secret_ref: &str,
        value: &str,
        context: SecretEntryContext<'_>,
    ) -> Result<(), HostVaultError> {
        validate_non_empty("secret_ref", secret_ref)?;
        validate_non_empty("secret value", value)?;
        validate_non_empty("entry_kind", context.entry_kind)?;
        validate_non_empty("account_id", context.account_id)?;
        validate_non_empty("purpose", context.purpose)?;

        let key = self.domain_key(b"encryption")?;
        let cipher = XChaCha20Poly1305::new(Key::from_slice(&key));
        let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
        let aad = entry_aad(secret_ref, context);
        let ciphertext = cipher
            .encrypt(
                &nonce,
                Payload {
                    msg: value.as_bytes(),
                    aad: aad.as_bytes(),
                },
            )
            .map_err(|_| HostVaultError::Crypto)?;
        let now = Utc::now().to_rfc3339();
        let connection = self.connection()?;
        connection.execute(
            r#"
            INSERT INTO vault_entries (
                secret_ref, entry_kind, account_id, purpose, version, nonce, ciphertext, aad, created_at, updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            ON CONFLICT(secret_ref)
            DO UPDATE SET
                entry_kind = excluded.entry_kind,
                account_id = excluded.account_id,
                purpose = excluded.purpose,
                version = excluded.version,
                nonce = excluded.nonce,
                ciphertext = excluded.ciphertext,
                aad = excluded.aad,
                updated_at = excluded.updated_at
            "#,
            params![
                secret_ref.trim(),
                context.entry_kind.trim(),
                context.account_id.trim(),
                context.purpose.trim(),
                VAULT_VERSION,
                BASE64_STANDARD.encode(nonce.as_slice()),
                BASE64_STANDARD.encode(ciphertext),
                aad,
                now,
                now
            ],
        )?;
        self.upsert_manifest_entry(secret_ref, context)?;
        Ok(())
    }

    pub fn resolve_host_secret(
        &self,
        reference: &SecretReference,
    ) -> Result<ResolvedSecret, SecretResolutionError> {
        if reference.store_kind != SecretStoreKind::HostVault {
            return Err(SecretResolutionError::UnsupportedStoreKind(
                reference.store_kind.as_str().to_owned(),
            ));
        }
        let value = self
            .read_secret(&reference.secret_ref)
            .map_err(host_secret_store_failure)?;
        ResolvedSecret::new(value)
    }

    pub fn read_secret(&self, secret_ref: &str) -> Result<String, HostVaultError> {
        validate_non_empty("secret_ref", secret_ref)?;
        let connection = self.connection()?;
        let row = connection
            .query_row(
                r#"
                SELECT version, nonce, ciphertext, aad
                FROM vault_entries
                WHERE secret_ref = ?1
                "#,
                params![secret_ref.trim()],
                |row| {
                    Ok(StoredVaultEntry {
                        version: row.get(0)?,
                        nonce: row.get(1)?,
                        ciphertext: row.get(2)?,
                        aad: row.get(3)?,
                    })
                },
            )
            .optional()?
            .ok_or_else(|| HostVaultError::MissingSecret {
                secret_ref: secret_ref.trim().to_owned(),
            })?;
        if row.version != VAULT_VERSION {
            return Err(HostVaultError::UnsupportedVaultVersion(row.version));
        }
        let key = self.domain_key(b"encryption")?;
        let cipher = XChaCha20Poly1305::new(Key::from_slice(&key));
        let nonce = BASE64_STANDARD
            .decode(row.nonce)
            .map_err(|_| HostVaultError::InvalidEncoding)?;
        let ciphertext = BASE64_STANDARD
            .decode(row.ciphertext)
            .map_err(|_| HostVaultError::InvalidEncoding)?;
        let plaintext = cipher
            .decrypt(
                XNonce::from_slice(&nonce),
                Payload {
                    msg: &ciphertext,
                    aad: row.aad.as_bytes(),
                },
            )
            .map_err(|_| HostVaultError::Crypto)?;

        String::from_utf8(plaintext).map_err(|_| HostVaultError::InvalidEncoding)
    }

    pub fn export_recovery(&self) -> Result<RecoveryExportResponse, HostVaultError> {
        let key = self.current_master_key()?;
        let phrase = recovery_phrase(&key)?;
        let recovery_key = derive_domain_key(&key, b"recovery")?;
        let cipher = XChaCha20Poly1305::new(Key::from_slice(&recovery_key));
        let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
        let ciphertext = cipher
            .encrypt(&nonce, key.as_slice())
            .map_err(|_| HostVaultError::Crypto)?;
        let file = RecoveryFile {
            version: VAULT_VERSION,
            nonce: BASE64_STANDARD.encode(nonce.as_slice()),
            ciphertext: BASE64_STANDARD.encode(ciphertext),
        };
        let path = self.recovery_file_path();
        write_secure_file(&path, &serde_json::to_vec_pretty(&file)?)?;
        Ok(RecoveryExportResponse {
            path,
            recovery_phrase: phrase,
        })
    }

    pub fn import_recovery(&self, recovery_phrase: &str) -> Result<VaultStatus, HostVaultError> {
        validate_non_empty("recovery_phrase", recovery_phrase)?;
        let master_key = master_key_from_recovery_phrase(recovery_phrase)?;
        self.store_master_key(&master_key)?;
        self.set_unlocked(SessionKey::new(master_key))?;
        self.write_vault_check()?;
        self.status()
    }

    fn initialize_database(&self) -> Result<(), HostVaultError> {
        let connection = self.connection()?;
        connection.execute_batch(
            r#"
            PRAGMA journal_mode = WAL;
            PRAGMA foreign_keys = ON;

            CREATE TABLE IF NOT EXISTS vault_entries (
                secret_ref TEXT PRIMARY KEY,
                entry_kind TEXT NOT NULL,
                account_id TEXT NOT NULL,
                purpose TEXT NOT NULL,
                version INTEGER NOT NULL,
                nonce TEXT NOT NULL,
                ciphertext TEXT NOT NULL,
                aad TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS account_secret_manifest (
                secret_ref TEXT PRIMARY KEY,
                entry_kind TEXT NOT NULL,
                account_id TEXT NOT NULL,
                purpose TEXT NOT NULL,
                secret_kind TEXT NOT NULL,
                store_kind TEXT NOT NULL,
                label TEXT NOT NULL,
                metadata TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS vault_check (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                version INTEGER NOT NULL,
                nonce TEXT NOT NULL,
                ciphertext TEXT NOT NULL,
                aad TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            "#,
        )?;
        ensure_secure_file(&self.database_path())?;
        Ok(())
    }

    fn connection(&self) -> Result<Connection, HostVaultError> {
        Ok(Connection::open(self.database_path())?)
    }

    fn database_path(&self) -> PathBuf {
        self.home.join("vault.db")
    }

    fn recovery_file_path(&self) -> PathBuf {
        self.home.join("hermes-recovery.key")
    }

    fn upsert_manifest_entry(
        &self,
        secret_ref: &str,
        context: SecretEntryContext<'_>,
    ) -> Result<(), HostVaultError> {
        let metadata = serde_json::to_string(&context.metadata)?;
        let now = Utc::now().to_rfc3339();
        self.connection()?.execute(
            r#"
            INSERT INTO account_secret_manifest (
                secret_ref, entry_kind, account_id, purpose, secret_kind, store_kind, label, metadata, updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, 'host_vault', ?6, ?7, ?8)
            ON CONFLICT(secret_ref)
            DO UPDATE SET
                entry_kind = excluded.entry_kind,
                account_id = excluded.account_id,
                purpose = excluded.purpose,
                secret_kind = excluded.secret_kind,
                store_kind = excluded.store_kind,
                label = excluded.label,
                metadata = excluded.metadata,
                updated_at = excluded.updated_at
            "#,
            params![
                secret_ref.trim(),
                context.entry_kind.trim(),
                context.account_id.trim(),
                context.purpose.trim(),
                context.secret_kind,
                context.label.trim(),
                metadata,
                now
            ],
        )?;
        Ok(())
    }

    fn write_vault_check(&self) -> Result<(), HostVaultError> {
        let key = self.domain_key(b"integrity")?;
        let cipher = XChaCha20Poly1305::new(Key::from_slice(&key));
        let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
        let aad = format!("vault_check:v{VAULT_VERSION}");
        let ciphertext = cipher
            .encrypt(
                &nonce,
                Payload {
                    msg: b"hermes-host-vault",
                    aad: aad.as_bytes(),
                },
            )
            .map_err(|_| HostVaultError::Crypto)?;
        self.connection()?.execute(
            r#"
            INSERT INTO vault_check (id, version, nonce, ciphertext, aad, updated_at)
            VALUES (1, ?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(id)
            DO UPDATE SET
                version = excluded.version,
                nonce = excluded.nonce,
                ciphertext = excluded.ciphertext,
                aad = excluded.aad,
                updated_at = excluded.updated_at
            "#,
            params![
                VAULT_VERSION,
                BASE64_STANDARD.encode(nonce.as_slice()),
                BASE64_STANDARD.encode(ciphertext),
                aad,
                Utc::now().to_rfc3339()
            ],
        )?;
        Ok(())
    }

    fn read_vault_check(&self) -> Result<(), HostVaultError> {
        let Some(row) = self
            .connection()?
            .query_row(
                "SELECT version, nonce, ciphertext, aad FROM vault_check WHERE id = 1",
                [],
                |row| {
                    Ok(StoredVaultEntry {
                        version: row.get(0)?,
                        nonce: row.get(1)?,
                        ciphertext: row.get(2)?,
                        aad: row.get(3)?,
                    })
                },
            )
            .optional()?
        else {
            return Ok(());
        };
        if row.version != VAULT_VERSION {
            return Err(HostVaultError::UnsupportedVaultVersion(row.version));
        }
        let key = self.domain_key(b"integrity")?;
        let cipher = XChaCha20Poly1305::new(Key::from_slice(&key));
        let nonce = BASE64_STANDARD
            .decode(row.nonce)
            .map_err(|_| HostVaultError::InvalidEncoding)?;
        let ciphertext = BASE64_STANDARD
            .decode(row.ciphertext)
            .map_err(|_| HostVaultError::InvalidEncoding)?;
        cipher
            .decrypt(
                XNonce::from_slice(&nonce),
                Payload {
                    msg: &ciphertext,
                    aad: row.aad.as_bytes(),
                },
            )
            .map_err(|_| HostVaultError::Crypto)?;
        Ok(())
    }

    fn domain_key(&self, label: &[u8]) -> Result<[u8; MASTER_KEY_LEN], HostVaultError> {
        let key = self.current_master_key()?;
        derive_domain_key(&key, label)
    }

    fn current_master_key(&self) -> Result<[u8; MASTER_KEY_LEN], HostVaultError> {
        let state = self
            .state
            .lock()
            .map_err(|_| HostVaultError::StatePoisoned)?;
        match &*state {
            HostVaultState::Unlocked(key) => Ok(key.bytes),
            HostVaultState::Locked => Err(HostVaultError::Locked),
        }
    }

    fn set_unlocked(&self, key: SessionKey) -> Result<(), HostVaultError> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| HostVaultError::StatePoisoned)?;
        *state = HostVaultState::Unlocked(key);
        Ok(())
    }

    fn is_unlocked(&self) -> Result<bool, HostVaultError> {
        let state = self
            .state
            .lock()
            .map_err(|_| HostVaultError::StatePoisoned)?;
        Ok(matches!(*state, HostVaultState::Unlocked(_)))
    }

    fn has_stored_master_key(&self) -> Result<bool, HostVaultError> {
        if self.dev_mode {
            return Ok(self.dev_key_path.exists());
        }
        #[cfg(target_os = "macos")]
        {
            Ok(keyring_entry()?.get_password().is_ok())
        }
        #[cfg(not(target_os = "macos"))]
        {
            Err(HostVaultError::UnsupportedPlatform)
        }
    }

    fn store_master_key(&self, master_key: &[u8; MASTER_KEY_LEN]) -> Result<(), HostVaultError> {
        let encoded = BASE64_STANDARD.encode(master_key);
        if self.dev_mode {
            write_secure_file(&self.dev_key_path, encoded.as_bytes())?;
            return Ok(());
        }
        #[cfg(target_os = "macos")]
        {
            keyring_entry()?.set_password(&encoded)?;
            Ok(())
        }
        #[cfg(not(target_os = "macos"))]
        {
            Err(HostVaultError::UnsupportedPlatform)
        }
    }

    fn load_master_key(&self) -> Result<[u8; MASTER_KEY_LEN], HostVaultError> {
        let encoded = if self.dev_mode {
            fs::read_to_string(&self.dev_key_path)?
        } else {
            #[cfg(target_os = "macos")]
            {
                keyring_entry()?.get_password()?
            }
            #[cfg(not(target_os = "macos"))]
            {
                return Err(HostVaultError::UnsupportedPlatform);
            }
        };
        decode_master_key(&encoded)
    }
}

impl SecretResolver for HostVault {
    fn resolve<'a>(&'a self, reference: &'a SecretReference) -> SecretResolutionFuture<'a> {
        Box::pin(std::future::ready(self.resolve_host_secret(reference)))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HostVaultConfig {
    pub home: PathBuf,
    pub dev_mode: bool,
    pub dev_key_path: PathBuf,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VaultMode {
    Uninitialized,
    Locked,
    Unlocked,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VaultStatus {
    pub state: VaultMode,
    pub needs_entropy: bool,
    pub needs_biometric: bool,
    pub needs_recovery: bool,
    pub version: u16,
    pub recoverable: bool,
    pub entropy_progress: u8,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EntropyEvent {
    pub x: f64,
    pub y: f64,
    pub dx: f64,
    pub dy: f64,
    pub timestamp_ms: f64,
    pub velocity: f64,
    pub acceleration: f64,
    pub interval_ms: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct SecretEntryContext<'a> {
    pub entry_kind: &'a str,
    pub account_id: &'a str,
    pub purpose: &'a str,
    pub secret_kind: &'a str,
    pub label: &'a str,
    pub metadata: &'a serde_json::Value,
}

#[derive(Clone, Debug, Serialize)]
pub struct RecoveryExportResponse {
    pub path: PathBuf,
    pub recovery_phrase: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct RecoveryFile {
    version: u16,
    nonce: String,
    ciphertext: String,
}

#[derive(Debug)]
struct StoredVaultEntry {
    version: u16,
    nonce: String,
    ciphertext: String,
    aad: String,
}

#[derive(Zeroize, ZeroizeOnDrop)]
struct SessionKey {
    bytes: [u8; MASTER_KEY_LEN],
}

impl SessionKey {
    fn new(bytes: [u8; MASTER_KEY_LEN]) -> Self {
        Self { bytes }
    }
}

enum HostVaultState {
    Locked,
    Unlocked(SessionKey),
}

fn derive_master_key(
    os_random: &[u8],
    entropy: &[EntropyEvent],
) -> Result<[u8; MASTER_KEY_LEN], HostVaultError> {
    let entropy_json = serde_json::to_vec(entropy)?;
    let mut hasher = Sha512::new();
    hasher.update(os_random);
    hasher.update(&entropy_json);
    hasher.update(
        Utc::now()
            .timestamp_nanos_opt()
            .unwrap_or_default()
            .to_be_bytes(),
    );
    let digest = hasher.finalize();
    let hkdf = Hkdf::<sha2::Sha256>::new(None, &digest);
    let mut key = [0_u8; MASTER_KEY_LEN];
    hkdf.expand(b"hermes-host-vault:master:v1", &mut key)
        .map_err(|_| HostVaultError::Crypto)?;
    Ok(key)
}

fn derive_domain_key(
    master_key: &[u8; MASTER_KEY_LEN],
    label: &[u8],
) -> Result<[u8; MASTER_KEY_LEN], HostVaultError> {
    let hkdf = Hkdf::<sha2::Sha256>::new(None, master_key);
    let mut key = [0_u8; MASTER_KEY_LEN];
    let mut info = b"hermes-host-vault:v1:".to_vec();
    info.extend_from_slice(label);
    hkdf.expand(&info, &mut key)
        .map_err(|_| HostVaultError::Crypto)?;
    Ok(key)
}

fn entry_aad(secret_ref: &str, context: SecretEntryContext<'_>) -> String {
    format!(
        "v={VAULT_VERSION};ref={};kind={};account_id={};purpose={};secret_kind={}",
        secret_ref.trim(),
        context.entry_kind.trim(),
        context.account_id.trim(),
        context.purpose.trim(),
        context.secret_kind.trim()
    )
}

fn recovery_phrase(master_key: &[u8; MASTER_KEY_LEN]) -> Result<String, HostVaultError> {
    Ok(master_key
        .chunks(2)
        .map(|chunk| format!("{:02x}{:02x}", chunk[0], chunk[1]))
        .collect::<Vec<_>>()
        .join(" "))
}

fn master_key_from_recovery_phrase(phrase: &str) -> Result<[u8; MASTER_KEY_LEN], HostVaultError> {
    let compact = phrase.split_whitespace().collect::<String>();
    if compact.len() != MASTER_KEY_LEN * 2 {
        return Err(HostVaultError::InvalidRecoveryPhrase);
    }
    let mut key = [0_u8; MASTER_KEY_LEN];
    for index in 0..MASTER_KEY_LEN {
        let byte = u8::from_str_radix(&compact[index * 2..index * 2 + 2], 16)
            .map_err(|_| HostVaultError::InvalidRecoveryPhrase)?;
        key[index] = byte;
    }
    Ok(key)
}

fn decode_master_key(encoded: &str) -> Result<[u8; MASTER_KEY_LEN], HostVaultError> {
    let decoded = BASE64_STANDARD
        .decode(encoded.trim())
        .map_err(|_| HostVaultError::InvalidEncoding)?;
    decoded
        .try_into()
        .map_err(|_| HostVaultError::InvalidEncoding)
}

fn entropy_progress(events: usize) -> u8 {
    ((events.min(MIN_ENTROPY_EVENTS) * 100) / MIN_ENTROPY_EVENTS) as u8
}

fn validate_non_empty(field: &'static str, value: &str) -> Result<(), HostVaultError> {
    if value.trim().is_empty() {
        return Err(HostVaultError::EmptyField(field));
    }
    Ok(())
}

fn write_secure_file(path: &Path, bytes: &[u8]) -> Result<(), HostVaultError> {
    if let Some(parent) = path.parent() {
        ensure_secure_dir(parent)?;
    }
    let temp_path = path.with_extension("tmp");
    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&temp_path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    ensure_secure_file(&temp_path)?;
    fs::rename(&temp_path, path)?;
    ensure_secure_file(path)?;
    Ok(())
}

fn ensure_secure_dir(path: &Path) -> Result<(), HostVaultError> {
    fs::create_dir_all(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}

fn ensure_secure_file(path: &Path) -> Result<(), HostVaultError> {
    if !path.exists() {
        return Ok(());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

fn guard_release_dev_mode(dev_mode: bool) -> Result<(), HostVaultError> {
    if dev_mode && !cfg!(debug_assertions) {
        return Err(HostVaultError::DevModeForbiddenInRelease);
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn keyring_entry() -> Result<keyring::Entry, HostVaultError> {
    Ok(keyring::Entry::new(SERVICE_NAME, KEYCHAIN_USER)?)
}

fn host_secret_store_failure(error: HostVaultError) -> SecretResolutionError {
    SecretResolutionError::StoreFailure {
        message: error.public_message(),
    }
}

#[derive(Debug, Error)]
pub enum HostVaultError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[cfg(target_os = "macos")]
    #[error(transparent)]
    Keyring(#[from] keyring::Error),

    #[error("host vault is not initialized")]
    Uninitialized,

    #[error("host vault is already initialized")]
    AlreadyInitialized,

    #[error("host vault is locked")]
    Locked,

    #[error("host vault state is poisoned")]
    StatePoisoned,

    #[error("insufficient vault entropy: collected {collected}, required {required}")]
    InsufficientEntropy { collected: usize, required: usize },

    #[error("entropy batch must not be empty")]
    EmptyEntropyBatch,

    #[error("host vault cryptographic operation failed")]
    Crypto,

    #[error("host vault random generation failed")]
    Random,

    #[error("invalid host vault encoding")]
    InvalidEncoding,

    #[error("invalid host vault recovery phrase")]
    InvalidRecoveryPhrase,

    #[error("unsupported host vault version: {0}")]
    UnsupportedVaultVersion(u16),

    #[error("secret was not found in host vault: {secret_ref}")]
    MissingSecret { secret_ref: String },

    #[error("host vault dev mode is forbidden in release builds")]
    DevModeForbiddenInRelease,

    #[error("host vault release runtime is macOS-only")]
    UnsupportedPlatform,

    #[error("{0} must not be empty")]
    EmptyField(&'static str),
}

impl HostVaultError {
    fn public_message(&self) -> String {
        match self {
            Self::Crypto => "invalid host vault key or corrupted encrypted payload".to_owned(),
            Self::InvalidEncoding => "invalid host vault encoding".to_owned(),
            Self::InvalidRecoveryPhrase => "invalid host vault recovery phrase".to_owned(),
            Self::Locked => "host vault is locked".to_owned(),
            Self::Uninitialized => "host vault is not initialized".to_owned(),
            Self::MissingSecret { secret_ref } => format!("secret was not found: {secret_ref}"),
            Self::EmptyField(field) => format!("{field} must not be empty"),
            Self::InsufficientEntropy {
                collected,
                required,
            } => {
                format!("insufficient entropy: collected {collected}, required {required}")
            }
            Self::AlreadyInitialized => "host vault is already initialized".to_owned(),
            Self::EmptyEntropyBatch => "entropy batch must not be empty".to_owned(),
            Self::UnsupportedVaultVersion(_) => "unsupported host vault version".to_owned(),
            Self::DevModeForbiddenInRelease => {
                "host vault dev mode is forbidden in release".to_owned()
            }
            Self::UnsupportedPlatform => "host vault release runtime is macOS-only".to_owned(),
            Self::Io(_) | Self::Sqlite(_) | Self::Json(_) | Self::StatePoisoned | Self::Random => {
                "host vault operation failed".to_owned()
            }
            #[cfg(target_os = "macos")]
            Self::Keyring(_) => "macOS Keychain operation failed".to_owned(),
        }
    }
}

pub fn default_vault_home(home_dir: &Path) -> PathBuf {
    home_dir.join(".hermes").join("vault")
}

pub fn default_dev_key_path(home_dir: &Path) -> PathBuf {
    home_dir.join(".hermes").join("dev").join("master.key")
}
