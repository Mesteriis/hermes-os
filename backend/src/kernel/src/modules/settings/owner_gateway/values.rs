//! Public-to-canonical typed Settings value conversion.

use hermes_gateway_protocol::v1::{OwnerSettingEntryV1, owner_setting_value_v1};
use hermes_gateway_runtime::OwnerModuleSettingsRouteErrorV1;
use hermes_runtime_protocol::v1::{
    SettingValueV1, SettingsSnapshotV1, SettingsValueEntryV1, setting_value_v1,
};

pub(super) fn canonical_snapshot(
    registration_id: &str,
    expected_desired_revision: u64,
    mut values: Vec<OwnerSettingEntryV1>,
) -> Result<SettingsSnapshotV1, OwnerModuleSettingsRouteErrorV1> {
    let revision = expected_desired_revision
        .checked_add(1)
        .ok_or(OwnerModuleSettingsRouteErrorV1::InvalidArgument)?;
    values.sort_unstable_by(|left, right| left.setting_id.cmp(&right.setting_id));
    if values
        .windows(2)
        .any(|pair| pair[0].setting_id == pair[1].setting_id)
    {
        return Err(OwnerModuleSettingsRouteErrorV1::InvalidArgument);
    }
    let values = values
        .into_iter()
        .map(canonical_entry)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(SettingsSnapshotV1 {
        target_id: registration_id.to_owned(),
        revision,
        values,
    })
}

fn canonical_entry(
    entry: OwnerSettingEntryV1,
) -> Result<SettingsValueEntryV1, OwnerModuleSettingsRouteErrorV1> {
    if entry.setting_id.is_empty() {
        return Err(OwnerModuleSettingsRouteErrorV1::InvalidArgument);
    }
    let value = entry
        .value
        .and_then(|value| value.value)
        .ok_or(OwnerModuleSettingsRouteErrorV1::InvalidArgument)?;
    let value = match value {
        owner_setting_value_v1::Value::BooleanValue(value) => {
            setting_value_v1::Value::BooleanValue(value)
        }
        owner_setting_value_v1::Value::SignedIntegerValue(value) => {
            setting_value_v1::Value::SignedIntegerValue(value)
        }
        owner_setting_value_v1::Value::UnsignedIntegerValue(value) => {
            setting_value_v1::Value::UnsignedIntegerValue(value)
        }
        owner_setting_value_v1::Value::DecimalValue(value) => {
            setting_value_v1::Value::DecimalValue(value)
        }
        owner_setting_value_v1::Value::StringValue(value) => {
            setting_value_v1::Value::StringValue(value)
        }
        owner_setting_value_v1::Value::DurationMillis(value) => {
            setting_value_v1::Value::DurationMillis(value)
        }
        owner_setting_value_v1::Value::TimestampUnixMillis(value) => {
            setting_value_v1::Value::TimestampUnixMillis(value)
        }
        owner_setting_value_v1::Value::EnumValue(value) => {
            setting_value_v1::Value::EnumValue(value)
        }
        owner_setting_value_v1::Value::ResourceReference(value) => {
            setting_value_v1::Value::ResourceReference(value)
        }
    };
    Ok(SettingsValueEntryV1 {
        setting_id: entry.setting_id,
        value: Some(SettingValueV1 { value: Some(value) }),
    })
}
