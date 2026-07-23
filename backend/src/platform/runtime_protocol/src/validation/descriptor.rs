use prost::Message;

use crate::v1::{
    CapabilityCriticalityV1, DurableEnvelopeKindV1, EventRouteDirectionV1,
    EventSubscriptionRequirementV1, InitialOwnerEnrollmentChallengeV1, InitialOwnerEnrollmentV1,
    ModuleDescriptorV1, ModuleKindV1, ProvidedSurfaceKindV1, SettingApplyModeV1, SettingClientVisibilityV1,
    SettingMutationAuthorityV1, SettingTargetScopeV1, SettingValueTypeV1, SettingsSchemaV1,
    SettingsSnapshotV1, VaultActionV1, VaultSecretClassV1, VaultTargetScopeV1,
    capability_request_v1, setting_value_v1,
};

pub const MAX_DESCRIPTOR_BYTES: usize = 256 * 1024;
pub const MAX_CAPABILITIES: usize = 128;
pub const MAX_REFERENCES_PER_CAPABILITY: usize = 128;
pub const MAX_VAULT_ACTIONS: usize = 7;
pub const MAX_VAULT_SECRET_CLASSES: usize = 6;
pub const MAX_VAULT_LEASE_TTL_SECONDS: u32 = 3_600;
pub const MAX_REQUEST_TIMEOUT_MILLIS: u32 = 60_000;
pub const MAX_REQUEST_CONNECTION_BUDGET: u32 = 1_024;
pub const MAX_TELEMETRY_SIGNALS_PER_MINUTE: u32 = 1_000_000;
pub const MAX_EVENT_ROUTE_IN_FLIGHT: u32 = 4_096;
pub const MAX_BLOB_QUOTA_BYTES: u64 = 1 << 40;
pub const MAX_IDENTIFIER_BYTES: usize = 128;
pub const MAX_DISPLAY_BYTES: usize = 4096;
pub const MAX_SETTINGS_SCHEMA_BYTES: usize = 256 * 1024;
pub const MAX_SETTING_DEFINITIONS: usize = 512;
pub const MAX_SETTINGS_SNAPSHOT_BYTES: usize = 256 * 1024;
pub const MAX_SETTING_STRING_VALUE_BYTES: usize = 8192;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DescriptorValidationError {
    TooLarge,
    InvalidEncoding,
    InvalidMajor,
    InvalidKind,
    InvalidIdentifier,
    InvalidDisplayText,
    InvalidSettingsSchemaReference,
    TooManyCapabilities,
    UnorderedCapabilityIds,
    InvalidCapability,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsSchemaValidationError {
    TooLarge,
    InvalidEncoding,
    InvalidVersion,
    TooManyDefinitions,
    UnorderedSettingIds,
    InvalidDefinition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsSnapshotValidationError {
    Structural(SettingsSchemaValidationError),
    UnknownSettingId,
    ValueTypeMismatch,
}

pub fn validate_initial_owner_enrollment(
    challenge: &InitialOwnerEnrollmentChallengeV1,
    enrollment: &InitialOwnerEnrollmentV1,
) -> bool {
    challenge.protocol_major == 1
        && challenge.instance_id.len() == 32
        && challenge.nonce.len() == 32
        && challenge.kernel_generation == 1
        && enrollment.protocol_major == 1
        && enrollment.device_public_key_sec1.len() == 65
        && enrollment.device_public_key_sec1[0] == 0x04
        && enrollment.challenge_signature_raw.len() == 64
        && valid_enrollment_identity(&enrollment.owner_id)
        && valid_enrollment_identity(&enrollment.device_id)
}

fn valid_enrollment_identity(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_IDENTIFIER_BYTES
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

pub fn decode_descriptor_v1(bytes: &[u8]) -> Result<ModuleDescriptorV1, DescriptorValidationError> {
    if bytes.len() > MAX_DESCRIPTOR_BYTES {
        return Err(DescriptorValidationError::TooLarge);
    }
    let descriptor = ModuleDescriptorV1::decode(bytes)
        .map_err(|_| DescriptorValidationError::InvalidEncoding)?;
    validate_descriptor_v1(&descriptor)?;
    Ok(descriptor)
}

pub fn validate_descriptor_v1(
    descriptor: &ModuleDescriptorV1,
) -> Result<(), DescriptorValidationError> {
    if descriptor.descriptor_major != 1 {
        return Err(DescriptorValidationError::InvalidMajor);
    }
    if ModuleKindV1::try_from(descriptor.module_kind)
        .ok()
        .filter(|kind| *kind != ModuleKindV1::Unspecified)
        .is_none()
    {
        return Err(DescriptorValidationError::InvalidKind);
    }
    for identifier in [
        &descriptor.module_id,
        &descriptor.owner_id,
        &descriptor.module_version,
        &descriptor.build_id,
    ] {
        validate_identifier(identifier)?;
    }
    if descriptor.display_name.len() > MAX_DISPLAY_BYTES
        || descriptor.display_name.contains(['<', '>', '`'])
    {
        return Err(DescriptorValidationError::InvalidDisplayText);
    }
    if descriptor.capabilities.len() > MAX_CAPABILITIES {
        return Err(DescriptorValidationError::TooManyCapabilities);
    }
    if !valid_settings_schema_reference(descriptor) {
        return Err(DescriptorValidationError::InvalidSettingsSchemaReference);
    }
    let mut previous = "";
    for capability in &descriptor.capabilities {
        validate_identifier(&capability.capability_id)
            .map_err(|_| DescriptorValidationError::InvalidCapability)?;
        if capability.capability_id.as_str() <= previous {
            return Err(DescriptorValidationError::UnorderedCapabilityIds);
        }
        previous = &capability.capability_id;
        if CapabilityCriticalityV1::try_from(capability.criticality)
            .ok()
            .filter(|criticality| *criticality != CapabilityCriticalityV1::Unspecified)
            .is_none()
            || capability.provides.len() > MAX_REFERENCES_PER_CAPABILITY
            || capability.provides.iter().any(|surface| !valid_provided_surface(surface))
            || capability.dependencies.len() > MAX_REFERENCES_PER_CAPABILITY
            || capability
                .requests
                .iter()
                .any(|request| !valid_capability_request(request))
        {
            return Err(DescriptorValidationError::InvalidCapability);
        }
    }
    Ok(())
}

fn valid_provided_surface(surface: &crate::v1::ProvidedSurfaceV1) -> bool {
    let Some(kind) = ProvidedSurfaceKindV1::try_from(surface.kind).ok() else { return false; };
    if kind == ProvidedSurfaceKindV1::Unspecified || !surface.contract.as_ref().is_some_and(valid_contract_reference) {
        return false;
    }
    match kind {
        ProvidedSurfaceKindV1::ClientRpc => surface.client_rpc_route.as_ref().is_some_and(|route| valid_client_rpc_path(&route.path)),
        _ => surface.client_rpc_route.is_none(),
    }
}

fn valid_client_rpc_path(path: &str) -> bool {
    let mut segments = path.split('/');
    matches!(segments.next(), Some(""))
        && segments.next().is_some_and(valid_connect_component)
        && segments.next().is_some_and(valid_connect_component)
        && segments.next().is_none()
        && path.len() <= 512
}

fn valid_connect_component(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_'))
}

fn valid_settings_schema_reference(descriptor: &ModuleDescriptorV1) -> bool {
    descriptor
        .settings_schema_ref
        .as_ref()
        .is_none_or(|reference| {
            reference.major > 0
                && reference.revision > 0
                && (1..=MAX_SETTINGS_SCHEMA_BYTES as u64).contains(&reference.artifact_size_bytes)
                && reference.sha256.len() == 32
        })
}

fn valid_capability_request(request: &crate::v1::CapabilityRequestV1) -> bool {
    match request.request.as_ref() {
        Some(capability_request_v1::Request::StorageNamespace(storage)) => {
            validate_identifier(&storage.owner_id).is_ok()
                && (1..=MAX_REQUEST_CONNECTION_BUDGET).contains(&storage.connection_budget)
                && (1..=MAX_REQUEST_TIMEOUT_MILLIS).contains(&storage.timeout_millis)
        }
        Some(capability_request_v1::Request::VaultPurpose(purpose)) => {
            valid_vault_purpose_request(purpose)
        }
        Some(capability_request_v1::Request::BlobQuota(blob)) => {
            (1..=MAX_BLOB_QUOTA_BYTES).contains(&blob.max_bytes)
        }
        Some(capability_request_v1::Request::ClockTimer(clock)) => clock.requires_wall_clock,
        Some(capability_request_v1::Request::SchedulerJob(scheduler)) => scheduler
            .job_kind
            .as_ref()
            .is_some_and(valid_contract_reference),
        Some(capability_request_v1::Request::TelemetrySignal(telemetry)) => {
            validate_identifier(&telemetry.signal_class).is_ok()
                && (1..=MAX_TELEMETRY_SIGNALS_PER_MINUTE).contains(&telemetry.quota_per_minute)
        }
        Some(capability_request_v1::Request::HostCapability(host)) => {
            validate_identifier(&host.capability_id).is_ok()
        }
        Some(capability_request_v1::Request::EventRoute(route)) => valid_event_route_request(route),
        None => false,
    }
}

fn valid_event_route_request(route: &crate::v1::EventRouteRequestV1) -> bool {
    DurableEnvelopeKindV1::try_from(route.envelope_kind)
        .ok()
        .is_some_and(|kind| kind != DurableEnvelopeKindV1::Unspecified)
        && route
            .contract
            .as_ref()
            .is_some_and(valid_contract_reference)
        && EventRouteDirectionV1::try_from(route.direction)
            .ok()
            .is_some_and(|direction| direction != EventRouteDirectionV1::Unspecified)
        && (1..=MAX_EVENT_ROUTE_IN_FLIGHT).contains(&route.max_in_flight)
        && valid_event_delivery_policy(route)
}

fn valid_event_delivery_policy(route: &crate::v1::EventRouteRequestV1) -> bool {
    match EventRouteDirectionV1::try_from(route.direction).ok() {
        Some(EventRouteDirectionV1::Publish) => {
            route.subscription_requirement == EventSubscriptionRequirementV1::Unspecified as i32
                && route.max_deliver == 0
                && route.ack_wait_millis == 0
        }
        Some(EventRouteDirectionV1::Consume) => {
            matches!(
                EventSubscriptionRequirementV1::try_from(route.subscription_requirement).ok(),
                Some(
                    EventSubscriptionRequirementV1::Required
                        | EventSubscriptionRequirementV1::Optional
                )
            ) && (1..=32).contains(&route.max_deliver)
                && (1..=600_000).contains(&route.ack_wait_millis)
        }
        _ => false,
    }
}

fn valid_contract_reference(reference: &crate::v1::ContractReferenceV1) -> bool {
    validate_identifier(&reference.owner).is_ok()
        && validate_identifier(&reference.name).is_ok()
        && reference.major > 0
        && reference.revision > 0
        && reference.schema_sha256.len() == 32
}

fn valid_vault_purpose_request(purpose: &crate::v1::VaultPurposeRequestV1) -> bool {
    validate_identifier(&purpose.purpose_id).is_ok()
        && (1..=MAX_VAULT_LEASE_TTL_SECONDS).contains(&purpose.requested_lease_ttl_seconds)
        && valid_ordered_enum_set(
            &purpose.allowed_secret_classes,
            MAX_VAULT_SECRET_CLASSES,
            |value| {
                VaultSecretClassV1::try_from(value)
                    .ok()
                    .is_some_and(|value| value != VaultSecretClassV1::Unspecified)
            },
        )
        && valid_ordered_enum_set(&purpose.actions, MAX_VAULT_ACTIONS, |value| {
            VaultActionV1::try_from(value)
                .ok()
                .is_some_and(|value| value != VaultActionV1::Unspecified)
        })
        && valid_vault_target_scope(purpose)
}

fn valid_vault_target_scope(purpose: &crate::v1::VaultPurposeRequestV1) -> bool {
    match VaultTargetScopeV1::try_from(purpose.target_scope).ok() {
        Some(VaultTargetScopeV1::ConfigurationInstance) => purpose.key_schema_revision == 0,
        Some(VaultTargetScopeV1::OwnerDerivedProjectionKey) => {
            purpose.key_schema_revision != 0
                && purpose.allowed_secret_classes
                    == [VaultSecretClassV1::OwnerDerivedKey as i32]
                && purpose.actions == [VaultActionV1::IssueOwnerDerivedKey as i32]
        }
        _ => false,
    }
}

fn valid_ordered_enum_set(values: &[i32], maximum: usize, valid: impl Fn(i32) -> bool) -> bool {
    !values.is_empty()
        && values.len() <= maximum
        && values.iter().copied().all(&valid)
        && values.windows(2).all(|items| items[0] < items[1])
}

fn validate_identifier(value: &str) -> Result<(), DescriptorValidationError> {
    if value.is_empty() || value.len() > MAX_IDENTIFIER_BYTES || !value.is_ascii() {
        Err(DescriptorValidationError::InvalidIdentifier)
    } else {
        Ok(())
    }
}

pub fn decode_settings_schema_v1(
    bytes: &[u8],
) -> Result<SettingsSchemaV1, SettingsSchemaValidationError> {
    if bytes.len() > MAX_SETTINGS_SCHEMA_BYTES {
        return Err(SettingsSchemaValidationError::TooLarge);
    }
    let schema = SettingsSchemaV1::decode(bytes)
        .map_err(|_| SettingsSchemaValidationError::InvalidEncoding)?;
    validate_settings_schema_v1(&schema)?;
    Ok(schema)
}

pub fn validate_settings_schema_v1(
    schema: &SettingsSchemaV1,
) -> Result<(), SettingsSchemaValidationError> {
    if schema.major == 0 || schema.revision == 0 {
        return Err(SettingsSchemaValidationError::InvalidVersion);
    }
    if schema.definitions.len() > MAX_SETTING_DEFINITIONS {
        return Err(SettingsSchemaValidationError::TooManyDefinitions);
    }
    let mut previous = "";
    for definition in &schema.definitions {
        if definition.setting_id.is_empty()
            || definition.setting_id.len() > MAX_IDENTIFIER_BYTES
            || !definition.setting_id.is_ascii()
            || definition.setting_id.as_str() <= previous
            || (!definition.capability_id.is_empty()
                && (definition.capability_id.len() > MAX_IDENTIFIER_BYTES
                    || !definition.capability_id.is_ascii()))
            || definition.display_name.len() > MAX_DISPLAY_BYTES
            || definition.display_name.contains(['<', '>', '`'])
            || SettingValueTypeV1::try_from(definition.value_type)
                .ok()
                .filter(|value| *value != SettingValueTypeV1::Unspecified)
                .is_none()
            || SettingMutationAuthorityV1::try_from(definition.mutation_authority)
                .ok()
                .filter(|value| *value != SettingMutationAuthorityV1::Unspecified)
                .is_none()
            || SettingTargetScopeV1::try_from(definition.target_scope)
                .ok()
                .filter(|value| *value != SettingTargetScopeV1::Unspecified)
                .is_none()
            || SettingApplyModeV1::try_from(definition.apply_mode)
                .ok()
                .filter(|value| *value != SettingApplyModeV1::Unspecified)
                .is_none()
            || SettingClientVisibilityV1::try_from(definition.client_visibility)
                .ok()
                .filter(|value| *value != SettingClientVisibilityV1::Unspecified)
                .is_none()
            || (definition.mutation_authority == SettingMutationAuthorityV1::KernelManaged as i32
                && definition.client_visibility == SettingClientVisibilityV1::Editable as i32)
            || (definition.mutation_authority == SettingMutationAuthorityV1::OperatorManaged as i32
                && !definition.kernel_controller_id.is_empty())
            || (definition.mutation_authority == SettingMutationAuthorityV1::KernelManaged as i32
                && (definition.kernel_controller_id.is_empty()
                    || definition.kernel_controller_id.len() > MAX_IDENTIFIER_BYTES
                    || !definition.kernel_controller_id.is_ascii()))
        {
            return Err(SettingsSchemaValidationError::InvalidDefinition);
        }
        previous = &definition.setting_id;
    }
    Ok(())
}

pub fn decode_settings_snapshot_v1(
    bytes: &[u8],
) -> Result<SettingsSnapshotV1, SettingsSchemaValidationError> {
    if bytes.len() > MAX_SETTINGS_SNAPSHOT_BYTES {
        return Err(SettingsSchemaValidationError::TooLarge);
    }
    let snapshot = SettingsSnapshotV1::decode(bytes)
        .map_err(|_| SettingsSchemaValidationError::InvalidEncoding)?;
    if snapshot.target_id.is_empty()
        || snapshot.target_id.len() > MAX_IDENTIFIER_BYTES
        || !snapshot.target_id.is_ascii()
        || snapshot.revision == 0
        || snapshot.values.len() > MAX_SETTING_DEFINITIONS
    {
        return Err(SettingsSchemaValidationError::InvalidDefinition);
    }
    let mut previous = "";
    for entry in &snapshot.values {
        if entry.setting_id.is_empty()
            || entry.setting_id.len() > MAX_IDENTIFIER_BYTES
            || !entry.setting_id.is_ascii()
            || entry.setting_id.as_str() <= previous
            || entry
                .value
                .as_ref()
                .and_then(|value| value.value.as_ref())
                .is_none()
        {
            return Err(SettingsSchemaValidationError::InvalidDefinition);
        }
        previous = &entry.setting_id;
    }
    Ok(snapshot)
}

pub fn validate_settings_snapshot_against_schema_v1(
    schema: &SettingsSchemaV1,
    snapshot: &SettingsSnapshotV1,
) -> Result<(), SettingsSnapshotValidationError> {
    validate_settings_schema_v1(schema).map_err(SettingsSnapshotValidationError::Structural)?;
    let encoded_snapshot = snapshot.encode_to_vec();
    decode_settings_snapshot_v1(&encoded_snapshot)
        .map_err(SettingsSnapshotValidationError::Structural)?;
    for entry in &snapshot.values {
        let definition = schema
            .definitions
            .binary_search_by(|definition| definition.setting_id.cmp(&entry.setting_id))
            .map(|index| &schema.definitions[index])
            .map_err(|_| SettingsSnapshotValidationError::UnknownSettingId)?;
        let value = entry
            .value
            .as_ref()
            .and_then(|value| value.value.as_ref())
            .ok_or(SettingsSnapshotValidationError::Structural(
                SettingsSchemaValidationError::InvalidDefinition,
            ))?;
        if !value_matches_setting_type(value, definition.value_type)
            || !value_within_protocol_limits(value)
        {
            return Err(SettingsSnapshotValidationError::ValueTypeMismatch);
        }
    }
    Ok(())
}

fn value_within_protocol_limits(value: &setting_value_v1::Value) -> bool {
    match value {
        setting_value_v1::Value::DecimalValue(value)
        | setting_value_v1::Value::StringValue(value)
        | setting_value_v1::Value::EnumValue(value) => {
            value.len() <= MAX_SETTING_STRING_VALUE_BYTES
        }
        setting_value_v1::Value::ResourceReference(value) => {
            !value.is_empty() && value.len() <= MAX_IDENTIFIER_BYTES && value.is_ascii()
        }
        _ => true,
    }
}

fn value_matches_setting_type(value: &setting_value_v1::Value, value_type: i32) -> bool {
    matches!(
        (value, SettingValueTypeV1::try_from(value_type).ok()),
        (
            setting_value_v1::Value::BooleanValue(_),
            Some(SettingValueTypeV1::Boolean)
        ) | (
            setting_value_v1::Value::SignedIntegerValue(_),
            Some(SettingValueTypeV1::SignedInteger)
        ) | (
            setting_value_v1::Value::UnsignedIntegerValue(_),
            Some(SettingValueTypeV1::UnsignedInteger)
        ) | (
            setting_value_v1::Value::DecimalValue(_),
            Some(SettingValueTypeV1::Decimal)
        ) | (
            setting_value_v1::Value::StringValue(_),
            Some(SettingValueTypeV1::String)
        ) | (
            setting_value_v1::Value::DurationMillis(_),
            Some(SettingValueTypeV1::Duration)
        ) | (
            setting_value_v1::Value::TimestampUnixMillis(_),
            Some(SettingValueTypeV1::Timestamp)
        ) | (
            setting_value_v1::Value::EnumValue(_),
            Some(SettingValueTypeV1::Enum)
        ) | (
            setting_value_v1::Value::ResourceReference(_),
            Some(SettingValueTypeV1::ResourceReference)
        )
    )
}

#[cfg(test)]
mod tests {
    use super::{validate_descriptor_v1, valid_vault_purpose_request};
    use crate::v1::{
        CapabilityCriticalityV1, CapabilityDescriptorV1, ClientRpcRouteV1, ContractReferenceV1,
        ModuleDescriptorV1, ModuleKindV1, ProvidedSurfaceKindV1, ProvidedSurfaceV1,
        VaultActionV1, VaultPurposeRequestV1, VaultSecretClassV1, VaultTargetScopeV1,
    };

    fn owner_derived_key_purpose() -> VaultPurposeRequestV1 {
        VaultPurposeRequestV1 {
            purpose_id: "communications.search.index".to_owned(),
            requested_lease_ttl_seconds: 60,
            allowed_secret_classes: vec![VaultSecretClassV1::OwnerDerivedKey as i32],
            actions: vec![VaultActionV1::IssueOwnerDerivedKey as i32],
            target_scope: VaultTargetScopeV1::OwnerDerivedProjectionKey as i32,
            key_schema_revision: 1,
        }
    }

    #[test]
    fn accepts_only_exact_owner_derived_key_purpose_shape() {
        assert!(valid_vault_purpose_request(&owner_derived_key_purpose()));
        let mut missing_revision = owner_derived_key_purpose();
        missing_revision.key_schema_revision = 0;
        assert!(!valid_vault_purpose_request(&missing_revision));
        let mut mixed_class = owner_derived_key_purpose();
        mixed_class.allowed_secret_classes.push(VaultSecretClassV1::PlatformCredential as i32);
        assert!(!valid_vault_purpose_request(&mixed_class));
        let mut wrong_action = owner_derived_key_purpose();
        wrong_action.actions = vec![VaultActionV1::Resolve as i32];
        assert!(!valid_vault_purpose_request(&wrong_action));
    }

    #[test]
    fn client_rpc_surface_requires_one_exact_connect_path() {
        let mut descriptor = client_rpc_descriptor();
        assert_eq!(validate_descriptor_v1(&descriptor), Ok(()));

        descriptor.capabilities[0].provides[0].client_rpc_route = None;
        assert!(validate_descriptor_v1(&descriptor).is_err());

        let mut descriptor = client_rpc_descriptor();
        descriptor.capabilities[0].provides[0].client_rpc_route = Some(ClientRpcRouteV1 {
            path: "/hermes.notes.v1.NotesQueryService/../Query".to_owned(),
        });
        assert!(validate_descriptor_v1(&descriptor).is_err());

        let mut descriptor = client_rpc_descriptor();
        descriptor.capabilities[0].provides[0].kind = ProvidedSurfaceKindV1::QueryRpc as i32;
        assert!(validate_descriptor_v1(&descriptor).is_err());
    }

    fn client_rpc_descriptor() -> ModuleDescriptorV1 {
        ModuleDescriptorV1 {
            descriptor_major: 1,
            descriptor_revision: 1,
            module_id: "module_notes".to_owned(),
            owner_id: "owner_notes".to_owned(),
            module_kind: ModuleKindV1::Domain as i32,
            module_version: "1".to_owned(),
            build_id: "build".to_owned(),
            capabilities: vec![CapabilityDescriptorV1 {
                capability_id: "notes.query".to_owned(),
                capability_revision: 1,
                criticality: CapabilityCriticalityV1::Required as i32,
                provides: vec![ProvidedSurfaceV1 {
                    kind: ProvidedSurfaceKindV1::ClientRpc as i32,
                    contract: Some(ContractReferenceV1 {
                        owner: "owner_notes".to_owned(),
                        name: "notes.query".to_owned(),
                        major: 1,
                        revision: 1,
                        schema_sha256: vec![7; 32],
                    }),
                    client_rpc_route: Some(ClientRpcRouteV1 {
                        path: "/hermes.notes.v1.NotesQueryService/Query".to_owned(),
                    }),
                }],
                ..Default::default()
            }],
            ..Default::default()
        }
    }
}
