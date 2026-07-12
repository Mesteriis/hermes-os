use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::PgRow;

use super::errors::AutomationError;
use super::models::{AutomationPolicy, AutomationTemplate};

pub(super) fn row_to_template(row: PgRow) -> Result<AutomationTemplate, AutomationError> {
    Ok(AutomationTemplate {
        template_id: row.try_get("template_id")?,
        name: row.try_get("name")?,
        body_template: row.try_get("body_template")?,
        required_variables: string_vec_from_value(row.try_get("required_variables")?)?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

pub(super) fn row_to_policy(row: PgRow) -> Result<AutomationPolicy, AutomationError> {
    Ok(AutomationPolicy {
        policy_id: row.try_get("policy_id")?,
        template_id: row.try_get("template_id")?,
        name: row.try_get("name")?,
        enabled: row.try_get("enabled")?,
        account_id: row.try_get("account_id")?,
        allowed_chat_ids: string_vec_from_value(row.try_get("allowed_chat_ids")?)?,
        scopes: Vec::new(),
        trigger_kind: row.try_get("trigger_kind")?,
        max_sends_per_hour: row.try_get("max_sends_per_hour")?,
        quiet_hours: row.try_get("quiet_hours")?,
        expires_at: row.try_get("expires_at")?,
        conditions: row.try_get("conditions")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

pub(super) fn string_vec_from_value(value: Value) -> Result<Vec<String>, AutomationError> {
    let values = value
        .as_array()
        .ok_or_else(|| AutomationError::InvalidRequest("expected array".to_owned()))?;
    values
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(ToOwned::to_owned)
                .ok_or_else(|| AutomationError::InvalidRequest("expected string array".to_owned()))
        })
        .collect()
}
