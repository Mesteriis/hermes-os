use chrono::Utc;
use rusqlite::params;

use super::HostVault;
use super::crypto::validate_non_empty;
use super::errors::HostVaultError;
use super::models::{HostVaultManifestEntry, SecretEntryContext};

impl HostVault {
    pub fn account_secret_manifest(&self) -> Result<Vec<HostVaultManifestEntry>, HostVaultError> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            r#"
            SELECT secret_ref, entry_kind, account_id, purpose, secret_kind, store_kind, label, metadata, updated_at
            FROM account_secret_manifest
            ORDER BY account_id ASC, purpose ASC, secret_ref ASC
            "#,
        )?;
        let mut rows = statement.query([])?;
        let mut entries = Vec::new();
        while let Some(row) = rows.next()? {
            let metadata: String = row.get("metadata")?;
            entries.push(HostVaultManifestEntry {
                secret_ref: row.get("secret_ref")?,
                entry_kind: row.get("entry_kind")?,
                account_id: row.get("account_id")?,
                purpose: row.get("purpose")?,
                secret_kind: row.get("secret_kind")?,
                store_kind: row.get("store_kind")?,
                label: row.get("label")?,
                metadata: serde_json::from_str(&metadata)?,
                updated_at: row.get("updated_at")?,
            });
        }
        Ok(entries)
    }

    pub fn upsert_account_secret_manifest_entry(
        &self,
        secret_ref: &str,
        context: SecretEntryContext<'_>,
    ) -> Result<(), HostVaultError> {
        validate_non_empty("secret_ref", secret_ref)?;
        validate_non_empty("entry_kind", context.entry_kind)?;
        validate_non_empty("account_id", context.account_id)?;
        validate_non_empty("purpose", context.purpose)?;
        self.upsert_manifest_entry(secret_ref, context)
    }

    pub(super) fn upsert_manifest_entry(
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
}
