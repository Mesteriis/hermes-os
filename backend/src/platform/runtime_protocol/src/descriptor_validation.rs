use prost::Message;

use crate::v1::{CapabilityCriticalityV1, InitialOwnerEnrollmentChallengeV1, InitialOwnerEnrollmentV1, ModuleDescriptorV1, ModuleKindV1, SettingApplyModeV1, SettingClientVisibilityV1, SettingMutationAuthorityV1, SettingTargetScopeV1, SettingValueTypeV1, SettingsSchemaV1, SettingsSnapshotV1, setting_value_v1};

pub const MAX_DESCRIPTOR_BYTES: usize = 256 * 1024;
pub const MAX_CAPABILITIES: usize = 128;
pub const MAX_REFERENCES_PER_CAPABILITY: usize = 128;
pub const MAX_IDENTIFIER_BYTES: usize = 128;
pub const MAX_DISPLAY_BYTES: usize = 4096;
pub const MAX_SETTINGS_SCHEMA_BYTES: usize = 256 * 1024;
pub const MAX_SETTING_DEFINITIONS: usize = 512;
pub const MAX_SETTINGS_SNAPSHOT_BYTES: usize = 256 * 1024;
pub const MAX_SETTING_STRING_VALUE_BYTES: usize = 8192;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DescriptorValidationError { TooLarge, InvalidEncoding, InvalidMajor, InvalidKind, InvalidIdentifier, InvalidDisplayText, TooManyCapabilities, UnorderedCapabilityIds, InvalidCapability }

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsSchemaValidationError { TooLarge, InvalidEncoding, InvalidVersion, TooManyDefinitions, UnorderedSettingIds, InvalidDefinition }

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsSnapshotValidationError { Structural(SettingsSchemaValidationError), UnknownSettingId, ValueTypeMismatch }

pub fn validate_initial_owner_enrollment(challenge: &InitialOwnerEnrollmentChallengeV1, enrollment: &InitialOwnerEnrollmentV1) -> bool {
    challenge.protocol_major == 1 && challenge.instance_id.len() == 32 && challenge.nonce.len() == 32 && challenge.kernel_generation == 1
        && enrollment.protocol_major == 1 && enrollment.device_public_key_sec1.len() == 65 && enrollment.device_public_key_sec1[0] == 0x04 && enrollment.challenge_signature_raw.len() == 64
        && valid_enrollment_identity(&enrollment.owner_id) && valid_enrollment_identity(&enrollment.device_id)
}

fn valid_enrollment_identity(value: &str) -> bool {
    !value.is_empty() && value.len() <= MAX_IDENTIFIER_BYTES && value.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

pub fn decode_descriptor_v1(bytes: &[u8]) -> Result<ModuleDescriptorV1, DescriptorValidationError> {
    if bytes.len() > MAX_DESCRIPTOR_BYTES { return Err(DescriptorValidationError::TooLarge); }
    let descriptor = ModuleDescriptorV1::decode(bytes).map_err(|_| DescriptorValidationError::InvalidEncoding)?;
    validate_descriptor_v1(&descriptor)?;
    Ok(descriptor)
}

pub fn validate_descriptor_v1(descriptor: &ModuleDescriptorV1) -> Result<(), DescriptorValidationError> {
    if descriptor.descriptor_major != 1 { return Err(DescriptorValidationError::InvalidMajor); }
    if ModuleKindV1::try_from(descriptor.module_kind).ok().filter(|kind| *kind != ModuleKindV1::Unspecified).is_none() { return Err(DescriptorValidationError::InvalidKind); }
    for identifier in [&descriptor.module_id, &descriptor.owner_id, &descriptor.module_version, &descriptor.build_id] { validate_identifier(identifier)?; }
    if descriptor.display_name.len() > MAX_DISPLAY_BYTES || descriptor.display_name.contains(['<', '>', '`']) { return Err(DescriptorValidationError::InvalidDisplayText); }
    if descriptor.capabilities.len() > MAX_CAPABILITIES { return Err(DescriptorValidationError::TooManyCapabilities); }
    let mut previous = "";
    for capability in &descriptor.capabilities {
        validate_identifier(&capability.capability_id).map_err(|_| DescriptorValidationError::InvalidCapability)?;
        if capability.capability_id.as_str() <= previous { return Err(DescriptorValidationError::UnorderedCapabilityIds); }
        previous = &capability.capability_id;
        if CapabilityCriticalityV1::try_from(capability.criticality).ok().filter(|criticality| *criticality != CapabilityCriticalityV1::Unspecified).is_none()
            || capability.provides.len() > MAX_REFERENCES_PER_CAPABILITY
            || capability.dependencies.len() > MAX_REFERENCES_PER_CAPABILITY
            || capability.requests.iter().any(|request| request.request.is_none()) { return Err(DescriptorValidationError::InvalidCapability); }
    }
    Ok(())
}

fn validate_identifier(value: &str) -> Result<(), DescriptorValidationError> {
    if value.is_empty() || value.len() > MAX_IDENTIFIER_BYTES || !value.is_ascii() { Err(DescriptorValidationError::InvalidIdentifier) } else { Ok(()) }
}

pub fn decode_settings_schema_v1(bytes: &[u8]) -> Result<SettingsSchemaV1, SettingsSchemaValidationError> {
    if bytes.len() > MAX_SETTINGS_SCHEMA_BYTES { return Err(SettingsSchemaValidationError::TooLarge); }
    let schema = SettingsSchemaV1::decode(bytes).map_err(|_| SettingsSchemaValidationError::InvalidEncoding)?;
    validate_settings_schema_v1(&schema)?;
    Ok(schema)
}

pub fn validate_settings_schema_v1(schema: &SettingsSchemaV1) -> Result<(), SettingsSchemaValidationError> {
    if schema.major == 0 || schema.revision == 0 { return Err(SettingsSchemaValidationError::InvalidVersion); }
    if schema.definitions.len() > MAX_SETTING_DEFINITIONS { return Err(SettingsSchemaValidationError::TooManyDefinitions); }
    let mut previous = "";
    for definition in &schema.definitions {
        if definition.setting_id.is_empty() || definition.setting_id.len() > MAX_IDENTIFIER_BYTES || !definition.setting_id.is_ascii() || definition.setting_id.as_str() <= previous
            || (!definition.capability_id.is_empty() && (definition.capability_id.len() > MAX_IDENTIFIER_BYTES || !definition.capability_id.is_ascii()))
            || definition.display_name.len() > MAX_DISPLAY_BYTES || definition.display_name.contains(['<', '>', '`'])
            || SettingValueTypeV1::try_from(definition.value_type).ok().filter(|value| *value != SettingValueTypeV1::Unspecified).is_none()
            || SettingMutationAuthorityV1::try_from(definition.mutation_authority).ok().filter(|value| *value != SettingMutationAuthorityV1::Unspecified).is_none()
            || SettingTargetScopeV1::try_from(definition.target_scope).ok().filter(|value| *value != SettingTargetScopeV1::Unspecified).is_none()
            || SettingApplyModeV1::try_from(definition.apply_mode).ok().filter(|value| *value != SettingApplyModeV1::Unspecified).is_none()
            || SettingClientVisibilityV1::try_from(definition.client_visibility).ok().filter(|value| *value != SettingClientVisibilityV1::Unspecified).is_none()
            || (definition.mutation_authority == SettingMutationAuthorityV1::KernelManaged as i32 && definition.client_visibility == SettingClientVisibilityV1::Editable as i32)
            || (definition.mutation_authority == SettingMutationAuthorityV1::OperatorManaged as i32 && !definition.kernel_controller_id.is_empty())
            || (definition.mutation_authority == SettingMutationAuthorityV1::KernelManaged as i32 && (definition.kernel_controller_id.is_empty() || definition.kernel_controller_id.len() > MAX_IDENTIFIER_BYTES || !definition.kernel_controller_id.is_ascii())) {
            return Err(SettingsSchemaValidationError::InvalidDefinition);
        }
        previous = &definition.setting_id;
    }
    Ok(())
}

pub fn decode_settings_snapshot_v1(bytes: &[u8]) -> Result<SettingsSnapshotV1, SettingsSchemaValidationError> {
    if bytes.len() > MAX_SETTINGS_SNAPSHOT_BYTES { return Err(SettingsSchemaValidationError::TooLarge); }
    let snapshot = SettingsSnapshotV1::decode(bytes).map_err(|_| SettingsSchemaValidationError::InvalidEncoding)?;
    if snapshot.target_id.is_empty() || snapshot.target_id.len() > MAX_IDENTIFIER_BYTES || !snapshot.target_id.is_ascii() || snapshot.revision == 0 || snapshot.values.len() > MAX_SETTING_DEFINITIONS { return Err(SettingsSchemaValidationError::InvalidDefinition); }
    let mut previous = "";
    for entry in &snapshot.values { if entry.setting_id.is_empty() || entry.setting_id.len() > MAX_IDENTIFIER_BYTES || !entry.setting_id.is_ascii() || entry.setting_id.as_str() <= previous || entry.value.as_ref().and_then(|value| value.value.as_ref()).is_none() { return Err(SettingsSchemaValidationError::InvalidDefinition); } previous = &entry.setting_id; }
    Ok(snapshot)
}

pub fn validate_settings_snapshot_against_schema_v1(schema: &SettingsSchemaV1, snapshot: &SettingsSnapshotV1) -> Result<(), SettingsSnapshotValidationError> {
    validate_settings_schema_v1(schema).map_err(SettingsSnapshotValidationError::Structural)?;
    let encoded_snapshot = snapshot.encode_to_vec();
    decode_settings_snapshot_v1(&encoded_snapshot).map_err(SettingsSnapshotValidationError::Structural)?;
    for entry in &snapshot.values {
        let definition = schema.definitions.binary_search_by(|definition| definition.setting_id.cmp(&entry.setting_id)).map(|index| &schema.definitions[index]).map_err(|_| SettingsSnapshotValidationError::UnknownSettingId)?;
        let value = entry.value.as_ref().and_then(|value| value.value.as_ref()).ok_or(SettingsSnapshotValidationError::Structural(SettingsSchemaValidationError::InvalidDefinition))?;
        if !value_matches_setting_type(value, definition.value_type) || !value_within_protocol_limits(value) { return Err(SettingsSnapshotValidationError::ValueTypeMismatch); }
    }
    Ok(())
}

fn value_within_protocol_limits(value: &setting_value_v1::Value) -> bool {
    match value {
        setting_value_v1::Value::DecimalValue(value)
        | setting_value_v1::Value::StringValue(value)
        | setting_value_v1::Value::EnumValue(value) => value.len() <= MAX_SETTING_STRING_VALUE_BYTES,
        setting_value_v1::Value::ResourceReference(value) => !value.is_empty() && value.len() <= MAX_IDENTIFIER_BYTES && value.is_ascii(),
        _ => true,
    }
}

fn value_matches_setting_type(value: &setting_value_v1::Value, value_type: i32) -> bool {
    matches!(
        (value, SettingValueTypeV1::try_from(value_type).ok()),
        (setting_value_v1::Value::BooleanValue(_), Some(SettingValueTypeV1::Boolean))
            | (setting_value_v1::Value::SignedIntegerValue(_), Some(SettingValueTypeV1::SignedInteger))
            | (setting_value_v1::Value::UnsignedIntegerValue(_), Some(SettingValueTypeV1::UnsignedInteger))
            | (setting_value_v1::Value::DecimalValue(_), Some(SettingValueTypeV1::Decimal))
            | (setting_value_v1::Value::StringValue(_), Some(SettingValueTypeV1::String))
            | (setting_value_v1::Value::DurationMillis(_), Some(SettingValueTypeV1::Duration))
            | (setting_value_v1::Value::TimestampUnixMillis(_), Some(SettingValueTypeV1::Timestamp))
            | (setting_value_v1::Value::EnumValue(_), Some(SettingValueTypeV1::Enum))
            | (setting_value_v1::Value::ResourceReference(_), Some(SettingValueTypeV1::ResourceReference))
    )
}
