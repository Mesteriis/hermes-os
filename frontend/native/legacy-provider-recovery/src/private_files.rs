use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::error::{LegacyProviderRecoveryErrorV1, LegacyProviderRecoveryResultV1};

pub(crate) const MANIFEST_FILE: &str = "manifest.v1.json";
pub(crate) const CATALOG_FILE: &str = "catalog.v1.json";
pub(crate) const VAULT_FILE: &str = "vault.db";
pub(crate) const VAULT_MASTER_KEY_FILE: &str = "legacy-vault-master-key.v1";
pub(crate) const PROVIDER_CONFIGURATION_FILE: &str = "legacy-provider-config.v1";
pub(crate) const GOOGLE_OAUTH_FILE: &str = "google-oauth-client.v1.json";
pub(crate) const BUNDLE_FILES: [&str; 6] = [
    CATALOG_FILE,
    GOOGLE_OAUTH_FILE,
    PROVIDER_CONFIGURATION_FILE,
    VAULT_FILE,
    VAULT_MASTER_KEY_FILE,
    MANIFEST_FILE,
];
pub(crate) const MANIFEST_DATA_FILES: [&str; 5] = [
    CATALOG_FILE,
    GOOGLE_OAUTH_FILE,
    PROVIDER_CONFIGURATION_FILE,
    VAULT_FILE,
    VAULT_MASTER_KEY_FILE,
];

const MAX_JSON_BYTES: u64 = 1024 * 1024;
const MAX_VAULT_BYTES: u64 = 128 * 1024 * 1024;

pub(crate) fn validate_bundle_root(root: &Path) -> LegacyProviderRecoveryResultV1<PathBuf> {
    if !root.is_absolute() {
        return Err(LegacyProviderRecoveryErrorV1::InvalidBundle);
    }
    let metadata =
        fs::symlink_metadata(root).map_err(|_| LegacyProviderRecoveryErrorV1::InvalidBundle)?;
    if !metadata.file_type().is_dir() || metadata.file_type().is_symlink() {
        return Err(LegacyProviderRecoveryErrorV1::InvalidBundle);
    }
    require_private_permissions(&metadata)?;
    let canonical =
        fs::canonicalize(root).map_err(|_| LegacyProviderRecoveryErrorV1::InvalidBundle)?;
    let actual = fs::read_dir(&canonical)
        .map_err(|_| LegacyProviderRecoveryErrorV1::InvalidBundle)?
        .map(|entry| {
            entry
                .map_err(|_| LegacyProviderRecoveryErrorV1::InvalidBundle)
                .and_then(|entry| {
                    entry
                        .file_name()
                        .into_string()
                        .map_err(|_| LegacyProviderRecoveryErrorV1::InvalidBundle)
                })
        })
        .collect::<LegacyProviderRecoveryResultV1<BTreeSet<_>>>()?;
    let expected = BUNDLE_FILES
        .into_iter()
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    if actual != expected {
        return Err(LegacyProviderRecoveryErrorV1::InvalidBundle);
    }
    Ok(canonical)
}

pub(crate) fn read_private_file(
    root: &Path,
    relative_path: &str,
) -> LegacyProviderRecoveryResultV1<Vec<u8>> {
    let maximum = if relative_path == VAULT_FILE {
        MAX_VAULT_BYTES
    } else {
        MAX_JSON_BYTES
    };
    if !BUNDLE_FILES.contains(&relative_path) {
        return Err(LegacyProviderRecoveryErrorV1::InvalidBundle);
    }
    let path = root.join(relative_path);
    let metadata =
        fs::symlink_metadata(&path).map_err(|_| LegacyProviderRecoveryErrorV1::InvalidBundle)?;
    if !metadata.file_type().is_file()
        || metadata.file_type().is_symlink()
        || metadata.len() == 0
        || metadata.len() > maximum
    {
        return Err(LegacyProviderRecoveryErrorV1::InvalidBundle);
    }
    require_private_permissions(&metadata)?;
    fs::read(path).map_err(|_| LegacyProviderRecoveryErrorV1::IoUnavailable)
}

pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>()
}

pub(crate) fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

pub(crate) fn is_generation(value: &str) -> bool {
    value.len() == 32
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

fn require_private_permissions(metadata: &fs::Metadata) -> LegacyProviderRecoveryResultV1<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o077 != 0 {
            return Err(LegacyProviderRecoveryErrorV1::InvalidBundle);
        }
    }
    Ok(())
}

#[cfg(any(feature = "prepare", test))]
pub(crate) fn set_private_permissions(path: &Path, mode: u32) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(mode))
            .expect("set private test permissions");
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::*;

    #[test]
    fn accepts_only_the_exact_private_regular_file_inventory() {
        let temporary = tempdir().expect("create temporary directory");
        let bundle = temporary.path().join("bundle");
        fs::create_dir(&bundle).expect("create bundle directory");
        set_private_permissions(&bundle, 0o700);
        for relative_path in BUNDLE_FILES {
            let path = bundle.join(relative_path);
            fs::write(&path, b"fixture").expect("write bundle fixture");
            set_private_permissions(&path, 0o600);
        }
        assert_eq!(
            validate_bundle_root(&bundle).expect("accept exact private bundle"),
            fs::canonicalize(&bundle).expect("canonicalize fixture")
        );

        let extra = bundle.join("unexpected");
        fs::write(&extra, b"fixture").expect("write unexpected file");
        set_private_permissions(&extra, 0o600);
        assert_eq!(
            validate_bundle_root(&bundle),
            Err(LegacyProviderRecoveryErrorV1::InvalidBundle)
        );
        fs::remove_file(&extra).expect("remove unexpected fixture");

        let missing = bundle.join(GOOGLE_OAUTH_FILE);
        fs::remove_file(&missing).expect("remove expected fixture");
        assert_eq!(
            validate_bundle_root(&bundle),
            Err(LegacyProviderRecoveryErrorV1::InvalidBundle)
        );
        fs::write(&missing, b"fixture").expect("restore expected fixture");
        set_private_permissions(&missing, 0o600);

        set_private_permissions(&bundle.join(CATALOG_FILE), 0o644);
        assert_eq!(
            read_private_file(&bundle, CATALOG_FILE),
            Err(LegacyProviderRecoveryErrorV1::InvalidBundle)
        );
        set_private_permissions(&bundle.join(CATALOG_FILE), 0o600);

        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;

            let manifest = bundle.join(MANIFEST_FILE);
            fs::remove_file(&manifest).expect("remove manifest fixture");
            symlink(bundle.join(CATALOG_FILE), &manifest).expect("create manifest symlink");
            assert_eq!(
                read_private_file(&bundle, MANIFEST_FILE),
                Err(LegacyProviderRecoveryErrorV1::InvalidBundle)
            );
        }
    }
}
