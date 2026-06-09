// This file exceeds 700 lines because it groups the application settings
// store with all typed setting definitions, validation, and defaults.
// Settings are allowlisted and type-checked; splitting the definitions
// from the store would require duplicating the type registry.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use thiserror::Error;

use crate::platform::config::AppConfig;

const SECRET_LIKE_MARKERS: [&str; 5] = ["secret", "password", "token", "credential", "private_key"];

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

        Ok(AiRuntimeSettings {
            base_url: string_value(&settings, "ai.ollama_base_url")
                .unwrap_or_else(|| fallback.ollama_base_url().to_owned()),
            chat_model: string_value(&settings, "ai.chat_model")
                .unwrap_or_else(|| fallback.ollama_chat_model().to_owned()),
            embedding_model: string_value(&settings, "ai.embedding_model")
                .unwrap_or_else(|| fallback.ollama_embed_model().to_owned()),
            timeout_seconds: integer_value(&settings, "ai.timeout_seconds")
                .and_then(|value| u64::try_from(value).ok())
                .filter(|value| *value > 0)
                .unwrap_or_else(|| fallback.ollama_timeout_seconds()),
        })
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
struct DeclaredApplicationSetting {
    setting_key: &'static str,
    category: &'static str,
    value_kind: SettingValueKind,
    default_value: Value,
    label: &'static str,
    description: &'static str,
    metadata: Value,
    is_editable: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ExistingApplicationSettingRow {
    category: String,
    value_kind: String,
    value: Value,
    label: String,
    description: String,
    metadata: Value,
    is_editable: bool,
}

impl ExistingApplicationSettingRow {
    fn is_value_compatible_with(&self, declared: &DeclaredApplicationSetting) -> bool {
        SettingValueKind::try_from(self.value_kind.as_str())
            .is_ok_and(|value_kind| value_kind == declared.value_kind)
            && declared
                .value_kind
                .validate_value(&self.value, &declared.metadata)
                .is_ok()
    }

    fn needs_repair(&self, declared: &DeclaredApplicationSetting, next_value: &Value) -> bool {
        self.category != declared.category
            || self.value_kind != declared.value_kind.db_value()
            || &self.value != next_value
            || self.label != declared.label
            || self.description != declared.description
            || self.metadata != declared.metadata
            || self.is_editable != declared.is_editable
    }
}

fn declared_setting_keys() -> Vec<String> {
    declared_application_settings()
        .into_iter()
        .map(|setting| setting.setting_key.to_owned())
        .collect()
}

fn declared_setting(setting_key: &str) -> Option<DeclaredApplicationSetting> {
    declared_application_settings()
        .into_iter()
        .find(|setting| setting.setting_key == setting_key)
}

fn declared_application_settings() -> Vec<DeclaredApplicationSetting> {
    vec![
        DeclaredApplicationSetting {
            setting_key: "server.http_addr",
            category: "server",
            value_kind: SettingValueKind::String,
            default_value: json!("127.0.0.1:8080"),
            label: "Backend HTTP bind",
            description: "Backend HTTP address used when the local server starts. Changes require a backend restart.",
            metadata: json!({
                "ui_control": "text",
                "placeholder": "127.0.0.1:8080",
                "restart_required": true,
                "bootstrap": true,
                "env_var": "HERMES_HTTP_ADDR"
            }),
            is_editable: true,
        },
        DeclaredApplicationSetting {
            setting_key: "frontend.api_base_url",
            category: "frontend",
            value_kind: SettingValueKind::String,
            default_value: json!("http://127.0.0.1:8080"),
            label: "Frontend API base URL",
            description: "Backend URL used by the desktop shell after it has loaded local settings.",
            metadata: json!({
                "ui_control": "text",
                "placeholder": "http://127.0.0.1:8080",
                "bootstrap": true,
                "env_var": "VITE_HERMES_API_BASE_URL"
            }),
            is_editable: true,
        },
        DeclaredApplicationSetting {
            setting_key: "frontend.layout",
            category: "frontend",
            value_kind: SettingValueKind::Json,
            default_value: json!({
                "schemaVersion": 2,
                "views": {}
            }),
            label: "Frontend layout",
            description: "Desktop widget layout preset selections and user overrides. Stores layout metadata only, never message bodies, document text or secrets.",
            metadata: json!({
                "ui_control": "json",
                "schema_version": 2,
                "stores_private_content": false,
                "restart_required": false
            }),
            is_editable: true,
        },
        DeclaredApplicationSetting {
            setting_key: "frontend.sidebar",
            category: "frontend",
            value_kind: SettingValueKind::Json,
            default_value: json!({
                "schemaVersion": 3,
                "rootItemIds": [
                    "home",
                    "group:communications",
                    "persons",
                    "projects",
                    "tasks",
                    "calendar",
                    "documents",
                    "notes",
                    "knowledge",
                    "agents"
                ],
                "groups": [
                    {
                        "id": "communications",
                        "label": "Communications",
                        "icon": "tabler:messages",
                        "itemIds": [
                            "communications.mail",
                            "communications.telegram",
                            "communications.whatsapp",
                            "communications.calls",
                            "communications.meetings",
                            "timeline"
                        ],
                        "separatorBeforeItemIds": []
                    }
                ],
                "hiddenItemIds": []
            }),
            label: "Frontend sidebar",
            description: "Desktop sidebar grouping, item order and hidden workspace metadata. Stores navigation preferences only, never message bodies, document text or secrets.",
            metadata: json!({
                "ui_control": "json",
                "schema_version": 3,
                "stores_private_content": false,
                "restart_required": false
            }),
            is_editable: true,
        },
        DeclaredApplicationSetting {
            setting_key: "frontend.theme",
            category: "frontend",
            value_kind: SettingValueKind::Json,
            default_value: json!({
                "schemaVersion": 1,
                "shellBackground": "network-mesh",
                "backgroundBrightness": 70,
                "accentColor": "teal",
                "panelOpacity": 70,
                "panelBlur": 12
            }),
            label: "Frontend appearance",
            description: "Desktop shell background, image brightness, panel transparency, panel blur and accent color. Stores visual preferences only, never message bodies, document text or secrets.",
            metadata: json!({
                "ui_control": "appearance",
                "schema_version": 1,
                "allowed_backgrounds": [
                    "none",
                    "network-mesh",
                    "data-stream",
                    "node-frame",
                    "eclipse-grid",
                    "dna-blueprint",
                    "forest-network",
                    "forest-stream",
                    "knowledge-map",
                    "rune-gold",
                    "rune-teal"
                ],
                "allowed_brightness": [30, 40, 50, 60, 70, 80, 90, 100],
                "allowed_accent_colors": ["teal", "cyan", "blue", "violet", "amber", "rose"],
                "allowed_panel_opacity": [40, 50, 60, 70, 80, 90, 100],
                "allowed_panel_blur": [0, 4, 8, 12, 16, 20, 24],
                "stores_private_content": false,
                "restart_required": false
            }),
            is_editable: true,
        },
        DeclaredApplicationSetting {
            setting_key: "ai.ollama_base_url",
            category: "ai",
            value_kind: SettingValueKind::String,
            default_value: json!("http://127.0.0.1:11434"),
            label: "Ollama base URL",
            description: "Local Ollama HTTP endpoint used by AI runtime requests.",
            metadata: json!({
                "ui_control": "text",
                "placeholder": "http://127.0.0.1:11434"
            }),
            is_editable: true,
        },
        DeclaredApplicationSetting {
            setting_key: "ai.chat_model",
            category: "ai",
            value_kind: SettingValueKind::String,
            default_value: json!("qwen3:4b"),
            label: "Chat model",
            description: "Ollama model used for chat and source-backed answers.",
            metadata: json!({
                "ui_control": "text",
                "placeholder": "qwen3:4b"
            }),
            is_editable: true,
        },
        DeclaredApplicationSetting {
            setting_key: "ai.embedding_model",
            category: "ai",
            value_kind: SettingValueKind::String,
            default_value: json!("qwen3-embedding:4b"),
            label: "Embedding model",
            description: "Ollama model used for semantic embeddings.",
            metadata: json!({
                "ui_control": "text",
                "placeholder": "qwen3-embedding:4b"
            }),
            is_editable: true,
        },
        DeclaredApplicationSetting {
            setting_key: "ai.timeout_seconds",
            category: "ai",
            value_kind: SettingValueKind::Integer,
            default_value: json!(120),
            label: "AI request timeout",
            description: "Timeout in seconds for Ollama HTTP requests.",
            metadata: json!({
                "ui_control": "number",
                "min": 1,
                "max": 600,
                "step": 1
            }),
            is_editable: true,
        },
        DeclaredApplicationSetting {
            setting_key: "ui.theme",
            category: "ui",
            value_kind: SettingValueKind::String,
            default_value: json!("system"),
            label: "Theme",
            description: "Desktop shell color theme preference.",
            metadata: json!({
                "ui_control": "select",
                "allowed_values": ["system", "dark", "light"]
            }),
            is_editable: true,
        },
        DeclaredApplicationSetting {
            setting_key: "ui.density",
            category: "ui",
            value_kind: SettingValueKind::String,
            default_value: json!("comfortable"),
            label: "UI density",
            description: "Desktop shell spacing density preference.",
            metadata: json!({
                "ui_control": "select",
                "allowed_values": ["comfortable", "compact"]
            }),
            is_editable: true,
        },
    ]
}

async fn ensure_application_settings_table(pool: &PgPool) -> Result<(), SettingsError> {
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

async fn fetch_existing_setting_row(
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

async fn insert_declared_setting(
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

async fn update_declared_setting(
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

fn validate_declared_setting(declared: &DeclaredApplicationSetting) -> Result<(), SettingsError> {
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
    fn db_value(self) -> &'static str {
        match self {
            Self::Boolean => "boolean",
            Self::Integer => "integer",
            Self::String => "string",
            Self::Json => "json",
        }
    }

    fn validate_value(self, value: &Value, metadata: &Value) -> Result<(), SettingsError> {
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AiRuntimeSettings {
    pub base_url: String,
    pub chat_model: String,
    pub embedding_model: String,
    pub timeout_seconds: u64,
}

impl AiRuntimeSettings {
    pub fn from_config(config: &AppConfig) -> Self {
        Self {
            base_url: config.ollama_base_url().to_owned(),
            chat_model: config.ollama_chat_model().to_owned(),
            embedding_model: config.ollama_embed_model().to_owned(),
            timeout_seconds: config.ollama_timeout_seconds(),
        }
    }
}

fn row_to_setting(row: PgRow) -> Result<ApplicationSetting, SettingsError> {
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

fn string_value(settings: &[ApplicationSetting], setting_key: &str) -> Option<String> {
    settings
        .iter()
        .find(|setting| setting.setting_key == setting_key)
        .and_then(|setting| setting.value.as_str())
        .map(str::to_owned)
}

fn integer_value(settings: &[ApplicationSetting], setting_key: &str) -> Option<i64> {
    settings
        .iter()
        .find(|setting| setting.setting_key == setting_key)
        .and_then(|setting| setting.value.as_i64())
}

fn validate_setting_key(setting_key: &str) -> Result<(), SettingsError> {
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

fn validate_non_empty(field: &'static str, value: &str) -> Result<(), SettingsError> {
    if value.trim().is_empty() {
        return Err(SettingsError::EmptyField(field));
    }

    Ok(())
}

#[derive(Debug, Error)]
pub enum SettingsError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error("unsupported setting value kind: {0}")]
    UnsupportedValueKind(String),

    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error("setting key has invalid format")]
    InvalidSettingKey,

    #[error("setting key must not refer to secrets or credentials")]
    SecretLikeSettingKey,

    #[error("invalid setting value: {0}")]
    InvalidValue(&'static str),

    #[error("application setting was not found: {setting_key}")]
    SettingNotFound { setting_key: String },

    #[error("application setting is read-only: {setting_key}")]
    ReadOnlySetting { setting_key: String },
}

impl SettingsError {
    pub fn is_invalid_request(&self) -> bool {
        matches!(
            self,
            Self::UnsupportedValueKind(_)
                | Self::EmptyField(_)
                | Self::InvalidSettingKey
                | Self::SecretLikeSettingKey
                | Self::InvalidValue(_)
                | Self::ReadOnlySetting { .. }
        )
    }
}
