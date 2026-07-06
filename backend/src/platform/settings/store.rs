use serde_json::Value;
use sqlx::postgres::PgPool;

use crate::platform::config::AppConfig;

use super::ai_runtime::{AiRuntimeSettings, runtime_settings_from_values};
use super::definitions::{declared_application_settings, declared_setting, declared_setting_keys};
use super::errors::SettingsError;
use super::models::{ApplicationSetting, ApplicationSettingsRepairSummary};
use super::persistence::{
    ensure_application_settings_table, fetch_existing_setting_row, insert_declared_setting,
    row_to_setting, update_declared_setting,
};
use super::validation::{validate_declared_setting, validate_non_empty, validate_setting_key};

const PUBLIC_SETTINGS_EXCLUDED_CATEGORIES: &[&str] = &["ai"];

#[derive(Clone)]
pub struct ApplicationSettingsStore {
    pool: PgPool,
}

impl ApplicationSettingsStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list_settings(&self) -> Result<Vec<ApplicationSetting>, SettingsError> {
        let setting_keys = declared_setting_keys();
        let rows = sqlx::query(
            r#"
            SELECT
                setting_key,
                category,
                value_kind,
                value,
                label,
                description,
                metadata,
                is_editable,
                updated_by_actor_id,
                created_at,
                updated_at
            FROM application_settings
            WHERE setting_key = ANY($1)
            ORDER BY category ASC, setting_key ASC
            "#,
        )
        .bind(&setting_keys)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_setting).collect()
    }

    pub async fn list_public_settings(&self) -> Result<Vec<ApplicationSetting>, SettingsError> {
        let settings = self.list_settings().await?;

        Ok(settings
            .into_iter()
            .filter(|setting| {
                !PUBLIC_SETTINGS_EXCLUDED_CATEGORIES.contains(&setting.category.as_str())
            })
            .collect())
    }

    pub async fn setting(
        &self,
        setting_key: &str,
    ) -> Result<Option<ApplicationSetting>, SettingsError> {
        validate_setting_key(setting_key)?;
        let setting_key = setting_key.trim();
        if declared_setting(setting_key).is_none() {
            return Ok(None);
        }

        let row = sqlx::query(
            r#"
            SELECT
                setting_key,
                category,
                value_kind,
                value,
                label,
                description,
                metadata,
                is_editable,
                updated_by_actor_id,
                created_at,
                updated_at
            FROM application_settings
            WHERE setting_key = $1
            "#,
        )
        .bind(setting_key)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_setting).transpose()
    }

    pub async fn update_setting_value(
        &self,
        setting_key: &str,
        value: &Value,
        actor_id: &str,
    ) -> Result<ApplicationSetting, SettingsError> {
        validate_setting_key(setting_key)?;
        validate_non_empty("actor_id", actor_id)?;
        let setting_key = setting_key.trim();

        if declared_setting(setting_key).is_none() {
            return Err(SettingsError::SettingNotFound {
                setting_key: setting_key.to_owned(),
            });
        };

        let existing = match self.setting(setting_key).await? {
            Some(setting) => setting,
            None => {
                self.repair_declared_settings().await?;
                self.setting(setting_key)
                    .await?
                    .ok_or_else(|| SettingsError::SettingNotFound {
                        setting_key: setting_key.to_owned(),
                    })?
            }
        };

        if !existing.is_editable {
            return Err(SettingsError::ReadOnlySetting {
                setting_key: existing.setting_key,
            });
        }
        existing
            .value_kind
            .validate_value(value, &existing.metadata)?;

        let row = sqlx::query(
            r#"
            UPDATE application_settings
            SET
                value = $2,
                updated_by_actor_id = $3,
                updated_at = now()
            WHERE setting_key = $1
            RETURNING
                setting_key,
                category,
                value_kind,
                value,
                label,
                description,
                metadata,
                is_editable,
                updated_by_actor_id,
                created_at,
                updated_at
            "#,
        )
        .bind(setting_key)
        .bind(value)
        .bind(actor_id.trim())
        .fetch_one(&self.pool)
        .await?;

        row_to_setting(row)
    }

    pub async fn ai_runtime_settings(
        &self,
        fallback: &AppConfig,
    ) -> Result<AiRuntimeSettings, SettingsError> {
        let settings = self.list_settings().await?;

        Ok(runtime_settings_from_values(&settings, fallback))
    }

    pub async fn repair_declared_settings(
        &self,
    ) -> Result<ApplicationSettingsRepairSummary, SettingsError> {
        ensure_application_settings_table(&self.pool).await?;

        let mut summary = ApplicationSettingsRepairSummary::default();
        for declared in declared_application_settings() {
            validate_declared_setting(&declared)?;
            let Some(existing) =
                fetch_existing_setting_row(&self.pool, declared.setting_key).await?
            else {
                insert_declared_setting(&self.pool, &declared).await?;
                summary.inserted += 1;
                continue;
            };

            let next_value = if existing.is_value_compatible_with(&declared) {
                existing.value.clone()
            } else {
                summary.reset_values += 1;
                declared.default_value.clone()
            };

            if existing.needs_repair(&declared, &next_value) {
                update_declared_setting(&self.pool, &declared, &next_value).await?;
                summary.repaired += 1;
            }
        }

        Ok(summary)
    }
}
