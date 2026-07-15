use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use chacha20poly1305::aead::{Aead, AeadCore, KeyInit, OsRng, Payload};
use chacha20poly1305::{Key, XChaCha20Poly1305, XNonce};
use chrono::Utc;
use rusqlite::{OptionalExtension, params};

use crate::platform::secrets::errors::SecretResolutionError;
use crate::platform::secrets::models::{ResolvedSecret, SecretReference, SecretStoreKind};
use crate::platform::secrets::resolver::{SecretResolutionFuture, SecretResolver};

use super::HostVault;
use super::constants::VAULT_VERSION;
use super::crypto::{entry_aad, validate_non_empty};
use super::errors::{HostVaultError, host_secret_store_failure};
use super::models::{SecretEntryContext, StoredVaultEntry};

impl HostVault {
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

    pub fn delete_secret(&self, secret_ref: &str) -> Result<bool, HostVaultError> {
        validate_non_empty("secret_ref", secret_ref)?;
        let deleted = self.connection()?.execute(
            r#"
            DELETE FROM vault_entries
            WHERE secret_ref = ?1
            "#,
            params![secret_ref.trim()],
        )?;
        self.delete_manifest_entry(secret_ref)?;
        Ok(deleted > 0)
    }
}

impl SecretResolver for HostVault {
    fn resolve<'a>(&'a self, reference: &'a SecretReference) -> SecretResolutionFuture<'a> {
        Box::pin(std::future::ready(self.resolve_host_secret(reference)))
    }
}
