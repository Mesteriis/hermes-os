use serde_json::Value;

use super::constants::SECRET_LIKE_MARKERS;
use super::errors::SettingsError;
use super::models::DeclaredApplicationSetting;

pub(crate) fn validate_declared_setting(
    declared: &DeclaredApplicationSetting,
) -> Result<(), SettingsError> {
    validate_setting_key(declared.setting_key)?;
    validate_non_empty("category", declared.category)?;
    validate_non_empty("label", declared.label)?;
    if !declared.metadata.is_object() {
        return Err(SettingsError::InvalidValue(
            "metadata must be a JSON object",
        ));
    }
    declared
        .value_kind
        .validate_value(&declared.default_value, &declared.metadata)?;

    Ok(())
}

pub(crate) fn validate_json_metadata_constraints(
    value: &Value,
    metadata: &Value,
) -> Result<(), SettingsError> {
    if let Some(max_bytes) = metadata.get("max_bytes").and_then(Value::as_u64)
        && (value.to_string().len() as u64) > max_bytes
    {
        return Err(SettingsError::InvalidValue(
            "JSON value exceeds maximum size",
        ));
    }

    let forbidden_keys = metadata
        .get("forbidden_keys")
        .and_then(Value::as_array)
        .map(|keys| {
            keys.iter()
                .filter_map(Value::as_str)
                .map(str::to_ascii_lowercase)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if !forbidden_keys.is_empty() && json_value_has_forbidden_key(value, &forbidden_keys) {
        return Err(SettingsError::InvalidValue(
            "JSON value contains private content keys",
        ));
    }

    Ok(())
}

fn json_value_has_forbidden_key(value: &Value, forbidden_keys: &[String]) -> bool {
    match value {
        Value::Object(object) => object.iter().any(|(key, child)| {
            is_forbidden_json_key(key, forbidden_keys)
                || json_value_has_forbidden_key(child, forbidden_keys)
        }),
        Value::Array(items) => items
            .iter()
            .any(|item| json_value_has_forbidden_key(item, forbidden_keys)),
        _ => false,
    }
}

fn is_forbidden_json_key(key: &str, forbidden_keys: &[String]) -> bool {
    let key = key.to_ascii_lowercase();
    forbidden_keys.iter().any(|marker| {
        key == *marker
            || key.starts_with(marker)
            || key.contains(&format!("_{marker}"))
            || key.contains(&format!("{marker}_"))
            || key.contains(&format!("-{marker}"))
            || key.contains(&format!("{marker}-"))
            || key.contains(&format!(".{marker}"))
            || key.contains(&format!("{marker}."))
            || (marker != "text" && key.ends_with(marker))
    })
}

pub(crate) fn validate_setting_key(setting_key: &str) -> Result<(), SettingsError> {
    validate_non_empty("setting_key", setting_key)?;
    let setting_key = setting_key.trim();
    let has_valid_format = setting_key.chars().all(|character| {
        character.is_ascii_lowercase()
            || character.is_ascii_digit()
            || matches!(character, '_' | '-' | '.')
    }) && setting_key
        .chars()
        .next()
        .is_some_and(|character| character.is_ascii_lowercase() || character.is_ascii_digit())
        && setting_key
            .chars()
            .last()
            .is_some_and(|character| character.is_ascii_lowercase() || character.is_ascii_digit());
    if !has_valid_format {
        return Err(SettingsError::InvalidSettingKey);
    }

    if SECRET_LIKE_MARKERS
        .iter()
        .any(|marker| setting_key.contains(marker))
    {
        return Err(SettingsError::SecretLikeSettingKey);
    }

    Ok(())
}

pub(crate) fn validate_non_empty(field: &'static str, value: &str) -> Result<(), SettingsError> {
    if value.trim().is_empty() {
        return Err(SettingsError::EmptyField(field));
    }

    Ok(())
}
