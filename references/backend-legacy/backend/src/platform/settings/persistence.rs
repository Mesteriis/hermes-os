use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};

use super::errors::SettingsError;
use super::models::{ApplicationSetting, DeclaredApplicationSetting, SettingValueKind};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ExistingApplicationSettingRow {
    pub(crate) category: String,
    pub(crate) value_kind: String,
    pub(crate) value: Value,
    pub(crate) label: String,
    pub(crate) description: String,
    pub(crate) metadata: Value,
    pub(crate) is_editable: bool,
}

impl ExistingApplicationSettingRow {
    pub(crate) fn is_value_compatible_with(&self, declared: &DeclaredApplicationSetting) -> bool {
        SettingValueKind::try_from(self.value_kind.as_str())
            .is_ok_and(|value_kind| value_kind == declared.value_kind)
            && declared
                .value_kind
                .validate_value(&self.value, &declared.metadata)
                .is_ok()
    }

    pub(crate) fn needs_repair(
        &self,
        declared: &DeclaredApplicationSetting,
        next_value: &Value,
    ) -> bool {
        self.category != declared.category
            || self.value_kind != declared.value_kind.db_value()
            || &self.value != next_value
            || self.label != declared.label
            || self.description != declared.description
            || self.metadata != declared.metadata
            || self.is_editable != declared.is_editable
    }
}

pub(crate) async fn ensure_application_settings_table(pool: &PgPool) -> Result<(), SettingsError> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS application_settings (
            setting_key TEXT PRIMARY KEY,
            category TEXT NOT NULL,
            value_kind TEXT NOT NULL,
            value JSONB NOT NULL,
            label TEXT NOT NULL,
            description TEXT NOT NULL DEFAULT '',
            metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
            is_editable BOOLEAN NOT NULL DEFAULT true,
            updated_by_actor_id TEXT,
            created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

            CONSTRAINT application_settings_key_not_empty CHECK (length(trim(setting_key)) > 0),
            CONSTRAINT application_settings_key_format CHECK (setting_key ~ '^[a-z0-9][a-z0-9_.-]*[a-z0-9]$'),
            CONSTRAINT application_settings_key_not_secret_like CHECK (
                setting_key !~* '(secret|password|token|credential|private_key)'
            ),
            CONSTRAINT application_settings_category_not_empty CHECK (length(trim(category)) > 0),
            CONSTRAINT application_settings_label_not_empty CHECK (length(trim(label)) > 0),
            CONSTRAINT application_settings_value_kind CHECK (
                value_kind IN ('boolean', 'integer', 'string', 'json')
            ),
            CONSTRAINT application_settings_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS application_settings_category_idx
            ON application_settings (category, setting_key)
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub(crate) async fn fetch_existing_setting_row(
    pool: &PgPool,
    setting_key: &str,
) -> Result<Option<ExistingApplicationSettingRow>, SettingsError> {
    let Some(row) = sqlx::query(
        r#"
        SELECT
            category,
            value_kind,
            value,
            label,
            description,
            metadata,
            is_editable
        FROM application_settings
        WHERE setting_key = $1
        "#,
    )
    .bind(setting_key)
    .fetch_optional(pool)
    .await?
    else {
        return Ok(None);
    };

    Ok(Some(ExistingApplicationSettingRow {
        category: row.try_get("category")?,
        value_kind: row.try_get("value_kind")?,
        value: row.try_get("value")?,
        label: row.try_get("label")?,
        description: row.try_get("description")?,
        metadata: row.try_get("metadata")?,
        is_editable: row.try_get("is_editable")?,
    }))
}

pub(crate) async fn insert_declared_setting(
    pool: &PgPool,
    declared: &DeclaredApplicationSetting,
) -> Result<(), SettingsError> {
    sqlx::query(
        r#"
        INSERT INTO application_settings (
            setting_key,
            category,
            value_kind,
            value,
            label,
            description,
            metadata,
            is_editable,
            updated_by_actor_id
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'system:settings_repair')
        "#,
    )
    .bind(declared.setting_key)
    .bind(declared.category)
    .bind(declared.value_kind.db_value())
    .bind(&declared.default_value)
    .bind(declared.label)
    .bind(declared.description)
    .bind(&declared.metadata)
    .bind(declared.is_editable)
    .execute(pool)
    .await?;

    Ok(())
}

pub(crate) async fn update_declared_setting(
    pool: &PgPool,
    declared: &DeclaredApplicationSetting,
    next_value: &Value,
) -> Result<(), SettingsError> {
    sqlx::query(
        r#"
        UPDATE application_settings
        SET
            category = $2,
            value_kind = $3,
            value = $4,
            label = $5,
            description = $6,
            metadata = $7,
            is_editable = $8,
            updated_by_actor_id = 'system:settings_repair',
            updated_at = now()
        WHERE setting_key = $1
        "#,
    )
    .bind(declared.setting_key)
    .bind(declared.category)
    .bind(declared.value_kind.db_value())
    .bind(next_value)
    .bind(declared.label)
    .bind(declared.description)
    .bind(&declared.metadata)
    .bind(declared.is_editable)
    .execute(pool)
    .await?;

    Ok(())
}

pub(crate) fn row_to_setting(row: PgRow) -> Result<ApplicationSetting, SettingsError> {
    let value_kind = SettingValueKind::try_from(row.try_get::<String, _>("value_kind")?.as_str())?;

    Ok(ApplicationSetting {
        setting_key: row.try_get("setting_key")?,
        category: row.try_get("category")?,
        value_kind,
        value: row.try_get("value")?,
        label: row.try_get("label")?,
        description: row.try_get("description")?,
        metadata: row.try_get("metadata")?,
        is_editable: row.try_get("is_editable")?,
        updated_by_actor_id: row.try_get("updated_by_actor_id")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}
