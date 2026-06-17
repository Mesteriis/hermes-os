use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::PgPool;

use super::chat_reconciliation::{
    expected_archive_state_for_command_kind, expected_marked_as_unread_state_for_command_kind,
    expected_mute_state_for_command_kind, expected_pin_state_for_command_kind,
    reconcile_dialog_boolean_commands_from_provider_state,
};
use super::errors::TelegramError;
use super::lifecycle::mark_command_reconciled;
use super::models::messages::TelegramProviderWriteCommand;
use super::rows::row_to_telegram_provider_write_command;
use super::store::TelegramStore;

const DIALOG_PIN_PROVIDER_MISMATCH_ERROR: &str =
    "Provider observed a different dialog pin state than requested";
const DIALOG_ARCHIVE_PROVIDER_MISMATCH_ERROR: &str =
    "Provider observed a different archive state than requested";
const DIALOG_MUTE_PROVIDER_MISMATCH_ERROR: &str =
    "Provider observed a different mute state than requested";
const DIALOG_MARK_UNREAD_PROVIDER_MISMATCH_ERROR: &str =
    "Provider observed a different unread state than requested";

impl TelegramStore {
    pub async fn apply_provider_marked_as_unread(
        &self,
        telegram_chat_id: &str,
        is_marked_as_unread: bool,
        source_event: &str,
    ) -> Result<serde_json::Value, TelegramError> {
        let mut metadata = self.chat_metadata_map(telegram_chat_id).await?;
        metadata.insert(
            "is_marked_as_unread".to_owned(),
            serde_json::Value::Bool(is_marked_as_unread),
        );
        metadata.insert(
            "marked_as_unread_source".to_owned(),
            serde_json::Value::String(source_event.to_owned()),
        );
        self.persist_chat_metadata(telegram_chat_id, metadata).await
    }

    pub async fn apply_provider_notification_settings(
        &self,
        telegram_chat_id: &str,
        use_default_mute_for: bool,
        mute_for: i64,
        source_event: &str,
    ) -> Result<serde_json::Value, TelegramError> {
        let mut metadata = self.chat_metadata_map(telegram_chat_id).await?;
        let is_muted = !use_default_mute_for && mute_for > 0;
        metadata.insert("is_muted".to_owned(), serde_json::Value::Bool(is_muted));
        metadata.insert(
            "tdlib_notification_settings".to_owned(),
            json!({
                "use_default_mute_for": use_default_mute_for,
                "mute_for": mute_for.max(0),
            }),
        );
        metadata.insert(
            "mute_source".to_owned(),
            serde_json::Value::String(source_event.to_owned()),
        );
        self.persist_chat_metadata(telegram_chat_id, metadata).await
    }

    pub async fn apply_provider_chat_position(
        &self,
        telegram_chat_id: &str,
        position: &TelegramProviderChatPositionUpdate,
    ) -> Result<serde_json::Value, TelegramError> {
        let mut metadata = self.chat_metadata_map(telegram_chat_id).await?;
        let mut positions = metadata
            .remove("tdlib_chat_positions")
            .and_then(|value| value.as_object().cloned())
            .unwrap_or_default();

        match position.list_kind.as_str() {
            "main" | "archive" => {
                if position.order > 0 {
                    positions.insert(
                        position.list_kind.clone(),
                        json!({
                            "order": position.order,
                            "is_pinned": position.is_pinned,
                        }),
                    );
                } else {
                    positions.remove(&position.list_kind);
                }
            }
            "folder" => {
                let mut folder_ids = positions
                    .get("folder_ids")
                    .and_then(serde_json::Value::as_array)
                    .cloned()
                    .unwrap_or_default();
                let folder_id = position.provider_folder_id.ok_or_else(|| {
                    TelegramError::InvalidRequest(
                        "folder chat position update missing provider_folder_id".to_owned(),
                    )
                })?;
                let folder_value = serde_json::Value::Number(folder_id.into());
                if position.order > 0 {
                    if !folder_ids.iter().any(|value| value == &folder_value) {
                        folder_ids.push(folder_value);
                    }
                } else {
                    folder_ids.retain(|value| value != &folder_value);
                }
                positions.insert(
                    "folder_ids".to_owned(),
                    serde_json::Value::Array(folder_ids),
                );
            }
            _ => {}
        }

        let is_archived = positions
            .get("archive")
            .and_then(serde_json::Value::as_object)
            .and_then(|archive| archive.get("order"))
            .and_then(serde_json::Value::as_i64)
            .is_some_and(|order| order > 0);
        let is_pinned = ["main", "archive"]
            .into_iter()
            .filter_map(|key| positions.get(key))
            .filter_map(serde_json::Value::as_object)
            .any(|value| {
                value
                    .get("is_pinned")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false)
            });

        if position.list_kind == "folder" {
            let folder_ids = positions
                .get("folder_ids")
                .and_then(serde_json::Value::as_array)
                .cloned()
                .unwrap_or_default();
            let primary_folder_id = folder_ids.first().and_then(serde_json::Value::as_i64);
            if folder_ids.is_empty() {
                metadata.remove("folder_labels");
                metadata.remove("folder_name");
                metadata.remove("provider_folder_id");
                metadata.remove("provider_folder_ids");
            } else {
                let existing_labels = metadata
                    .get("folder_labels")
                    .and_then(serde_json::Value::as_array)
                    .cloned()
                    .unwrap_or_default();
                if existing_labels.is_empty() {
                    let fallback_labels = folder_ids
                        .iter()
                        .filter_map(serde_json::Value::as_i64)
                        .map(|folder_id| {
                            serde_json::Value::String(format!("Unknown folder {folder_id}"))
                        })
                        .collect::<Vec<_>>();
                    if let Some(primary_label) =
                        fallback_labels.first().and_then(|value| value.as_str())
                    {
                        metadata.insert(
                            "folder_name".to_owned(),
                            serde_json::Value::String(primary_label.to_owned()),
                        );
                    }
                    metadata.insert(
                        "folder_labels".to_owned(),
                        serde_json::Value::Array(fallback_labels),
                    );
                }
                metadata.insert(
                    "provider_folder_ids".to_owned(),
                    serde_json::Value::Array(folder_ids.clone()),
                );
                if let Some(primary_folder_id) = primary_folder_id {
                    metadata.insert(
                        "provider_folder_id".to_owned(),
                        serde_json::Value::Number(primary_folder_id.into()),
                    );
                }
            }
        }
        metadata.insert(
            "tdlib_chat_positions".to_owned(),
            serde_json::Value::Object(positions),
        );
        metadata.insert(
            "is_archived".to_owned(),
            serde_json::Value::Bool(is_archived),
        );
        metadata.insert("is_pinned".to_owned(), serde_json::Value::Bool(is_pinned));
        metadata.insert(
            "archive_source".to_owned(),
            serde_json::Value::String(position.source_event.clone()),
        );
        self.persist_chat_metadata(telegram_chat_id, metadata).await
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TelegramProviderChatPositionUpdate {
    pub list_kind: String,
    pub provider_folder_id: Option<i64>,
    pub order: i64,
    pub is_pinned: bool,
    pub source_event: String,
}

pub async fn reconcile_marked_as_unread_commands_from_provider_state(
    pool: &PgPool,
    account_id: &str,
    provider_chat_id: &str,
    is_marked_as_unread: bool,
    observed_at: DateTime<Utc>,
    observed_via: &str,
) -> Result<Vec<TelegramProviderWriteCommand>, TelegramError> {
    reconcile_dialog_boolean_commands_from_provider_state(
        pool,
        account_id,
        provider_chat_id,
        "is_marked_as_unread",
        "expected_is_marked_as_unread",
        "observed_is_marked_as_unread",
        is_marked_as_unread,
        observed_at,
        observed_via,
        DIALOG_MARK_UNREAD_PROVIDER_MISMATCH_ERROR,
        expected_marked_as_unread_state_for_command_kind,
        &[],
    )
    .await
}

pub async fn reconcile_mark_read_commands_from_provider_state(
    pool: &PgPool,
    account_id: &str,
    provider_chat_id: &str,
    last_read_inbox_message_id: &str,
    observed_at: DateTime<Utc>,
    observed_via: &str,
) -> Result<Vec<TelegramProviderWriteCommand>, TelegramError> {
    let Some(observed_message_id) =
        telegram_provider_message_numeric_suffix(last_read_inbox_message_id)
    else {
        return Ok(Vec::new());
    };
    let rows = sqlx::query(
        r#"
        SELECT *
        FROM telegram_provider_write_commands
        WHERE account_id = $1
          AND provider_chat_id = $2
          AND command_kind = 'mark_read'
          AND status IN ('queued', 'retrying', 'executing')
          AND provider_message_id IS NOT NULL
          AND confirmation_decision IN ('confirmed', 'not_required')
          AND capability_state IN ('available', 'degraded')
          AND happened_at <= $3
        ORDER BY happened_at ASC
        "#,
    )
    .bind(account_id)
    .bind(provider_chat_id)
    .bind(observed_at)
    .fetch_all(pool)
    .await
    .map_err(TelegramError::from)?;

    let mut reconciled = Vec::new();
    for row in rows {
        let command = row_to_telegram_provider_write_command(row)?;
        let Some(target_message_id) = telegram_provider_message_numeric_suffix(
            command.provider_message_id.as_deref().unwrap_or_default(),
        ) else {
            continue;
        };
        if target_message_id > observed_message_id {
            continue;
        }
        super::commands::mark_command_reconciled(
            pool,
            &command.command_id,
            observed_at,
            json!({
                "provider_chat_id": provider_chat_id,
                "last_read_inbox_message_id": last_read_inbox_message_id,
                "observed_via": observed_via,
            }),
            json!({
                "source": observed_via,
                "provider_chat_id": provider_chat_id,
                "last_read_inbox_message_id": last_read_inbox_message_id,
                "provider_observed_at": observed_at,
            }),
        )
        .await?;
        let refreshed =
            sqlx::query("SELECT * FROM telegram_provider_write_commands WHERE command_id = $1")
                .bind(&command.command_id)
                .fetch_one(pool)
                .await
                .map_err(TelegramError::from)?;
        reconciled.push(row_to_telegram_provider_write_command(refreshed)?);
    }
    Ok(reconciled)
}

pub async fn reconcile_mute_commands_from_provider_state(
    pool: &PgPool,
    account_id: &str,
    provider_chat_id: &str,
    use_default_mute_for: bool,
    mute_for: i64,
    observed_at: DateTime<Utc>,
    observed_via: &str,
) -> Result<Vec<TelegramProviderWriteCommand>, TelegramError> {
    reconcile_dialog_boolean_commands_from_provider_state(
        pool,
        account_id,
        provider_chat_id,
        "is_muted",
        "expected_is_muted",
        "observed_is_muted",
        !use_default_mute_for && mute_for > 0,
        observed_at,
        observed_via,
        DIALOG_MUTE_PROVIDER_MISMATCH_ERROR,
        expected_mute_state_for_command_kind,
        &[
            ("use_default_mute_for", json!(use_default_mute_for)),
            ("mute_for", json!(mute_for)),
        ],
    )
    .await
}

pub async fn reconcile_archive_commands_from_provider_state(
    pool: &PgPool,
    account_id: &str,
    provider_chat_id: &str,
    is_archived: bool,
    observed_at: DateTime<Utc>,
    observed_via: &str,
) -> Result<Vec<TelegramProviderWriteCommand>, TelegramError> {
    reconcile_dialog_boolean_commands_from_provider_state(
        pool,
        account_id,
        provider_chat_id,
        "is_archived",
        "expected_is_archived",
        "observed_is_archived",
        is_archived,
        observed_at,
        observed_via,
        DIALOG_ARCHIVE_PROVIDER_MISMATCH_ERROR,
        expected_archive_state_for_command_kind,
        &[],
    )
    .await
}

pub async fn reconcile_folder_add_commands_from_provider_state(
    pool: &PgPool,
    account_id: &str,
    provider_chat_id: &str,
    provider_folder_id: i64,
    observed_at: DateTime<Utc>,
    observed_via: &str,
) -> Result<Vec<TelegramProviderWriteCommand>, TelegramError> {
    let rows = sqlx::query(
        r#"
        SELECT *
        FROM telegram_provider_write_commands
        WHERE account_id = $1
          AND provider_chat_id = $2
          AND command_kind = 'folder_add'
          AND status IN ('queued', 'retrying', 'executing')
          AND confirmation_decision IN ('confirmed', 'not_required')
          AND capability_state IN ('available', 'degraded')
          AND (payload->>'provider_folder_id')::bigint = $3
          AND happened_at <= $4
        ORDER BY happened_at ASC
        "#,
    )
    .bind(account_id)
    .bind(provider_chat_id)
    .bind(provider_folder_id)
    .bind(observed_at)
    .fetch_all(pool)
    .await
    .map_err(TelegramError::from)?;

    let mut reconciled = Vec::new();
    for row in rows {
        let command = row_to_telegram_provider_write_command(row)?;
        mark_command_reconciled(
            pool,
            &command.command_id,
            observed_at,
            json!({
                "provider_chat_id": provider_chat_id,
                "provider_folder_id": provider_folder_id,
                "observed_via": observed_via,
            }),
            json!({
                "source": observed_via,
                "provider_chat_id": provider_chat_id,
                "provider_folder_id": provider_folder_id,
                "provider_observed_at": observed_at,
            }),
        )
        .await?;
        let refreshed =
            sqlx::query("SELECT * FROM telegram_provider_write_commands WHERE command_id = $1")
                .bind(&command.command_id)
                .fetch_one(pool)
                .await
                .map_err(TelegramError::from)?;
        reconciled.push(row_to_telegram_provider_write_command(refreshed)?);
    }

    Ok(reconciled)
}

pub async fn reconcile_folder_remove_commands_from_provider_state(
    pool: &PgPool,
    account_id: &str,
    provider_chat_id: &str,
    provider_folder_id: i64,
    observed_at: DateTime<Utc>,
    observed_via: &str,
) -> Result<Vec<TelegramProviderWriteCommand>, TelegramError> {
    let rows = sqlx::query(
        r#"
        SELECT *
        FROM telegram_provider_write_commands
        WHERE account_id = $1
          AND provider_chat_id = $2
          AND command_kind = 'folder_remove'
          AND status IN ('queued', 'retrying', 'executing')
          AND confirmation_decision IN ('confirmed', 'not_required')
          AND capability_state IN ('available', 'degraded')
          AND (payload->>'provider_folder_id')::bigint = $3
          AND happened_at <= $4
        ORDER BY happened_at ASC
        "#,
    )
    .bind(account_id)
    .bind(provider_chat_id)
    .bind(provider_folder_id)
    .bind(observed_at)
    .fetch_all(pool)
    .await
    .map_err(TelegramError::from)?;

    let mut reconciled = Vec::new();
    for row in rows {
        let command = row_to_telegram_provider_write_command(row)?;
        mark_command_reconciled(
            pool,
            &command.command_id,
            observed_at,
            json!({
                "provider_chat_id": provider_chat_id,
                "provider_folder_id": provider_folder_id,
                "observed_via": observed_via,
            }),
            json!({
                "source": observed_via,
                "provider_chat_id": provider_chat_id,
                "provider_folder_id": provider_folder_id,
                "provider_observed_at": observed_at,
            }),
        )
        .await?;
        let refreshed =
            sqlx::query("SELECT * FROM telegram_provider_write_commands WHERE command_id = $1")
                .bind(&command.command_id)
                .fetch_one(pool)
                .await
                .map_err(TelegramError::from)?;
        reconciled.push(row_to_telegram_provider_write_command(refreshed)?);
    }

    Ok(reconciled)
}

pub async fn reconcile_pin_commands_from_provider_state(
    pool: &PgPool,
    account_id: &str,
    provider_chat_id: &str,
    is_pinned: bool,
    observed_at: DateTime<Utc>,
    observed_via: &str,
) -> Result<Vec<TelegramProviderWriteCommand>, TelegramError> {
    reconcile_dialog_boolean_commands_from_provider_state(
        pool,
        account_id,
        provider_chat_id,
        "is_pinned",
        "expected_is_pinned",
        "observed_is_pinned",
        is_pinned,
        observed_at,
        observed_via,
        DIALOG_PIN_PROVIDER_MISMATCH_ERROR,
        expected_pin_state_for_command_kind,
        &[],
    )
    .await
}

async fn reconcile_chat_commands_by_kind(
    pool: &PgPool,
    account_id: &str,
    provider_chat_id: &str,
    command_kind: &str,
    observed_at: DateTime<Utc>,
    provider_state: serde_json::Value,
    result_payload: serde_json::Value,
) -> Result<Vec<TelegramProviderWriteCommand>, TelegramError> {
    let rows = sqlx::query(
        r#"
        UPDATE telegram_provider_write_commands
        SET status = 'completed',
            result_payload = $5,
            last_error = NULL,
            provider_observed_at = $4,
            provider_state = $6,
            reconciliation_status = 'observed',
            reconciled_at = $4,
            completed_at = $4,
            locked_at = NULL,
            locked_by = NULL,
            next_attempt_at = NULL,
            dead_lettered_at = NULL,
            updated_at = $4
        WHERE account_id = $1
          AND provider_chat_id = $2
          AND command_kind = $3
          AND status IN ('queued', 'retrying', 'executing')
          AND provider_message_id IS NULL
          AND confirmation_decision IN ('confirmed', 'not_required')
          AND capability_state IN ('available', 'degraded')
          AND happened_at <= $4
        RETURNING *
        "#,
    )
    .bind(account_id)
    .bind(provider_chat_id)
    .bind(command_kind)
    .bind(observed_at)
    .bind(&result_payload)
    .bind(&provider_state)
    .fetch_all(pool)
    .await
    .map_err(TelegramError::from)?;

    rows.into_iter()
        .map(row_to_telegram_provider_write_command)
        .collect()
}

fn telegram_provider_message_numeric_suffix(provider_message_id: &str) -> Option<i64> {
    provider_message_id
        .trim()
        .rsplit(':')
        .next()
        .unwrap_or_default()
        .trim()
        .parse::<i64>()
        .ok()
        .filter(|value| *value > 0)
}
