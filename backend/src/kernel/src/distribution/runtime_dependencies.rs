//! Exact descriptor/grant/distribution intersection for managed integration inputs.

use hermes_runtime_protocol::v1::{
    DistributionArtifactKindV1, DistributionManifestArtifactV1, DistributionManifestV1,
    ModuleDescriptorV1, RuntimeArtifactUseV1, capability_request_v1,
};

pub struct IntegrationRuntimeRequirementsV1 {
    runtime_artifacts: Vec<DistributionManifestArtifactV1>,
    state_layout_revision: Option<u32>,
}

impl IntegrationRuntimeRequirementsV1 {
    #[must_use]
    pub fn runtime_artifacts(&self) -> &[DistributionManifestArtifactV1] {
        &self.runtime_artifacts
    }

    #[must_use]
    pub fn state_layout_revision(&self) -> Option<u32> {
        self.state_layout_revision
    }
}

pub fn select(
    descriptor: &ModuleDescriptorV1,
    granted_capability_ids: &[String],
    manifest: &DistributionManifestV1,
) -> Result<IntegrationRuntimeRequirementsV1, String> {
    if granted_capability_ids
        .windows(2)
        .any(|pair| pair[0] >= pair[1])
    {
        return Err("managed integration grants are not exact ordered identities".to_owned());
    }
    for capability_id in granted_capability_ids {
        if descriptor
            .capabilities
            .binary_search_by(|candidate| candidate.capability_id.as_str().cmp(capability_id))
            .is_err()
        {
            return Err("managed integration grant is absent from exact descriptor".to_owned());
        }
    }

    let mut runtime_artifacts = Vec::new();
    let mut state_layout_revision = None;
    for capability in &descriptor.capabilities {
        if granted_capability_ids
            .binary_search_by(|candidate| candidate.as_str().cmp(&capability.capability_id))
            .is_err()
        {
            continue;
        }
        for request in &capability.requests {
            match request.request.as_ref() {
                Some(capability_request_v1::Request::RuntimeArtifact(request)) => {
                    if RuntimeArtifactUseV1::try_from(request.r#use)
                        .ok()
                        .is_none_or(|value| value != RuntimeArtifactUseV1::NativeDynamicLibrary)
                    {
                        return Err(
                            "managed integration runtime artifact use is unsupported".to_owned()
                        );
                    }
                    let artifact = manifest
                        .artifacts
                        .binary_search_by(|candidate| {
                            candidate.artifact_id.as_str().cmp(&request.artifact_id)
                        })
                        .ok()
                        .map(|index| &manifest.artifacts[index])
                        .ok_or_else(|| {
                            "managed integration runtime artifact is absent from distribution"
                                .to_owned()
                        })?;
                    if artifact.artifact_kind
                        != DistributionArtifactKindV1::ModuleRuntimeNativeDependency as i32
                        || artifact.bound_module_id != descriptor.module_id
                    {
                        return Err(
                            "managed integration runtime artifact binding is invalid".to_owned()
                        );
                    }
                    runtime_artifacts.push(artifact.clone());
                }
                Some(capability_request_v1::Request::IntegrationState(request)) => {
                    if request.state_layout_revision == 0
                        || state_layout_revision
                            .is_some_and(|revision| revision != request.state_layout_revision)
                    {
                        return Err(
                            "managed integration state layout request is ambiguous".to_owned()
                        );
                    }
                    state_layout_revision = Some(request.state_layout_revision);
                }
                _ => {}
            }
        }
    }

    runtime_artifacts.sort_by(|left, right| left.artifact_id.cmp(&right.artifact_id));
    runtime_artifacts.dedup_by(|left, right| left.artifact_id == right.artifact_id);
    Ok(IntegrationRuntimeRequirementsV1 {
        runtime_artifacts,
        state_layout_revision,
    })
}

#[cfg(test)]
mod tests {
    use super::select;
    use hermes_runtime_protocol::v1::{
        CapabilityCriticalityV1, CapabilityDescriptorV1, CapabilityRequestV1,
        DistributionArtifactKindV1, DistributionManifestArtifactV1, DistributionManifestV1,
        IntegrationStateRequestV1, ModuleDescriptorV1, ModuleKindV1, RuntimeArtifactRequestV1,
        RuntimeArtifactUseV1, capability_request_v1,
    };

    #[test]
    fn selects_only_granted_exact_module_artifacts_and_one_state_layout() {
        let descriptor = descriptor();
        let manifest = manifest("hermes-telegram-runtime");

        let none = select(&descriptor, &[], &manifest).expect("no grants");
        assert!(none.runtime_artifacts().is_empty());
        assert_eq!(none.state_layout_revision(), None);

        let requirements = select(&descriptor, &["telegram.runtime.v1".to_owned()], &manifest)
            .expect("granted runtime");
        assert_eq!(requirements.runtime_artifacts().len(), 1);
        assert_eq!(
            requirements.runtime_artifacts()[0].artifact_id,
            "telegram.tdjson.v1"
        );
        assert_eq!(requirements.state_layout_revision(), Some(1));
    }

    #[test]
    fn rejects_unknown_grants_and_cross_module_artifact_binding() {
        let descriptor = descriptor();
        assert!(
            select(
                &descriptor,
                &["telegram.unknown.v1".to_owned()],
                &manifest("hermes-telegram-runtime"),
            )
            .is_err()
        );
        assert!(
            select(
                &descriptor,
                &["telegram.runtime.v1".to_owned()],
                &manifest("hermes-other-runtime"),
            )
            .is_err()
        );
    }

    fn descriptor() -> ModuleDescriptorV1 {
        ModuleDescriptorV1 {
            descriptor_major: 1,
            descriptor_revision: 1,
            module_id: "hermes-telegram-runtime".to_owned(),
            owner_id: "telegram".to_owned(),
            module_kind: ModuleKindV1::Integration as i32,
            module_version: "1".to_owned(),
            build_id: "build-1".to_owned(),
            capabilities: vec![CapabilityDescriptorV1 {
                capability_id: "telegram.runtime.v1".to_owned(),
                capability_revision: 1,
                criticality: CapabilityCriticalityV1::Required as i32,
                requests: vec![
                    CapabilityRequestV1 {
                        request: Some(capability_request_v1::Request::RuntimeArtifact(
                            RuntimeArtifactRequestV1 {
                                artifact_id: "telegram.tdjson.v1".to_owned(),
                                r#use: RuntimeArtifactUseV1::NativeDynamicLibrary as i32,
                            },
                        )),
                    },
                    CapabilityRequestV1 {
                        request: Some(capability_request_v1::Request::IntegrationState(
                            IntegrationStateRequestV1 {
                                state_layout_revision: 1,
                            },
                        )),
                    },
                ],
                ..Default::default()
            }],
            ..Default::default()
        }
    }

    fn manifest(bound_module_id: &str) -> DistributionManifestV1 {
        DistributionManifestV1 {
            artifacts: vec![DistributionManifestArtifactV1 {
                artifact_kind: DistributionArtifactKindV1::ModuleRuntimeNativeDependency as i32,
                artifact_id: "telegram.tdjson.v1".to_owned(),
                bound_module_id: bound_module_id.to_owned(),
                ..Default::default()
            }],
            ..Default::default()
        }
    }
}
