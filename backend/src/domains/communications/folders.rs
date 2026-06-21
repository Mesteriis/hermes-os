use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgPool, PgRow};
use sqlx::{Postgres, Row, Transaction};
use thiserror::Error;

use crate::domains::communications::evidence::{link_mail_entity_in_transaction, merge_metadata};
use crate::domains::communications::messages::{LocalMessageState, WorkflowState};
use crate::platform::events::EventStore;
use crate::platform::observations::ObservationStoreError;

mod cursors;
mod events;

use cursors::{
    decode_folder_list_cursor, decode_folder_message_cursor, encode_folder_list_cursor,
    encode_folder_message_cursor,
};
use events::{
    EVENT_TYPE_FOLDER_CREATED, EVENT_TYPE_FOLDER_DELETED, EVENT_TYPE_FOLDER_UPDATED, folder_event,
    folder_message_event,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CommunicationFolder {
    pub folder_id: String,
    pub account_id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub sort_order: i32,
    pub message_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct NewCommunicationFolder {
    pub folder_id: Option<String>,
    pub account_id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub sort_order: Option<i32>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct UpdateCommunicationFolder {
    pub account_id: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub color: Option<String>,
    pub sort_order: Option<i32>,
}

#[derive(Clone, Debug, Serialize)]
pub struct CommunicationFolderListPage {
    pub items: Vec<CommunicationFolder>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

#[derive(Clone, Debug)]
pub struct CommunicationFolderListQuery<'a> {
    pub account_id: Option<&'a str>,
    pub cursor: Option<&'a str>,
    pub limit: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FolderMessageOperation {
    Copy,
    Move,
}

impl FolderMessageOperation {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Copy => "copy",
            Self::Move => "move",
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct FolderMessageActionResponse {
    pub operation: FolderMessageOperation,
    pub folder_id: String,
    pub message_id: String,
    pub message: FolderMessage,
}

#[derive(Clone, Debug, Serialize)]
pub struct FolderMessage {
    pub folder_id: String,
    pub message_id: String,
    pub account_id: String,
    pub subject: String,
    pub sender: String,
    pub occurred_at: Option<DateTime<Utc>>,
    pub projected_at: DateTime<Utc>,
    pub workflow_state: WorkflowState,
    pub local_state: LocalMessageState,
    pub added_at: DateTime<Utc>,
    pub attachment_count: i64,
}

#[derive(Clone, Debug, Serialize)]
pub struct FolderMessagePage {
    pub items: Vec<FolderMessage>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

#[derive(Clone, Debug)]
pub struct FolderMessageListQuery<'a> {
    pub folder_id: &'a str,
    pub cursor: Option<&'a str>,
    pub limit: i64,
}

#[derive(Clone)]
pub struct CommunicationFolderStore {
    pool: PgPool,
}

impl CommunicationFolderStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(
        &self,
        query: CommunicationFolderListQuery<'_>,
    ) -> Result<CommunicationFolderListPage, CommunicationFolderError> {
        let limit = validate_limit(query.limit);
        let account_id = normalize_optional(query.account_id.map(str::to_owned))?;
        let cursor = query
            .cursor
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(decode_folder_list_cursor)
            .transpose()?;
        let fetch_limit = limit + 1;
        let rows = sqlx::query(
            r#"
            SELECT
                f.folder_id,
                f.account_id,
                f.name,
                f.description,
                f.color,
                f.sort_order,
                count(fm.message_id)::BIGINT AS message_count,
                f.created_at,
                f.updated_at
            FROM communication_folders f
            LEFT JOIN communication_folder_messages fm ON fm.folder_id = f.folder_id
            WHERE ($1::text IS NULL OR f.account_id = $1)
              AND (
                $2::integer IS NULL
                OR f.sort_order > $2
                OR (f.sort_order = $2 AND lower(f.name) > $3)
                OR (f.sort_order = $2 AND lower(f.name) = $3 AND f.folder_id > $4)
              )
            GROUP BY f.folder_id, f.account_id, f.name, f.description, f.color, f.sort_order, f.created_at, f.updated_at
            ORDER BY f.sort_order ASC, lower(f.name) ASC, f.folder_id ASC
            LIMIT $5
            "#,
        )
        .bind(account_id.as_deref())
        .bind(cursor.as_ref().map(|value| value.sort_order))
        .bind(cursor.as_ref().map(|value| value.name_lower.as_str()))
        .bind(cursor.as_ref().map(|value| value.folder_id.as_str()))
        .bind(fetch_limit)
        .fetch_all(&self.pool)
        .await?;
        let has_more = rows.len() > limit as usize;
        let items = rows
            .into_iter()
            .take(limit as usize)
            .map(row_to_folder)
            .collect::<Result<Vec<_>, _>>()?;
        let next_cursor = if has_more {
            items.last().map(encode_folder_list_cursor).transpose()?
        } else {
            None
        };

        Ok(CommunicationFolderListPage {
            items,
            next_cursor,
            has_more,
        })
    }

    pub async fn create(
        &self,
        input: NewCommunicationFolder,
    ) -> Result<CommunicationFolder, CommunicationFolderError> {
        self.create_with_observation(input, None, "folder_upsert", None)
            .await
    }

    pub async fn create_with_observation(
        &self,
        input: NewCommunicationFolder,
        observation_id: Option<&str>,
        relationship_kind: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<CommunicationFolder, CommunicationFolderError> {
        let normalized = NormalizedCommunicationFolderInput::from_new(input)?;
        let mut transaction = self.pool.begin().await?;
        ensure_canonical_account_in_transaction(&mut transaction, normalized.account_id.as_deref())
            .await?;
        let folder = insert_folder(&mut transaction, &normalized).await?;
        let event = folder_event(EVENT_TYPE_FOLDER_CREATED, &folder)?;
        EventStore::append_in_transaction(&mut transaction, &event).await?;
        if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
            let link_metadata = merge_metadata(
                serde_json::json!({
                    "operation": "folder_create",
                    "name": folder.name,
                }),
                metadata,
            );
            link_mail_entity_in_transaction(
                &mut transaction,
                observation_id,
                "mail_folder",
                folder.folder_id.clone(),
                relationship_kind,
                link_metadata,
                None,
            )
            .await?;
        }
        transaction.commit().await?;
        Ok(folder)
    }

    pub async fn update(
        &self,
        folder_id: &str,
        update: UpdateCommunicationFolder,
    ) -> Result<Option<CommunicationFolder>, CommunicationFolderError> {
        self.update_with_observation(folder_id, update, None, "folder_upsert", None)
            .await
    }

    pub async fn update_with_observation(
        &self,
        folder_id: &str,
        update: UpdateCommunicationFolder,
        observation_id: Option<&str>,
        relationship_kind: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<Option<CommunicationFolder>, CommunicationFolderError> {
        let folder_id = normalize_required("folder_id", folder_id)?;
        let normalized = NormalizedCommunicationFolderUpdate::from_update(update)?;
        let mut transaction = self.pool.begin().await?;
        ensure_canonical_account_in_transaction(&mut transaction, normalized.account_id.as_deref())
            .await?;
        let Some(folder) = update_folder(&mut transaction, &folder_id, &normalized).await? else {
            transaction.rollback().await?;
            return Ok(None);
        };
        let event = folder_event(EVENT_TYPE_FOLDER_UPDATED, &folder)?;
        EventStore::append_in_transaction(&mut transaction, &event).await?;
        if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
            let link_metadata = merge_metadata(
                serde_json::json!({
                    "operation": "folder_update",
                    "name": folder.name,
                }),
                metadata,
            );
            link_mail_entity_in_transaction(
                &mut transaction,
                observation_id,
                "mail_folder",
                folder.folder_id.clone(),
                relationship_kind,
                link_metadata,
                None,
            )
            .await?;
        }
        transaction.commit().await?;
        Ok(Some(folder))
    }

    pub async fn delete(&self, folder_id: &str) -> Result<bool, CommunicationFolderError> {
        self.delete_with_observation(folder_id, None, "folder_delete", None)
            .await
    }

    pub async fn delete_with_observation(
        &self,
        folder_id: &str,
        observation_id: Option<&str>,
        relationship_kind: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<bool, CommunicationFolderError> {
        let folder_id = normalize_required("folder_id", folder_id)?;
        let mut transaction = self.pool.begin().await?;
        let Some(folder) = delete_folder(&mut transaction, &folder_id).await? else {
            transaction.rollback().await?;
            return Ok(false);
        };
        let event = folder_event(EVENT_TYPE_FOLDER_DELETED, &folder)?;
        EventStore::append_in_transaction(&mut transaction, &event).await?;
        if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
            let link_metadata = merge_metadata(
                serde_json::json!({
                    "operation": "folder_delete",
                }),
                metadata,
            );
            link_mail_entity_in_transaction(
                &mut transaction,
                observation_id,
                "mail_folder",
                folder_id.clone(),
                relationship_kind,
                link_metadata,
                None,
            )
            .await?;
        }
        transaction.commit().await?;
        Ok(true)
    }

    pub async fn list_messages(
        &self,
        query: FolderMessageListQuery<'_>,
    ) -> Result<FolderMessagePage, CommunicationFolderError> {
        let folder_id = normalize_required("folder_id", query.folder_id)?;
        let limit = validate_limit(query.limit);
        let cursor = query
            .cursor
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(decode_folder_message_cursor)
            .transpose()?;
        let fetch_limit = limit + 1;
        let rows = sqlx::query(
            r#"
            SELECT
                fm.folder_id,
                fm.message_id,
                fm.added_at,
                m.account_id,
                m.subject,
                m.sender,
                m.occurred_at,
                m.projected_at,
                m.workflow_state,
                m.local_state,
                count(a.attachment_id)::BIGINT AS attachment_count
            FROM communication_folder_messages fm
            JOIN communication_messages m ON m.message_id = fm.message_id
            LEFT JOIN communication_attachments a ON a.message_id = m.message_id
            WHERE fm.folder_id = $1
              AND (
                $2::timestamptz IS NULL
                OR fm.added_at < $2
                OR (fm.added_at = $2 AND fm.message_id > $3)
              )
            GROUP BY
                fm.folder_id,
                fm.message_id,
                fm.added_at,
                m.account_id,
                m.subject,
                m.sender,
                m.occurred_at,
                m.projected_at,
                m.workflow_state,
                m.local_state
            ORDER BY fm.added_at DESC, fm.message_id ASC
            LIMIT $4
            "#,
        )
        .bind(&folder_id)
        .bind(cursor.as_ref().map(|value| value.added_at))
        .bind(cursor.as_ref().map(|value| value.message_id.as_str()))
        .bind(fetch_limit)
        .fetch_all(&self.pool)
        .await?;
        let has_more = rows.len() > limit as usize;
        let items = rows
            .into_iter()
            .take(limit as usize)
            .map(row_to_folder_message)
            .collect::<Result<Vec<_>, _>>()?;
        let next_cursor = if has_more {
            items.last().map(encode_folder_message_cursor).transpose()?
        } else {
            None
        };

        Ok(FolderMessagePage {
            items,
            next_cursor,
            has_more,
        })
    }

    pub async fn copy_message(
        &self,
        folder_id: &str,
        message_id: &str,
    ) -> Result<Option<FolderMessageActionResponse>, CommunicationFolderError> {
        self.copy_message_with_observation(
            folder_id,
            message_id,
            None,
            "folder_message_transition",
            None,
        )
        .await
    }

    pub async fn copy_message_with_observation(
        &self,
        folder_id: &str,
        message_id: &str,
        observation_id: Option<&str>,
        relationship_kind: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<Option<FolderMessageActionResponse>, CommunicationFolderError> {
        self.apply_message_action_with_observation(
            folder_id,
            message_id,
            FolderMessageOperation::Copy,
            observation_id,
            relationship_kind,
            metadata,
        )
        .await
    }

    pub async fn move_message(
        &self,
        folder_id: &str,
        message_id: &str,
    ) -> Result<Option<FolderMessageActionResponse>, CommunicationFolderError> {
        self.move_message_with_observation(
            folder_id,
            message_id,
            None,
            "folder_message_transition",
            None,
        )
        .await
    }

    pub async fn move_message_with_observation(
        &self,
        folder_id: &str,
        message_id: &str,
        observation_id: Option<&str>,
        relationship_kind: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<Option<FolderMessageActionResponse>, CommunicationFolderError> {
        self.apply_message_action_with_observation(
            folder_id,
            message_id,
            FolderMessageOperation::Move,
            observation_id,
            relationship_kind,
            metadata,
        )
        .await
    }

    async fn apply_message_action(
        &self,
        folder_id: &str,
        message_id: &str,
        operation: FolderMessageOperation,
    ) -> Result<Option<FolderMessageActionResponse>, CommunicationFolderError> {
        self.apply_message_action_with_observation(
            folder_id,
            message_id,
            operation,
            None,
            "folder_message_transition",
            None,
        )
        .await
    }

    async fn apply_message_action_with_observation(
        &self,
        folder_id: &str,
        message_id: &str,
        operation: FolderMessageOperation,
        observation_id: Option<&str>,
        relationship_kind: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<Option<FolderMessageActionResponse>, CommunicationFolderError> {
        let folder_id = normalize_required("folder_id", folder_id)?;
        let message_id = normalize_required("message_id", message_id)?;
        let mut transaction = self.pool.begin().await?;
        if !folder_exists(&mut transaction, &folder_id).await?
            || !message_exists(&mut transaction, &message_id).await?
        {
            transaction.rollback().await?;
            return Ok(None);
        }

        if matches!(operation, FolderMessageOperation::Move) {
            sqlx::query(
                r#"
                DELETE FROM communication_folder_messages
                WHERE message_id = $1 AND folder_id <> $2
                "#,
            )
            .bind(&message_id)
            .bind(&folder_id)
            .execute(&mut *transaction)
            .await?;
        }

        upsert_folder_message(
            &mut transaction,
            &folder_id,
            &message_id,
            operation.as_str(),
        )
        .await?;
        let message = load_folder_message(&mut transaction, &folder_id, &message_id).await?;
        let response = FolderMessageActionResponse {
            operation,
            folder_id,
            message_id,
            message,
        };
        let event = folder_message_event(&response)?;
        EventStore::append_in_transaction(&mut transaction, &event).await?;
        if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
            let link_metadata = merge_metadata(
                serde_json::json!({
                    "folder_id": response.folder_id,
                    "operation": response.operation.as_str(),
                }),
                metadata,
            );
            link_mail_entity_in_transaction(
                &mut transaction,
                observation_id,
                "communication_message",
                response.message_id.clone(),
                relationship_kind,
                link_metadata,
                None,
            )
            .await?;
        }
        transaction.commit().await?;
        Ok(Some(response))
    }
}

#[derive(Debug)]
struct NormalizedCommunicationFolderInput {
    folder_id: String,
    account_id: Option<String>,
    name: String,
    description: Option<String>,
    color: Option<String>,
    sort_order: i32,
}

impl NormalizedCommunicationFolderInput {
    fn from_new(input: NewCommunicationFolder) -> Result<Self, CommunicationFolderError> {
        Ok(Self {
            folder_id: match input.folder_id {
                Some(value) => normalize_required("folder_id", &value)?,
                None => generate_folder_id(),
            },
            account_id: normalize_optional(input.account_id)?,
            name: normalize_required("name", &input.name)?,
            description: normalize_optional(input.description)?,
            color: normalize_optional(input.color)?,
            sort_order: input.sort_order.unwrap_or(0),
        })
    }
}

#[derive(Debug)]
struct NormalizedCommunicationFolderUpdate {
    account_id: Option<String>,
    name: Option<String>,
    description: Option<String>,
    color: Option<String>,
    sort_order: Option<i32>,
}

impl NormalizedCommunicationFolderUpdate {
    fn from_update(update: UpdateCommunicationFolder) -> Result<Self, CommunicationFolderError> {
        Ok(Self {
            account_id: normalize_optional(update.account_id)?,
            name: normalize_optional(update.name)?,
            description: normalize_optional(update.description)?,
            color: normalize_optional(update.color)?,
            sort_order: update.sort_order,
        })
    }
}

async fn ensure_canonical_account_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    account_id: Option<&str>,
) -> Result<(), CommunicationFolderError> {
    let Some(account_id) = account_id else {
        return Ok(());
    };

    sqlx::query(
        r#"
        INSERT INTO communication_accounts (
            account_id, provider_kind, display_name, external_account_id, config, metadata, created_at, updated_at
        )
        SELECT
            account_id,
            provider_kind,
            display_name,
            external_account_id,
            config,
            '{}'::jsonb,
            created_at,
            updated_at
        FROM communication_provider_accounts
        WHERE account_id = $1
        ON CONFLICT (account_id) DO NOTHING
        "#,
    )
    .bind(account_id)
    .execute(&mut **transaction)
    .await?;

    Ok(())
}

async fn insert_folder(
    transaction: &mut Transaction<'_, Postgres>,
    input: &NormalizedCommunicationFolderInput,
) -> Result<CommunicationFolder, CommunicationFolderError> {
    let row = sqlx::query(
        r#"
        WITH inserted AS (
            INSERT INTO communication_folders (
                folder_id, account_id, name, description, color, sort_order
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING folder_id, account_id, name, description, color, sort_order, created_at, updated_at
        )
        SELECT
            f.folder_id,
            f.account_id,
            f.name,
            f.description,
            f.color,
            f.sort_order,
            count(fm.message_id)::BIGINT AS message_count,
            f.created_at,
            f.updated_at
        FROM inserted f
        LEFT JOIN communication_folder_messages fm ON fm.folder_id = f.folder_id
        GROUP BY f.folder_id, f.account_id, f.name, f.description, f.color, f.sort_order, f.created_at, f.updated_at
        "#,
    )
    .bind(&input.folder_id)
    .bind(input.account_id.as_deref())
    .bind(&input.name)
    .bind(input.description.as_deref())
    .bind(input.color.as_deref())
    .bind(input.sort_order)
    .fetch_one(&mut **transaction)
    .await?;

    row_to_folder(row)
}

async fn update_folder(
    transaction: &mut Transaction<'_, Postgres>,
    folder_id: &str,
    update: &NormalizedCommunicationFolderUpdate,
) -> Result<Option<CommunicationFolder>, CommunicationFolderError> {
    let row = sqlx::query(
        r#"
        WITH updated AS (
            UPDATE communication_folders
            SET account_id = COALESCE($2, account_id),
                name = COALESCE($3, name),
                description = COALESCE($4, description),
                color = COALESCE($5, color),
                sort_order = COALESCE($6, sort_order),
                updated_at = now()
            WHERE folder_id = $1
            RETURNING folder_id, account_id, name, description, color, sort_order, created_at, updated_at
        )
        SELECT
            f.folder_id,
            f.account_id,
            f.name,
            f.description,
            f.color,
            f.sort_order,
            count(fm.message_id)::BIGINT AS message_count,
            f.created_at,
            f.updated_at
        FROM updated f
        LEFT JOIN communication_folder_messages fm ON fm.folder_id = f.folder_id
        GROUP BY f.folder_id, f.account_id, f.name, f.description, f.color, f.sort_order, f.created_at, f.updated_at
        "#,
    )
    .bind(folder_id)
    .bind(update.account_id.as_deref())
    .bind(update.name.as_deref())
    .bind(update.description.as_deref())
    .bind(update.color.as_deref())
    .bind(update.sort_order)
    .fetch_optional(&mut **transaction)
    .await?;

    row.map(row_to_folder).transpose()
}

async fn delete_folder(
    transaction: &mut Transaction<'_, Postgres>,
    folder_id: &str,
) -> Result<Option<CommunicationFolder>, CommunicationFolderError> {
    let row = sqlx::query(
        r#"
        WITH target AS (
            SELECT
                f.folder_id,
                f.account_id,
                f.name,
                f.description,
                f.color,
                f.sort_order,
                count(fm.message_id)::BIGINT AS message_count,
                f.created_at,
                f.updated_at
            FROM communication_folders f
            LEFT JOIN communication_folder_messages fm ON fm.folder_id = f.folder_id
            WHERE f.folder_id = $1
            GROUP BY f.folder_id, f.account_id, f.name, f.description, f.color, f.sort_order, f.created_at, f.updated_at
        ),
        deleted AS (
            DELETE FROM communication_folders WHERE folder_id = $1 RETURNING folder_id
        )
        SELECT target.*
        FROM target
        JOIN deleted ON deleted.folder_id = target.folder_id
        "#,
    )
    .bind(folder_id)
    .fetch_optional(&mut **transaction)
    .await?;

    row.map(row_to_folder).transpose()
}

async fn folder_exists(
    transaction: &mut Transaction<'_, Postgres>,
    folder_id: &str,
) -> Result<bool, CommunicationFolderError> {
    Ok(sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM communication_folders WHERE folder_id = $1)",
    )
    .bind(folder_id)
    .fetch_one(&mut **transaction)
    .await?)
}

async fn message_exists(
    transaction: &mut Transaction<'_, Postgres>,
    message_id: &str,
) -> Result<bool, CommunicationFolderError> {
    Ok(sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM communication_messages WHERE message_id = $1)",
    )
    .bind(message_id)
    .fetch_one(&mut **transaction)
    .await?)
}

async fn upsert_folder_message(
    transaction: &mut Transaction<'_, Postgres>,
    folder_id: &str,
    message_id: &str,
    operation: &str,
) -> Result<(), CommunicationFolderError> {
    sqlx::query(
        r#"
        INSERT INTO communication_folder_messages (folder_id, message_id, added_at, last_operation)
        VALUES ($1, $2, now(), $3)
        ON CONFLICT (folder_id, message_id)
        DO UPDATE SET added_at = EXCLUDED.added_at,
                      last_operation = EXCLUDED.last_operation
        "#,
    )
    .bind(folder_id)
    .bind(message_id)
    .bind(operation)
    .execute(&mut **transaction)
    .await?;

    Ok(())
}

async fn load_folder_message(
    transaction: &mut Transaction<'_, Postgres>,
    folder_id: &str,
    message_id: &str,
) -> Result<FolderMessage, CommunicationFolderError> {
    let row = sqlx::query(
        r#"
        SELECT
            fm.folder_id,
            fm.message_id,
            fm.added_at,
            m.account_id,
            m.subject,
            m.sender,
            m.occurred_at,
            m.projected_at,
            m.workflow_state,
            m.local_state,
            count(a.attachment_id)::BIGINT AS attachment_count
        FROM communication_folder_messages fm
        JOIN communication_messages m ON m.message_id = fm.message_id
        LEFT JOIN communication_attachments a ON a.message_id = m.message_id
        WHERE fm.folder_id = $1 AND fm.message_id = $2
        GROUP BY
            fm.folder_id,
            fm.message_id,
            fm.added_at,
            m.account_id,
            m.subject,
            m.sender,
            m.occurred_at,
            m.projected_at,
            m.workflow_state,
            m.local_state
        "#,
    )
    .bind(folder_id)
    .bind(message_id)
    .fetch_one(&mut **transaction)
    .await?;

    row_to_folder_message(row)
}

fn row_to_folder(row: PgRow) -> Result<CommunicationFolder, CommunicationFolderError> {
    Ok(CommunicationFolder {
        folder_id: row.try_get("folder_id")?,
        account_id: row.try_get("account_id")?,
        name: row.try_get("name")?,
        description: row.try_get("description")?,
        color: row.try_get("color")?,
        sort_order: row.try_get("sort_order")?,
        message_count: row.try_get("message_count")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn row_to_folder_message(row: PgRow) -> Result<FolderMessage, CommunicationFolderError> {
    let workflow_state: String = row.try_get("workflow_state")?;
    let local_state: String = row.try_get("local_state")?;

    Ok(FolderMessage {
        folder_id: row.try_get("folder_id")?,
        message_id: row.try_get("message_id")?,
        account_id: row.try_get("account_id")?,
        subject: row.try_get("subject")?,
        sender: row.try_get("sender")?,
        occurred_at: row.try_get("occurred_at")?,
        projected_at: row.try_get("projected_at")?,
        workflow_state: workflow_state
            .parse::<WorkflowState>()
            .map_err(|_| CommunicationFolderError::Invalid("workflow_state"))?,
        local_state: local_state
            .parse::<LocalMessageState>()
            .map_err(|_| CommunicationFolderError::Invalid("local_state"))?,
        added_at: row.try_get("added_at")?,
        attachment_count: row.try_get("attachment_count")?,
    })
}

fn validate_limit(limit: i64) -> i64 {
    limit.clamp(1, 1000)
}

fn normalize_required(
    field: &'static str,
    value: &str,
) -> Result<String, CommunicationFolderError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(CommunicationFolderError::Invalid(field));
    }
    Ok(value.to_owned())
}

fn normalize_optional(value: Option<String>) -> Result<Option<String>, CommunicationFolderError> {
    match value {
        Some(value) => {
            let value = value.trim();
            if value.is_empty() {
                Ok(None)
            } else {
                Ok(Some(value.to_owned()))
            }
        }
        None => Ok(None),
    }
}

fn generate_folder_id() -> String {
    format!("mail_folder:{:x}", system_time_nanos())
}

fn system_time_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default()
}

#[derive(Debug, Error)]
pub enum CommunicationFolderError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Observation(#[from] ObservationStoreError),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error(transparent)]
    EventStore(#[from] crate::platform::events::EventStoreError),
    #[error(transparent)]
    EventEnvelope(#[from] crate::platform::events::EventEnvelopeError),
    #[error("invalid mail folder field: {0}")]
    Invalid(&'static str),
    #[error("invalid mail folder cursor")]
    InvalidCursor,
}
