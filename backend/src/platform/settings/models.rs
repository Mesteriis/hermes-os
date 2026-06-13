use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::SettingsError;
use super::validation::validate_json_metadata_constraints;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ApplicationSettingsRepairSummary {
    pub inserted: u64,
    pub repaired: u64,
    pub reset_values: u64,
}

impl ApplicationSettingsRepairSummary {
    pub fn changed(&self) -> bool {
        self.inserted > 0 || self.repaired > 0 || self.reset_values > 0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DeclaredApplicationSetting {
    pub(crate) setting_key: &'static str,
    pub(crate) category: &'static str,
    pub(crate) value_kind: SettingValueKind,
    pub(crate) default_value: Value,
    pub(crate) label: &'static str,
    pub(crate) description: &'static str,
    pub(crate) metadata: Value,
    pub(crate) is_editable: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ApplicationSetting {
    pub setting_key: String,
    pub category: String,
    pub value_kind: SettingValueKind,
    pub value: Value,
    pub label: String,
    pub description: String,
    pub metadata: Value,
    pub is_editable: bool,
    pub updated_by_actor_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SettingValueKind {
    Boolean,
    Integer,
    String,
    Json,
}

impl SettingValueKind {
    pub(crate) fn db_value(self) -> &'static str {
        match self {
            Self::Boolean => "boolean",
            Self::Integer => "integer",
            Self::String => "string",
            Self::Json => "json",
        }
    }

    pub(crate) fn validate_value(
        self,
        value: &Value,
        metadata: &Value,
    ) -> Result<(), SettingsError> {
        match self {
            Self::Boolean if !value.is_boolean() => {
                return Err(SettingsError::InvalidValue("value must be a boolean"));
            }
            Self::Integer if value.as_i64().is_none() => {
                return Err(SettingsError::InvalidValue("value must be an integer"));
            }
            Self::String if value.as_str().is_none() => {
                return Err(SettingsError::InvalidValue("value must be a string"));
            }
            Self::Json if !(value.is_object() || value.is_array()) => {
                return Err(SettingsError::InvalidValue(
                    "value must be a JSON object or array",
                ));
            }
            _ => {}
        }

        if let Some(allowed_values) = metadata.get("allowed_values").and_then(Value::as_array) {
            let is_allowed = allowed_values.iter().any(|allowed| allowed == value);
            if !is_allowed {
                return Err(SettingsError::InvalidValue(
                    "value is not allowed for this setting",
                ));
            }
        }

        if let Some(value) = value.as_i64() {
            if let Some(min) = metadata.get("min").and_then(Value::as_i64)
                && value < min
            {
                return Err(SettingsError::InvalidValue(
                    "value is below the allowed minimum",
                ));
            }
            if let Some(max) = metadata.get("max").and_then(Value::as_i64)
                && value > max
            {
                return Err(SettingsError::InvalidValue(
                    "value is above the allowed maximum",
                ));
            }
        }

        validate_json_metadata_constraints(value, metadata)?;

        Ok(())
    }
}

impl TryFrom<&str> for SettingValueKind {
    type Error = SettingsError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.trim() {
            "boolean" => Ok(Self::Boolean),
            "integer" => Ok(Self::Integer),
            "string" => Ok(Self::String),
            "json" => Ok(Self::Json),
            _ => Err(SettingsError::UnsupportedValueKind(value.to_owned())),
        }
    }
}
