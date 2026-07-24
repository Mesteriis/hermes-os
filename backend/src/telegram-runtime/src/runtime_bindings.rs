use std::fs::{self, DirBuilder};
use std::os::unix::fs::{DirBuilderExt, MetadataExt};
use std::path::{Path, PathBuf};

use hermes_runtime_protocol::v1::{ManagedIntegrationRuntimeConfigurationV1, RuntimeArtifactUseV1};
use hermes_secure_file::{SecureReadPolicy, read};
use hermes_telegram_runtime::admission::{
    TELEGRAM_STATE_LAYOUT_REVISION_V1, TELEGRAM_TDJSON_ARTIFACT_ID,
};
use sha2::{Digest, Sha256};

const TDLIB_STATE_DIRECTORY_V1: &str = "tdlib-v1";
const MAX_TDJSON_ARTIFACT_BYTES: u64 = 512 * 1024 * 1024;

pub(crate) struct TelegramRuntimeBindingsV1 {
    tdjson_artifact_path: PathBuf,
    database_directory: PathBuf,
}

impl TelegramRuntimeBindingsV1 {
    pub(crate) fn tdjson_artifact_path(&self) -> &Path {
        &self.tdjson_artifact_path
    }

    #[cfg(test)]
    pub(crate) fn database_directory(&self) -> &Path {
        &self.database_directory
    }

    pub(crate) fn into_database_directory(self) -> PathBuf {
        self.database_directory
    }
}

pub(crate) fn resolve(
    configuration: &ManagedIntegrationRuntimeConfigurationV1,
) -> Result<TelegramRuntimeBindingsV1, String> {
    let artifact = configuration
        .runtime_artifacts
        .binary_search_by(|candidate| {
            candidate
                .artifact_id
                .as_str()
                .cmp(TELEGRAM_TDJSON_ARTIFACT_ID)
        })
        .ok()
        .map(|index| &configuration.runtime_artifacts[index])
        .ok_or_else(invalid_bindings)?;
    if RuntimeArtifactUseV1::try_from(artifact.r#use)
        .ok()
        .is_none_or(|value| value != RuntimeArtifactUseV1::NativeDynamicLibrary)
        || artifact.size_bytes == 0
        || artifact.size_bytes > MAX_TDJSON_ARTIFACT_BYTES
        || artifact.sha256.len() != 32
    {
        return Err(invalid_bindings());
    }
    let tdjson_artifact_path = PathBuf::from(&artifact.staged_path);
    let bytes = read(
        &tdjson_artifact_path,
        SecureReadPolicy::owner_private(artifact.size_bytes),
    )
    .map_err(|_| invalid_bindings())?;
    if bytes.len() as u64 != artifact.size_bytes
        || Sha256::digest(&bytes).as_slice() != artifact.sha256.as_slice()
    {
        return Err(invalid_bindings());
    }

    let state_root = configuration
        .integration_state_root
        .as_ref()
        .filter(|root| {
            root.state_generation != 0
                && root.state_layout_revision == TELEGRAM_STATE_LAYOUT_REVISION_V1
        })
        .ok_or_else(invalid_bindings)?;
    let database_directory = prepare_database_directory(Path::new(&state_root.root_path))?;
    Ok(TelegramRuntimeBindingsV1 {
        tdjson_artifact_path,
        database_directory,
    })
}

fn prepare_database_directory(root: &Path) -> Result<PathBuf, String> {
    if !root.is_absolute() {
        return Err(invalid_bindings());
    }
    validate_private_directory(root)?;
    let canonical_root = fs::canonicalize(root).map_err(|_| invalid_bindings())?;
    let database_directory = canonical_root.join(TDLIB_STATE_DIRECTORY_V1);
    match DirBuilder::new().mode(0o700).create(&database_directory) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(_) => return Err(invalid_bindings()),
    }
    validate_private_directory(&database_directory)?;
    let canonical_database =
        fs::canonicalize(&database_directory).map_err(|_| invalid_bindings())?;
    if !canonical_database.starts_with(&canonical_root) || canonical_database == canonical_root {
        return Err(invalid_bindings());
    }
    Ok(canonical_database)
}

fn validate_private_directory(path: &Path) -> Result<(), String> {
    let metadata = fs::symlink_metadata(path).map_err(|_| invalid_bindings())?;
    if metadata.file_type().is_symlink()
        || !metadata.is_dir()
        || metadata.uid() != unsafe { libc::geteuid() }
        || metadata.mode() & 0o077 != 0
    {
        return Err(invalid_bindings());
    }
    Ok(())
}

fn invalid_bindings() -> String {
    "Telegram runtime platform bindings are invalid".to_owned()
}

#[cfg(test)]
mod tests {
    use std::fs::{self, OpenOptions};
    use std::io::Write;
    use std::os::unix::fs::{OpenOptionsExt, PermissionsExt, symlink};
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    use hermes_runtime_protocol::v1::{
        IntegrationStateRootV1, ManagedIntegrationRuntimeConfigurationV1,
        ManagedRuntimeArtifactBindingV1, RuntimeArtifactUseV1,
    };
    use sha2::{Digest, Sha256};

    use super::resolve;

    #[test]
    fn resolves_only_exact_staged_tdlib_and_private_state_root() {
        let directory = test_directory("exact");
        let artifact = write_artifact(&directory, b"exact-tdlib");
        let state_root = directory.join("state");
        fs::create_dir(&state_root).expect("create state root");
        fs::set_permissions(&state_root, fs::Permissions::from_mode(0o700))
            .expect("protect state root");
        let configuration = configuration(&artifact, b"exact-tdlib", &state_root);

        let bindings = resolve(&configuration).expect("resolve exact runtime bindings");

        assert_eq!(bindings.tdjson_artifact_path(), artifact);
        assert_eq!(
            bindings.database_directory(),
            fs::canonicalize(&state_root)
                .expect("canonical state root")
                .join("tdlib-v1")
        );
        assert!(
            bindings
                .database_directory()
                .metadata()
                .expect("state metadata")
                .is_dir()
        );
        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn rejects_tampered_artifact_and_symlinked_state_child() {
        let directory = test_directory("tampered");
        let artifact = write_artifact(&directory, b"exact-tdlib");
        let state_root = directory.join("state");
        fs::create_dir(&state_root).expect("create state root");
        fs::set_permissions(&state_root, fs::Permissions::from_mode(0o700))
            .expect("protect state root");
        let configuration = configuration(&artifact, b"exact-tdlib", &state_root);
        fs::set_permissions(&artifact, fs::Permissions::from_mode(0o700))
            .expect("make test artifact writable");
        fs::write(&artifact, b"tampered-tdlib").expect("tamper staged artifact");

        assert!(resolve(&configuration).is_err());

        fs::write(&artifact, b"exact-tdlib").expect("restore staged artifact");
        fs::set_permissions(&artifact, fs::Permissions::from_mode(0o500))
            .expect("protect staged artifact");
        let outside = directory.join("outside");
        fs::create_dir(&outside).expect("create outside state");
        symlink(&outside, state_root.join("tdlib-v1")).expect("symlink state child");
        assert!(resolve(&configuration).is_err());
        let _ = fs::remove_dir_all(directory);
    }

    fn configuration(
        artifact: &Path,
        artifact_bytes: &[u8],
        state_root: &Path,
    ) -> ManagedIntegrationRuntimeConfigurationV1 {
        ManagedIntegrationRuntimeConfigurationV1 {
            runtime_artifacts: vec![ManagedRuntimeArtifactBindingV1 {
                artifact_id: "telegram.tdjson.v1".to_owned(),
                r#use: RuntimeArtifactUseV1::NativeDynamicLibrary as i32,
                staged_path: artifact.display().to_string(),
                size_bytes: artifact_bytes.len() as u64,
                sha256: Sha256::digest(artifact_bytes).to_vec(),
            }],
            integration_state_root: Some(IntegrationStateRootV1 {
                root_path: state_root.display().to_string(),
                state_generation: 1,
                state_layout_revision: 1,
            }),
            ..Default::default()
        }
    }

    fn write_artifact(directory: &Path, bytes: &[u8]) -> PathBuf {
        fs::create_dir(directory).expect("create test directory");
        fs::set_permissions(directory, fs::Permissions::from_mode(0o700))
            .expect("protect test directory");
        let path = directory.join("libtdjson.dylib");
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o500)
            .open(&path)
            .expect("create staged artifact");
        file.write_all(bytes).expect("write staged artifact");
        path
    }

    fn test_directory(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "hermes-telegram-runtime-bindings-{label}-{}-{nonce}",
            std::process::id()
        ))
    }
}
