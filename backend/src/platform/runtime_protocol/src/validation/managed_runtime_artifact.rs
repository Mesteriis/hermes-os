//! Structural validation shared by managed integration, workflow and engine bootstrap.

use std::{
    collections::BTreeSet,
    path::{Component, Path},
};

use crate::v1::{ManagedRuntimeArtifactBindingV1, RuntimeArtifactUseV1};

const MAX_IDENTIFIER_BYTES: usize = 128;
const MAX_PRIVATE_PATH_BYTES: usize = 4_096;
const MAX_RUNTIME_ARTIFACTS: usize = 16;
const MAX_RUNTIME_ARTIFACT_BYTES: u64 = 1024 * 1024 * 1024;

pub(crate) fn valid_runtime_artifacts(artifacts: &[ManagedRuntimeArtifactBindingV1]) -> bool {
    if artifacts.len() > MAX_RUNTIME_ARTIFACTS {
        return false;
    }
    let mut previous = "";
    let mut paths = BTreeSet::new();
    for artifact in artifacts {
        if !valid_identifier(&artifact.artifact_id)
            || artifact.artifact_id.as_str() <= previous
            || RuntimeArtifactUseV1::try_from(artifact.r#use)
                .ok()
                .is_none_or(|value| value == RuntimeArtifactUseV1::Unspecified)
            || !valid_private_path(&artifact.staged_path)
            || !paths.insert(artifact.staged_path.as_str())
            || !(1..=MAX_RUNTIME_ARTIFACT_BYTES).contains(&artifact.size_bytes)
            || artifact.sha256.len() != 32
            || !artifact.sha256.iter().any(|byte| *byte != 0)
        {
            return false;
        }
        previous = &artifact.artifact_id;
    }
    true
}

fn valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_IDENTIFIER_BYTES
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

fn valid_private_path(value: &str) -> bool {
    if value.is_empty()
        || value.len() > MAX_PRIVATE_PATH_BYTES
        || value.contains(['\0', '\n', '\r'])
    {
        return false;
    }
    let path = Path::new(value);
    path.is_absolute()
        && path
            .components()
            .all(|component| matches!(component, Component::RootDir | Component::Normal(_)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn binding(
        id: &str,
        path: &str,
        use_kind: RuntimeArtifactUseV1,
    ) -> ManagedRuntimeArtifactBindingV1 {
        ManagedRuntimeArtifactBindingV1 {
            artifact_id: id.to_owned(),
            r#use: use_kind as i32,
            staged_path: path.to_owned(),
            size_bytes: 1,
            sha256: vec![1; 32],
        }
    }

    #[test]
    fn accepts_ordered_exact_artifact_kinds() {
        assert!(valid_runtime_artifacts(&[
            binding(
                "ocr.eng.v1",
                "/private/eng",
                RuntimeArtifactUseV1::ReadOnlyData
            ),
            binding(
                "ocr.runner.v1",
                "/private/runner",
                RuntimeArtifactUseV1::NativeExecutable
            ),
        ]));
    }

    #[test]
    fn rejects_duplicate_paths_and_zero_digests() {
        let mut second = binding(
            "ocr.rus.v1",
            "/private/model",
            RuntimeArtifactUseV1::ReadOnlyData,
        );
        second.sha256 = vec![0; 32];
        assert!(!valid_runtime_artifacts(&[
            binding(
                "ocr.eng.v1",
                "/private/model",
                RuntimeArtifactUseV1::ReadOnlyData
            ),
            second,
        ]));
    }
}
