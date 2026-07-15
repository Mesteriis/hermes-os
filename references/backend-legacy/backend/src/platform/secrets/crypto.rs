use aes_gcm::aead::rand_core::RngCore;
use aes_gcm::aead::{KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Key};
use argon2::Argon2;
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;

use super::models::ResolvedSecret;

pub(super) const VAULT_VERSION: u8 = 1;
pub(super) const VAULT_KDF: &str = "argon2id:v1";
pub(super) const SALT_LEN: usize = 16;
pub(super) const NONCE_LEN: usize = 12;

pub(super) fn random_bytes<const N: usize>() -> [u8; N] {
    let mut bytes = [0_u8; N];
    OsRng.fill_bytes(&mut bytes);
    bytes
}

pub(super) fn encrypted_vault_cipher(
    master_key: &ResolvedSecret,
    encoded_salt: &str,
) -> Result<Aes256Gcm, super::file_vault::EncryptedVaultError> {
    let salt = BASE64_STANDARD
        .decode(encoded_salt)
        .map_err(|_| super::file_vault::EncryptedVaultError::InvalidEncoding)?;
    let mut key = [0_u8; 32];
    Argon2::default()
        .hash_password_into(master_key.expose_for_runtime().as_bytes(), &salt, &mut key)
        .map_err(|_| super::file_vault::EncryptedVaultError::Crypto)?;

    Ok(Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key)))
}

pub(super) fn database_vault_cipher(
    master_key: &ResolvedSecret,
    encoded_salt: &str,
) -> Result<Aes256Gcm, super::database_vault::DatabaseEncryptedVaultError> {
    let salt = BASE64_STANDARD
        .decode(encoded_salt)
        .map_err(|_| super::database_vault::DatabaseEncryptedVaultError::InvalidEncoding)?;
    let mut key = [0_u8; 32];
    Argon2::default()
        .hash_password_into(master_key.expose_for_runtime().as_bytes(), &salt, &mut key)
        .map_err(|_| super::database_vault::DatabaseEncryptedVaultError::Crypto)?;

    Ok(Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key)))
}
