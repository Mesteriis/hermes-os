use super::common::*;

#[test]
fn descriptor_validation_rejects_unsorted_or_untyped_capabilities() {
    let mut descriptor = descriptor_fixture();
    descriptor.capabilities = vec![capability("z"), capability("a")];
    assert!(decode_descriptor_v1(&descriptor.encode_to_vec()).is_err());
}

#[test]
fn descriptor_validation_accepts_a_canonical_typed_descriptor() {
    let mut descriptor = descriptor_fixture();
    descriptor.capabilities = vec![capability("read")];
    let encoded = descriptor.encode_to_vec();
    assert_eq!(
        decode_descriptor_v1(&encoded).expect("canonical descriptor"),
        descriptor
    );
    descriptor.settings_schema_ref = Some(SettingsSchemaRefV1::default());
    assert!(decode_descriptor_v1(&descriptor.encode_to_vec()).is_err());
}

fn descriptor_fixture() -> ModuleDescriptorV1 {
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: "mail".into(),
        owner_id: "communications".into(),
        module_kind: ModuleKindV1::Integration as i32,
        module_version: "1".into(),
        build_id: "build".into(),
        ..Default::default()
    }
}

fn capability(capability_id: &str) -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: capability_id.into(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        ..Default::default()
    }
}
