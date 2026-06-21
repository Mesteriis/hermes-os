use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use chacha20poly1305::aead::{Aead, AeadCore, KeyInit, OsRng};
use chacha20poly1305::{Key, XChaCha20Poly1305};

use super::HostVault;
use super::constants::VAULT_VERSION;
use super::crypto::{
    derive_domain_key, master_key_from_recovery_phrase, recovery_phrase, validate_non_empty,
};
use super::errors::HostVaultError;
use super::files::write_secure_file;
use super::models::{RecoveryExportResponse, RecoveryFile, SessionKey, VaultStatus};

impl HostVault {
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
}
