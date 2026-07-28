use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use zeroize::Zeroizing;

use crate::catalog::{
    recovery_source, validate_catalog, validate_google_oauth, validate_legacy_configuration,
};
use crate::error::{LegacyProviderRecoveryErrorV1, LegacyProviderRecoveryResultV1};
use crate::legacy_vault;
use crate::model::{
    GoogleOauthClientV1, LegacyProviderConfigurationV1, LegacyProviderRecoveryCountsV1,
    LegacyProviderRecoverySecretPurposeV1, LegacyProviderRecoverySourceV1,
    RECOVERY_SCHEMA_REVISION, RecoveryBundleManifestV1, RecoveryCatalogCandidateV1,
    RecoveryCatalogV1,
};
use crate::private_files::{
    CATALOG_FILE, GOOGLE_OAUTH_FILE, MANIFEST_DATA_FILES, MANIFEST_FILE,
    PROVIDER_CONFIGURATION_FILE, VAULT_FILE, VAULT_MASTER_KEY_FILE, is_generation, is_sha256,
    read_private_file, sha256_hex, validate_bundle_root,
};

pub struct LegacyProviderRecoveryBundleV1 {
    root: PathBuf,
    fingerprint_sha256: String,
    catalog: RecoveryCatalogV1,
    legacy_master_key: Zeroizing<[u8; 32]>,
    telegram_api_hash: Zeroizing<String>,
    sources: BTreeMap<String, LegacyProviderRecoverySourceV1>,
}

impl LegacyProviderRecoveryBundleV1 {
    pub fn open(root: &Path) -> LegacyProviderRecoveryResultV1<Self> {
        let root = validate_bundle_root(root)?;
        let manifest_bytes = read_private_file(&root, MANIFEST_FILE)?;
        let manifest: RecoveryBundleManifestV1 = serde_json::from_slice(&manifest_bytes)
            .map_err(|_| LegacyProviderRecoveryErrorV1::InvalidBundle)?;
        validate_manifest(&root, &manifest)?;

        let catalog: RecoveryCatalogV1 = read_json(&root, CATALOG_FILE)?;
        let legacy: LegacyProviderConfigurationV1 = read_json(&root, PROVIDER_CONFIGURATION_FILE)?;
        let google: GoogleOauthClientV1 = read_json(&root, GOOGLE_OAUTH_FILE)?;
        let legacy_master_key = legacy_vault::decode_master_key_file(&read_private_file(
            &root,
            VAULT_MASTER_KEY_FILE,
        )?)?;
        validate_catalog(&catalog)?;
        validate_legacy_configuration(&legacy)?;
        validate_google_oauth(&google)?;
        if catalog.source_generation != manifest.source_generation
            || catalog.counts != manifest.counts
            || manifest.catalog_row_count != 5
        {
            return Err(LegacyProviderRecoveryErrorV1::InvalidBundle);
        }

        let mut sources = BTreeMap::new();
        for candidate in &catalog.candidates {
            let source = recovery_source(candidate, &legacy, &google)?;
            if sources.insert(source.handle().to_owned(), source).is_some() {
                return Err(LegacyProviderRecoveryErrorV1::InvalidCatalog);
            }
        }
        let telegram_api_hash = legacy.telegram_api_hash;
        Ok(Self {
            root,
            fingerprint_sha256: sha256_hex(&manifest_bytes),
            catalog,
            legacy_master_key,
            telegram_api_hash,
            sources,
        })
    }

    pub fn fingerprint_sha256(&self) -> &str {
        &self.fingerprint_sha256
    }

    pub fn counts(&self) -> &LegacyProviderRecoveryCountsV1 {
        &self.catalog.counts
    }

    pub fn sources(&self) -> impl ExactSizeIterator<Item = &LegacyProviderRecoverySourceV1> {
        self.sources.values()
    }

    pub fn source(
        &self,
        handle: &str,
    ) -> LegacyProviderRecoveryResultV1<&LegacyProviderRecoverySourceV1> {
        self.sources
            .get(handle)
            .ok_or(LegacyProviderRecoveryErrorV1::InvalidSource)
    }

    pub fn assert_unchanged(&self) -> LegacyProviderRecoveryResultV1<()> {
        let current = Self::open(&self.root)?;
        if current.fingerprint_sha256 != self.fingerprint_sha256 {
            return Err(LegacyProviderRecoveryErrorV1::SourceChanged);
        }
        Ok(())
    }

    pub fn resolve_secret(
        &self,
        handle: &str,
        purpose: LegacyProviderRecoverySecretPurposeV1,
    ) -> LegacyProviderRecoveryResultV1<Zeroizing<Vec<u8>>> {
        self.assert_unchanged()?;
        let candidate = self.candidate(handle)?;
        match purpose {
            LegacyProviderRecoverySecretPurposeV1::IcloudImapPassword
                if matches!(
                    candidate.kind,
                    crate::model::LegacyProviderCandidateKindV1::Icloud
                ) =>
            {
                legacy_vault::decrypt_candidate_secret(
                    &self.root.join(VAULT_FILE),
                    candidate,
                    &self.legacy_master_key,
                )
            }
            LegacyProviderRecoverySecretPurposeV1::TelegramApiHash
                if matches!(
                    candidate.kind,
                    crate::model::LegacyProviderCandidateKindV1::TelegramUser
                ) =>
            {
                Ok(Zeroizing::new(self.telegram_api_hash.as_bytes().to_vec()))
            }
            LegacyProviderRecoverySecretPurposeV1::GeneratedTelegramSessionStoreKey
                if matches!(
                    candidate.kind,
                    crate::model::LegacyProviderCandidateKindV1::TelegramUser
                ) =>
            {
                let mut key = Zeroizing::new(vec![0_u8; 32]);
                getrandom::getrandom(&mut key)
                    .map_err(|_| LegacyProviderRecoveryErrorV1::CryptographyUnavailable)?;
                Ok(key)
            }
            _ => Err(LegacyProviderRecoveryErrorV1::InvalidSecret),
        }
    }

    fn candidate(
        &self,
        handle: &str,
    ) -> LegacyProviderRecoveryResultV1<&RecoveryCatalogCandidateV1> {
        self.catalog
            .candidates
            .iter()
            .find(|candidate| candidate.source_account_digest_sha256 == handle)
            .ok_or(LegacyProviderRecoveryErrorV1::InvalidSource)
    }
}

fn read_json<T>(root: &Path, relative_path: &str) -> LegacyProviderRecoveryResultV1<T>
where
    T: for<'de> serde::Deserialize<'de>,
{
    serde_json::from_slice(&read_private_file(root, relative_path)?)
        .map_err(|_| LegacyProviderRecoveryErrorV1::InvalidBundle)
}

fn validate_manifest(
    root: &Path,
    manifest: &RecoveryBundleManifestV1,
) -> LegacyProviderRecoveryResultV1<()> {
    if manifest.schema_revision != RECOVERY_SCHEMA_REVISION
        || manifest.created_at_unix_seconds == 0
        || !is_generation(&manifest.source_generation)
        || !manifest.counts.is_exact()
        || manifest.files.len() != MANIFEST_DATA_FILES.len()
    {
        return Err(LegacyProviderRecoveryErrorV1::InvalidBundle);
    }
    let declared = manifest
        .files
        .iter()
        .map(|file| (file.relative_path.as_str(), file))
        .collect::<BTreeMap<_, _>>();
    if declared.len() != MANIFEST_DATA_FILES.len()
        || MANIFEST_DATA_FILES
            .iter()
            .any(|relative| !declared.contains_key(relative))
    {
        return Err(LegacyProviderRecoveryErrorV1::InvalidBundle);
    }
    for relative_path in MANIFEST_DATA_FILES {
        let declared = declared
            .get(relative_path)
            .ok_or(LegacyProviderRecoveryErrorV1::InvalidBundle)?;
        if !is_sha256(&declared.sha256) {
            return Err(LegacyProviderRecoveryErrorV1::InvalidBundle);
        }
        let bytes = read_private_file(root, relative_path)?;
        if declared.size_bytes != bytes.len() as u64 || declared.sha256 != sha256_hex(&bytes) {
            return Err(LegacyProviderRecoveryErrorV1::SourceChanged);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;
    use zeroize::Zeroizing;

    use super::*;
    use crate::model::{
        LegacyProviderCandidateKindV1, LegacyProviderConfigurationV1, RecoveryBundleFileV1,
        RecoveryCatalogConfigurationV1, RecoveryCatalogSecretBindingV1,
    };
    use crate::private_files::set_private_permissions;

    #[test]
    fn rejects_data_changed_after_manifest_creation() {
        let temporary = tempdir().expect("create temporary directory");
        let bundle_root = temporary.path().join("bundle");
        write_bundle(&bundle_root);
        LegacyProviderRecoveryBundleV1::open(&bundle_root).expect("open exact test bundle");

        let catalog_path = bundle_root.join(CATALOG_FILE);
        let mut changed = fs::read(&catalog_path).expect("read catalog fixture");
        changed.push(b' ');
        fs::write(&catalog_path, changed).expect("change catalog fixture");
        set_private_permissions(&catalog_path, 0o600);
        assert!(matches!(
            LegacyProviderRecoveryBundleV1::open(&bundle_root),
            Err(LegacyProviderRecoveryErrorV1::SourceChanged)
        ));
    }

    fn write_bundle(root: &Path) {
        fs::create_dir(root).expect("create bundle fixture");
        set_private_permissions(root, 0o700);
        let catalog = RecoveryCatalogV1 {
            schema_revision: RECOVERY_SCHEMA_REVISION,
            source_generation: "a".repeat(32),
            counts: LegacyProviderRecoveryCountsV1::exact(),
            candidates: vec![
                candidate(
                    LegacyProviderCandidateKindV1::Gmail,
                    "1",
                    RecoveryCatalogConfigurationV1::Gmail {
                        oauth_client_id: "public-client".to_owned(),
                    },
                    "oauth_token",
                ),
                candidate(
                    LegacyProviderCandidateKindV1::Icloud,
                    "2",
                    RecoveryCatalogConfigurationV1::Icloud {
                        imap_host: "imap.mail.me.com".to_owned(),
                        imap_port: 993,
                        tls: true,
                        username: "owner@example.test".to_owned(),
                    },
                    "imap_password",
                ),
                candidate(
                    LegacyProviderCandidateKindV1::TelegramUser,
                    "3",
                    RecoveryCatalogConfigurationV1::TelegramUser,
                    "telegram_session_key",
                ),
            ],
        };
        write_json(root, CATALOG_FILE, &catalog);
        write_json(
            root,
            PROVIDER_CONFIGURATION_FILE,
            &LegacyProviderConfigurationV1 {
                schema_revision: RECOVERY_SCHEMA_REVISION,
                telegram_api_id: 123,
                telegram_api_hash: Zeroizing::new("api-hash".to_owned()),
            },
        );
        write_json(
            root,
            GOOGLE_OAUTH_FILE,
            &GoogleOauthClientV1 {
                schema_revision: RECOVERY_SCHEMA_REVISION,
                client_id: "public-client".to_owned(),
                redirect_uris: vec!["http://127.0.0.1:8080/callback".to_owned()],
            },
        );
        write_file(root, VAULT_FILE, b"sqlite-fixture");
        write_file(
            root,
            VAULT_MASTER_KEY_FILE,
            b"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=\n",
        );
        let files = MANIFEST_DATA_FILES
            .iter()
            .map(|relative_path| {
                let bytes = fs::read(root.join(relative_path)).expect("read manifest input");
                RecoveryBundleFileV1 {
                    relative_path: (*relative_path).to_owned(),
                    size_bytes: bytes.len() as u64,
                    sha256: sha256_hex(&bytes),
                }
            })
            .collect();
        write_json(
            root,
            MANIFEST_FILE,
            &RecoveryBundleManifestV1 {
                schema_revision: RECOVERY_SCHEMA_REVISION,
                created_at_unix_seconds: 1,
                source_generation: catalog.source_generation,
                files,
                catalog_row_count: 5,
                counts: catalog.counts,
            },
        );
    }

    fn candidate(
        kind: LegacyProviderCandidateKindV1,
        digest_digit: &str,
        configuration: RecoveryCatalogConfigurationV1,
        purpose: &str,
    ) -> RecoveryCatalogCandidateV1 {
        RecoveryCatalogCandidateV1 {
            kind,
            source_account_digest_sha256: digest_digit.repeat(64),
            account_id: format!("{digest_digit}-account"),
            display_name: format!("{kind:?}"),
            external_account_id: "owner@example.test".to_owned(),
            configuration,
            legacy_secret: RecoveryCatalogSecretBindingV1 {
                purpose: purpose.to_owned(),
                secret_ref: format!("secret:{digest_digit}"),
                secret_kind: "opaque".to_owned(),
                store_kind: "host_vault".to_owned(),
            },
        }
    }

    fn write_json<T: serde::Serialize>(root: &Path, relative_path: &str, value: &T) {
        let bytes = serde_json::to_vec(value).expect("serialize fixture");
        write_file(root, relative_path, &bytes);
    }

    fn write_file(root: &Path, relative_path: &str, bytes: &[u8]) {
        let path = root.join(relative_path);
        fs::write(&path, bytes).expect("write bundle fixture");
        set_private_permissions(&path, 0o600);
    }
}
