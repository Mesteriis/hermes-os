use chrono::Utc;
use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use sqlx::Row;

use super::errors::TelegramError;
use super::models::messages::{
    TelegramCommandKind, TelegramDeleteRequest, TelegramEditRequest, TelegramLifecycleResponse,
    TelegramMessageTombstone, TelegramMessageVersion, TelegramPinRequest,
    TelegramProviderWriteCommand, TelegramRestoreVisibilityRequest,
};
use super::rows::{
    row_to_telegram_message_tombstone, row_to_telegram_message_version,
    row_to_telegram_provider_write_command,
};

fn stable_short_hash(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())[..12].to_owned()
}

fn new_version_id() -> String {
    let now = Utc::now();
    format!(
        "tmsgver_{}_{}",
        now.timestamp_millis(),
        stable_short_hash(&format!("ver_{}", now.timestamp_nanos_opt().unwrap_or(0)))
    )
}

fn new_tombstone_id() -> String {
    let now = Utc::now();
    format!(
        "tmsgtomb_{}_{}",
        now.timestamp_millis(),
        stable_short_hash(&format!("tomb_{}", now.timestamp_nanos_opt().unwrap_or(0)))
    )
}

pub fn new_command_id() -> String {
    let now = Utc::now();
    format!(
        "tcmd_{}_{}",
        now.timestamp_millis(),
        stable_short_hash(&format!("cmd_{}", now.timestamp_nanos_opt().unwrap_or(0)))
    )
}

// ---------------------------------------------------------------------------
// Message version queries
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
pub async fn insert_message_version(
    pool: &PgPool,
    message_id: &str,
    account_id: &str,
    provider_message_id: &str,
    provider_chat_id: &str,
    version_number: i32,
    body_text: Option<&str>,
    edit_timestamp: chrono::DateTime<Utc>,
    source_event: Option<&str>,
    raw_diff: serde_json::Value,
    provenance: serde_json::Value,
) -> Result<TelegramMessageVersion, TelegramError> {
    let version_id = new_version_id();
    sqlx::query(
        r#"
        INSERT INTO telegram_message_versions
            (version_id, message_id, account_id, provider_message_id, provider_chat_id,
             version_number, body_text, edit_timestamp, source_event,
             raw_diff_payload, provenance)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        "#,
    )
    .bind(&version_id)
    .bind(message_id)
    .bind(account_id)
    .bind(provider_message_id)
    .bind(provider_chat_id)
    .bind(version_number)
    .bind(body_text)
    .bind(edit_timestamp)
    .bind(source_event)
    .bind(&raw_diff)
    .bind(&provenance)
    .execute(pool)
    .await?;

    let row = sqlx::query("SELECT * FROM telegram_message_versions WHERE version_id = $1")
        .bind(&version_id)
        .fetch_one(pool)
        .await?;

    row_to_telegram_message_version(row)
}

pub async fn list_message_versions(
    pool: &PgPool,
    message_id: &str,
) -> Result<Vec<TelegramMessageVersion>, TelegramError> {
    let rows = sqlx::query(
        r#"
        SELECT * FROM telegram_message_versions
        WHERE message_id = $1
        ORDER BY version_number DESC
        "#,
    )
    .bind(message_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(row_to_telegram_message_version)
        .collect()
}

pub async fn latest_version_number(pool: &PgPool, message_id: &str) -> Result<i32, TelegramError> {
    let row: Option<(i32,)> = sqlx::query_as(
        r#"
        SELECT COALESCE(MAX(version_number), 0) as max_ver
        FROM telegram_message_versions
        WHERE message_id = $1
        "#,
    )
    .bind(message_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| r.0).unwrap_or(0))
}

// ---------------------------------------------------------------------------
// Tombstone queries
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
pub async fn insert_tombstone(
    pool: &PgPool,
    message_id: &str,
    account_id: &str,
    provider_message_id: &str,
    provider_chat_id: &str,
    reason_class: &str,
    actor_class: &str,
    observed_at: chrono::DateTime<Utc>,
    source_event: Option<&str>,
    is_provider_delete: bool,
    is_local_visible: bool,
) -> Result<TelegramMessageTombstone, TelegramError> {
    let tombstone_id = new_tombstone_id();
    sqlx::query(
        r#"
        INSERT INTO telegram_message_tombstones
            (tombstone_id, message_id, account_id, provider_message_id, provider_chat_id,
             reason_class, actor_class, observed_at, source_event,
             is_provider_delete, is_local_visible)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        "#,
    )
    .bind(&tombstone_id)
    .bind(message_id)
    .bind(account_id)
    .bind(provider_message_id)
    .bind(provider_chat_id)
    .bind(reason_class)
    .bind(actor_class)
    .bind(observed_at)
    .bind(source_event)
    .bind(is_provider_delete)
    .bind(is_local_visible)
    .execute(pool)
    .await?;

    let row = sqlx::query("SELECT * FROM telegram_message_tombstones WHERE tombstone_id = $1")
        .bind(&tombstone_id)
        .fetch_one(pool)
        .await?;

    row_to_telegram_message_tombstone(row)
}

pub async fn list_tombstones(
    pool: &PgPool,
    message_id: &str,
) -> Result<Vec<TelegramMessageTombstone>, TelegramError> {
    let rows = sqlx::query(
        r#"
        SELECT * FROM telegram_message_tombstones
        WHERE message_id = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(message_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(row_to_telegram_message_tombstone)
        .collect()
}

pub async fn is_message_visible(pool: &PgPool, message_id: &str) -> Result<bool, TelegramError> {
    let row: Option<(bool,)> = sqlx::query_as(
        r#"
        SELECT is_local_visible
        FROM telegram_message_tombstones
        WHERE message_id = $1
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(message_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| r.0).unwrap_or(true))
}

// ---------------------------------------------------------------------------
// Provider-write command queries
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
pub async fn insert_command(
    pool: &PgPool,
    command_id: &str,
    account_id: &str,
    command_kind: &str,
    idempotency_key: &str,
    provider_chat_id: &str,
    provider_message_id: Option<&str>,
    capability_state: &str,
    action_class: &str,
    confirmation_decision: &str,
    actor_id: &str,
    payload: serde_json::Value,
    target_ref: serde_json::Value,
    audit_metadata: serde_json::Value,
) -> Result<TelegramProviderWriteCommand, TelegramError> {
    sqlx::query(
        r#"
        INSERT INTO telegram_provider_write_commands
            (command_id, account_id, command_kind, idempotency_key, provider_chat_id,
             provider_message_id, capability_state, action_class, confirmation_decision,
             status, retry_count, max_retries, actor_id, payload, target_ref, audit_metadata)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'queued', 0, 3, $10, $11, $12, $13)
        "#,
    )
    .bind(command_id)
    .bind(account_id)
    .bind(command_kind)
    .bind(idempotency_key)
    .bind(provider_chat_id)
    .bind(provider_message_id)
    .bind(capability_state)
    .bind(action_class)
    .bind(confirmation_decision)
    .bind(actor_id)
    .bind(&payload)
    .bind(&target_ref)
    .bind(&audit_metadata)
    .execute(pool)
    .await?;

    let row = sqlx::query("SELECT * FROM telegram_provider_write_commands WHERE command_id = $1")
        .bind(command_id)
        .fetch_one(pool)
        .await?;

    row_to_telegram_provider_write_command(row)
}

pub async fn update_command_status(
    pool: &PgPool,
    command_id: &str,
    status: &str,
    result_payload: serde_json::Value,
    last_error: Option<&str>,
    completed_at: Option<chrono::DateTime<Utc>>,
) -> Result<(), TelegramError> {
    sqlx::query(
        r#"
        UPDATE telegram_provider_write_commands
        SET status = $2, result_payload = $3, last_error = $4,
            completed_at = $5, updated_at = now()
        WHERE command_id = $1
        "#,
    )
    .bind(command_id)
    .bind(status)
    .bind(&result_payload)
    .bind(last_error)
    .bind(completed_at)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn retry_command(pool: &PgPool, command_id: &str) -> Result<(), TelegramError> {
    sqlx::query(
        r#"
        UPDATE telegram_provider_write_commands
        SET status = 'retrying', retry_count = retry_count + 1, updated_at = now()
        WHERE command_id = $1 AND retry_count < max_retries
        "#,
    )
    .bind(command_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn find_command_by_idempotency(
    pool: &PgPool,
    account_id: &str,
    idempotency_key: &str,
) -> Result<Option<TelegramProviderWriteCommand>, TelegramError> {
    let row = sqlx::query(
        r#"
        SELECT * FROM telegram_provider_write_commands
        WHERE account_id = $1 AND idempotency_key = $2
        "#,
    )
    .bind(account_id)
    .bind(idempotency_key)
    .fetch_optional(pool)
    .await?;

    row.map(row_to_telegram_provider_write_command).transpose()
}

pub async fn list_commands(
    pool: &PgPool,
    account_id: &str,
    limit: i64,
) -> Result<Vec<TelegramProviderWriteCommand>, TelegramError> {
    let rows = sqlx::query(
        r#"
        SELECT * FROM telegram_provider_write_commands
        WHERE account_id = $1
        ORDER BY created_at DESC
        LIMIT $2
        "#,
    )
    .bind(account_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(row_to_telegram_provider_write_command)
        .collect()
}

/// List commands eligible for executor dispatch: queued or retrying, retry count under limit.
/// Only returns provider-executable command kinds (edit, delete, react, unreact, pin, unpin).
pub async fn list_queued_commands_for_execution(
    pool: &PgPool,
    account_id: &str,
    limit: i64,
) -> Result<Vec<TelegramProviderWriteCommand>, TelegramError> {
    let rows = sqlx::query(
        r#"
        SELECT * FROM telegram_provider_write_commands
        WHERE account_id = $1
          AND status IN ('queued', 'retrying')
          AND retry_count < max_retries
          AND command_kind IN ('edit', 'delete', 'react', 'unreact', 'pin', 'unpin')
        ORDER BY created_at ASC
        LIMIT $2
        "#,
    )
    .bind(account_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(row_to_telegram_provider_write_command)
        .collect()
}

// ---------------------------------------------------------------------------
// Composite lifecycle operations
// ---------------------------------------------------------------------------

pub async fn record_edit(
    pool: &PgPool,
    request: &TelegramEditRequest,
    message_id: &str,
    actor_id: &str,
) -> Result<TelegramLifecycleResponse, TelegramError> {
    let now = Utc::now();
    let version_number = latest_version_number(pool, message_id).await? + 1;

    let _version = insert_message_version(
        pool,
        message_id,
        &request.account_id,
        &request.provider_message_id,
        &request.provider_chat_id,
        version_number,
        Some(&request.new_text),
        now,
        None,
        json!({"text_length": request.new_text.len()}),
        json!({"event": "local_edit"}),
    )
    .await?;

    let idempotency_key = format!("edit:{}:{}", request.provider_message_id, version_number);
    let _cmd = insert_command(
        pool,
        &request.command_id,
        &request.account_id,
        TelegramCommandKind::Edit.as_str(),
        &idempotency_key,
        &request.provider_chat_id,
        Some(&request.provider_message_id),
        "available",
        "provider_write",
        "confirmed",
        actor_id,
        json!({"new_text": &request.new_text}),
        json!({"provider_message_id": &request.provider_message_id}),
        json!({"edit_version": version_number}),
    )
    .await?;

    Ok(TelegramLifecycleResponse {
        operation: "edit".to_owned(),
        message_id: message_id.to_owned(),
        account_id: request.account_id.clone(),
        provider_chat_id: request.provider_chat_id.clone(),
        provider_message_id: request.provider_message_id.clone(),
        status: "version_recorded".to_owned(),
        timestamp: now,
        version_number: Some(version_number),
        tombstone_id: None,
    })
}

pub async fn record_delete(
    pool: &PgPool,
    request: &TelegramDeleteRequest,
    message_id: &str,
    actor_id: &str,
) -> Result<TelegramLifecycleResponse, TelegramError> {
    let now = Utc::now();

    let tombstone = insert_tombstone(
        pool,
        message_id,
        &request.account_id,
        &request.provider_message_id,
        &request.provider_chat_id,
        &request.reason_class,
        &request.actor_class,
        now,
        None,
        request.is_provider_delete,
        false,
    )
    .await?;

    let idempotency_key = format!(
        "delete:{}:{}",
        request.provider_message_id,
        now.timestamp_millis()
    );
    let _cmd = insert_command(
        pool,
        &request.command_id,
        &request.account_id,
        TelegramCommandKind::Delete.as_str(),
        &idempotency_key,
        &request.provider_chat_id,
        Some(&request.provider_message_id),
        "available",
        "destructive",
        "confirmed",
        actor_id,
        json!({"reason_class": &request.reason_class, "is_provider_delete": request.is_provider_delete}),
        json!({"provider_message_id": &request.provider_message_id}),
        json!({"tombstone_id": &tombstone.tombstone_id}),
    )
    .await?;

    Ok(TelegramLifecycleResponse {
        operation: "delete".to_owned(),
        message_id: message_id.to_owned(),
        account_id: request.account_id.clone(),
        provider_chat_id: request.provider_chat_id.clone(),
        provider_message_id: request.provider_message_id.clone(),
        status: "tombstone_recorded".to_owned(),
        timestamp: now,
        version_number: None,
        tombstone_id: Some(tombstone.tombstone_id),
    })
}

pub async fn record_restore_visibility(
    pool: &PgPool,
    request: &TelegramRestoreVisibilityRequest,
    message_id: &str,
    actor_id: &str,
) -> Result<TelegramLifecycleResponse, TelegramError> {
    let now = Utc::now();

    let tombstone = insert_tombstone(
        pool,
        message_id,
        &request.account_id,
        &request.provider_message_id,
        &request.provider_chat_id,
        "unknown",
        "owner",
        now,
        None,
        false,
        true,
    )
    .await?;

    let idempotency_key = format!(
        "restore_visibility:{}:{}",
        request.provider_message_id,
        now.timestamp_millis()
    );
    let _cmd = insert_command(
        pool,
        &request.command_id,
        &request.account_id,
        TelegramCommandKind::RestoreVisibility.as_str(),
        &idempotency_key,
        &request.provider_chat_id,
        Some(&request.provider_message_id),
        "degraded",
        "local_write",
        "confirmed",
        actor_id,
        json!({"reason": &request.reason}),
        json!({"provider_message_id": &request.provider_message_id}),
        json!({"tombstone_id": &tombstone.tombstone_id}),
    )
    .await?;

    Ok(TelegramLifecycleResponse {
        operation: "restore_visibility".to_owned(),
        message_id: message_id.to_owned(),
        account_id: request.account_id.clone(),
        provider_chat_id: request.provider_chat_id.clone(),
        provider_message_id: request.provider_message_id.clone(),
        status: "visibility_restored".to_owned(),
        timestamp: now,
        version_number: None,
        tombstone_id: Some(tombstone.tombstone_id),
    })
}

pub async fn record_pin_state(
    pool: &PgPool,
    request: &TelegramPinRequest,
    message_id: &str,
    actor_id: &str,
) -> Result<TelegramLifecycleResponse, TelegramError> {
    let now = Utc::now();
    let updated = sqlx::query(
        r#"
        UPDATE communication_messages
        SET message_metadata = jsonb_set(
                jsonb_set(
                    COALESCE(message_metadata, '{}'::jsonb),
                    '{pinned}',
                    to_jsonb($2::boolean),
                    true
                ),
                '{is_pinned}',
                to_jsonb($2::boolean),
                true
            )
        WHERE message_id = $1
        RETURNING message_metadata
        "#,
    )
    .bind(message_id)
    .bind(request.is_pinned)
    .fetch_optional(pool)
    .await?;

    if updated.is_none() {
        return Err(TelegramError::InvalidRequest(format!(
            "telegram message `{message_id}` was not found"
        )));
    }

    let command_kind = if request.is_pinned {
        TelegramCommandKind::Pin
    } else {
        TelegramCommandKind::Unpin
    };
    let idempotency_key = format!(
        "{}:{}:{}",
        command_kind.as_str(),
        request.provider_message_id,
        now.timestamp_millis()
    );
    insert_command(
        pool,
        &request.command_id,
        &request.account_id,
        command_kind.as_str(),
        &idempotency_key,
        &request.provider_chat_id,
        Some(&request.provider_message_id),
        "degraded",
        "local_write",
        "confirmed",
        actor_id,
        json!({"is_pinned": request.is_pinned}),
        json!({"provider_message_id": &request.provider_message_id}),
        json!({"message_id": message_id, "is_pinned": request.is_pinned}),
    )
    .await?;

    Ok(TelegramLifecycleResponse {
        operation: command_kind.as_str().to_owned(),
        message_id: message_id.to_owned(),
        account_id: request.account_id.clone(),
        provider_chat_id: request.provider_chat_id.clone(),
        provider_message_id: request.provider_message_id.clone(),
        status: if request.is_pinned {
            "pinned".to_owned()
        } else {
            "unpinned".to_owned()
        },
        timestamp: now,
        version_number: None,
        tombstone_id: None,
    })
}

// ---------------------------------------------------------------------------
// Reaction queries (ADR-0091)
// ---------------------------------------------------------------------------

use super::models::messages::{
    TelegramReaction, TelegramReactionGroup, TelegramReactionRequest, TelegramReactionResponse,
    TelegramReactionSummary,
};
use super::rows::row_to_telegram_reaction;

fn new_reaction_id() -> String {
    let now = Utc::now();
    format!(
        "tmsgreact_{}_{}",
        now.timestamp_millis(),
        stable_short_hash(&format!("react_{}", now.timestamp_nanos_opt().unwrap_or(0)))
    )
}

/// Add or update a reaction (sets is_active = true).
pub async fn add_reaction(
    pool: &PgPool,
    request: &TelegramReactionRequest,
    message_id: &str,
) -> Result<TelegramReactionResponse, TelegramError> {
    let now = Utc::now();
    let reaction_id = new_reaction_id();

    // Upsert: activate existing or insert new
    sqlx::query(
        r#"
        INSERT INTO telegram_message_reactions
            (reaction_id, message_id, account_id, provider_message_id, provider_chat_id,
             sender_id, sender_display_name, reaction_emoji, is_active, observed_at,
             provider_actor_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, true, $9, $10)
        ON CONFLICT (message_id, sender_id, reaction_emoji)
        DO UPDATE SET is_active = true, updated_at = now()
        "#,
    )
    .bind(&reaction_id)
    .bind(message_id)
    .bind(&request.account_id)
    .bind(&request.provider_message_id)
    .bind(&request.provider_chat_id)
    .bind(&request.sender_id)
    .bind(&request.sender_display_name)
    .bind(&request.reaction_emoji)
    .bind(now)
    .bind(&request.sender_id)
    .execute(pool)
    .await?;

    if let Some(command_id) = request.command_id.as_deref() {
        let idempotency_key = format!(
            "react:{}:{}:{}",
            request.provider_message_id, request.sender_id, request.reaction_emoji
        );
        let _cmd = insert_command(
            pool,
            command_id,
            &request.account_id,
            TelegramCommandKind::React.as_str(),
            &idempotency_key,
            &request.provider_chat_id,
            Some(&request.provider_message_id),
            "degraded",
            "local_write",
            "confirmed",
            request.sender_id.as_str(),
            json!({"reaction_emoji": &request.reaction_emoji}),
            json!({"provider_message_id": &request.provider_message_id}),
            json!({"reaction_id": &reaction_id, "is_active": true}),
        )
        .await?;
    }

    Ok(TelegramReactionResponse {
        reaction_id,
        message_id: message_id.to_owned(),
        account_id: request.account_id.clone(),
        provider_chat_id: request.provider_chat_id.clone(),
        provider_message_id: request.provider_message_id.clone(),
        reaction_emoji: request.reaction_emoji.clone(),
        is_active: true,
        status: "added".to_owned(),
        timestamp: now,
    })
}

/// Remove a reaction (sets is_active = false).
pub async fn remove_reaction(
    pool: &PgPool,
    request: &TelegramReactionRequest,
    message_id: &str,
) -> Result<TelegramReactionResponse, TelegramError> {
    let now = Utc::now();

    sqlx::query(
        r#"
        UPDATE telegram_message_reactions
        SET is_active = false, updated_at = now()
        WHERE message_id = $1
          AND sender_id = $2
          AND reaction_emoji = $3
          AND is_active = true
        "#,
    )
    .bind(message_id)
    .bind(&request.sender_id)
    .bind(&request.reaction_emoji)
    .execute(pool)
    .await?;

    if let Some(command_id) = request.command_id.as_deref() {
        let idempotency_key = format!(
            "unreact:{}:{}:{}",
            request.provider_message_id, request.sender_id, request.reaction_emoji
        );
        let _cmd = insert_command(
            pool,
            command_id,
            &request.account_id,
            TelegramCommandKind::Unreact.as_str(),
            &idempotency_key,
            &request.provider_chat_id,
            Some(&request.provider_message_id),
            "degraded",
            "local_write",
            "confirmed",
            request.sender_id.as_str(),
            json!({"reaction_emoji": &request.reaction_emoji}),
            json!({"provider_message_id": &request.provider_message_id}),
            json!({"is_active": false}),
        )
        .await?;
    }

    Ok(TelegramReactionResponse {
        reaction_id: String::new(),
        message_id: message_id.to_owned(),
        account_id: request.account_id.clone(),
        provider_chat_id: request.provider_chat_id.clone(),
        provider_message_id: request.provider_message_id.clone(),
        reaction_emoji: request.reaction_emoji.clone(),
        is_active: false,
        status: "removed".to_owned(),
        timestamp: now,
    })
}

/// List active reactions for a message.
pub async fn list_reactions(
    pool: &PgPool,
    message_id: &str,
) -> Result<Vec<TelegramReaction>, TelegramError> {
    let rows = sqlx::query(
        r#"
        SELECT * FROM telegram_message_reactions
        WHERE message_id = $1 AND is_active = true
        ORDER BY created_at DESC
        "#,
    )
    .bind(message_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter().map(row_to_telegram_reaction).collect()
}

/// Get reaction summary for a message (aggregate by emoji).
pub async fn reaction_summary(
    pool: &PgPool,
    message_id: &str,
) -> Result<TelegramReactionSummary, TelegramError> {
    let reactions = list_reactions(pool, message_id).await?;

    let mut groups: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    for r in &reactions {
        groups.entry(r.reaction_emoji.clone()).or_default().push(
            r.sender_display_name
                .clone()
                .unwrap_or_else(|| r.sender_id.clone()),
        );
    }

    let reaction_groups: Vec<TelegramReactionGroup> = groups
        .into_iter()
        .map(|(emoji, senders)| TelegramReactionGroup {
            reaction_emoji: emoji,
            count: senders.len() as i64,
            senders,
        })
        .collect();

    Ok(TelegramReactionSummary {
        message_id: message_id.to_owned(),
        total_reactions: reactions.len() as i64,
        active_reactions: reactions.len() as i64,
        reactions: reaction_groups,
    })
}

// ---------------------------------------------------------------------------
// Reply and Forward queries (ADR-0091)
// ---------------------------------------------------------------------------

use super::models::messages::{
    TelegramForwardChainResponse, TelegramForwardRef, TelegramMessageReferenceSummary,
    TelegramReplyChainResponse, TelegramReplyRef,
};
use super::rows::{row_to_telegram_forward_ref, row_to_telegram_reply_ref};
use std::collections::HashMap;

fn new_reply_ref_id() -> String {
    let now = Utc::now();
    format!(
        "tmsgreply_{}_{}",
        now.timestamp_millis(),
        stable_short_hash(&format!("reply_{}", now.timestamp_nanos_opt().unwrap_or(0)))
    )
}

fn new_forward_ref_id() -> String {
    let now = Utc::now();
    format!(
        "tmsgfwd_{}_{}",
        now.timestamp_millis(),
        stable_short_hash(&format!("fwd_{}", now.timestamp_nanos_opt().unwrap_or(0)))
    )
}

/// Insert a reply reference: source_message replies to target_message.
#[allow(clippy::too_many_arguments)]
pub async fn insert_reply_ref(
    pool: &PgPool,
    source_message_id: &str,
    target_message_id: &str,
    account_id: &str,
    provider_chat_id: &str,
    source_provider_id: &str,
    target_provider_id: &str,
    is_topic_reply: bool,
) -> Result<TelegramReplyRef, TelegramError> {
    let reply_ref_id = new_reply_ref_id();
    sqlx::query(
        r#"
        INSERT INTO telegram_message_reply_refs
            (reply_ref_id, source_message_id, target_message_id, account_id,
             provider_chat_id, source_provider_id, target_provider_id,
             reply_depth, is_topic_reply)
        VALUES ($1, $2, $3, $4, $5, $6, $7, 1, $8)
        ON CONFLICT (source_message_id, target_message_id) DO NOTHING
        "#,
    )
    .bind(&reply_ref_id)
    .bind(source_message_id)
    .bind(target_message_id)
    .bind(account_id)
    .bind(provider_chat_id)
    .bind(source_provider_id)
    .bind(target_provider_id)
    .bind(is_topic_reply)
    .execute(pool)
    .await?;

    let row = sqlx::query("SELECT * FROM telegram_message_reply_refs WHERE reply_ref_id = $1")
        .bind(&reply_ref_id)
        .fetch_one(pool)
        .await?;

    row_to_telegram_reply_ref(row)
}

/// Get the reply chain for a message: replies to it and what it replies to.
pub async fn reply_chain(
    pool: &PgPool,
    message_id: &str,
) -> Result<TelegramReplyChainResponse, TelegramError> {
    let mut replies: Vec<TelegramReplyRef> = sqlx::query(
        "SELECT * FROM telegram_message_reply_refs WHERE target_message_id = $1 ORDER BY created_at DESC",
    )
    .bind(message_id)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(row_to_telegram_reply_ref)
    .collect::<Result<_, _>>()?;

    let mut reply_to: Vec<TelegramReplyRef> = sqlx::query(
        "SELECT * FROM telegram_message_reply_refs WHERE source_message_id = $1 ORDER BY created_at DESC",
    )
    .bind(message_id)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(row_to_telegram_reply_ref)
    .collect::<Result<_, _>>()?;

    let summaries = reference_message_summaries(
        pool,
        replies
            .iter()
            .map(|item| item.source_message_id.as_str())
            .chain(reply_to.iter().map(|item| item.target_message_id.as_str()))
            .collect(),
    )
    .await?;
    for item in &mut replies {
        item.source_message_summary = summaries.get(&item.source_message_id).cloned();
    }
    for item in &mut reply_to {
        item.target_message_summary = summaries.get(&item.target_message_id).cloned();
    }

    Ok(TelegramReplyChainResponse {
        message_id: message_id.to_owned(),
        replies,
        reply_to,
    })
}

/// Insert a forward reference.
#[allow(clippy::too_many_arguments)]
pub async fn insert_forward_ref(
    pool: &PgPool,
    source_message_id: &str,
    account_id: &str,
    provider_chat_id: &str,
    source_provider_id: &str,
    origin_chat_id: Option<&str>,
    origin_message_id: Option<&str>,
    origin_sender_id: Option<&str>,
    origin_sender_name: Option<&str>,
    forward_date: Option<chrono::DateTime<Utc>>,
) -> Result<TelegramForwardRef, TelegramError> {
    let forward_ref_id = new_forward_ref_id();
    sqlx::query(
        r#"
        INSERT INTO telegram_message_forward_refs
            (forward_ref_id, source_message_id, account_id, provider_chat_id,
             source_provider_id, forward_origin_chat_id, forward_origin_message_id,
             forward_origin_sender_id, forward_origin_sender_name, forward_date, forward_depth)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, 1)
        ON CONFLICT (source_message_id, account_id) DO NOTHING
        "#,
    )
    .bind(&forward_ref_id)
    .bind(source_message_id)
    .bind(account_id)
    .bind(provider_chat_id)
    .bind(source_provider_id)
    .bind(origin_chat_id)
    .bind(origin_message_id)
    .bind(origin_sender_id)
    .bind(origin_sender_name)
    .bind(forward_date)
    .execute(pool)
    .await?;

    let row = sqlx::query("SELECT * FROM telegram_message_forward_refs WHERE forward_ref_id = $1")
        .bind(&forward_ref_id)
        .fetch_one(pool)
        .await?;

    row_to_telegram_forward_ref(row)
}

/// Get forward chain for a message.
pub async fn forward_chain(
    pool: &PgPool,
    message_id: &str,
) -> Result<TelegramForwardChainResponse, TelegramError> {
    let mut forwards: Vec<TelegramForwardRef> = sqlx::query(
        "SELECT * FROM telegram_message_forward_refs WHERE source_message_id = $1 ORDER BY created_at DESC",
    )
    .bind(message_id)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(row_to_telegram_forward_ref)
    .collect::<Result<_, _>>()?;

    let summaries = reference_message_summaries(
        pool,
        forwards
            .iter()
            .map(|item| item.source_message_id.as_str())
            .collect(),
    )
    .await?;
    for item in &mut forwards {
        item.source_message_summary = summaries.get(&item.source_message_id).cloned();
    }

    Ok(TelegramForwardChainResponse {
        message_id: message_id.to_owned(),
        forwards,
    })
}

async fn reference_message_summaries(
    pool: &PgPool,
    message_ids: Vec<&str>,
) -> Result<HashMap<String, TelegramMessageReferenceSummary>, TelegramError> {
    if message_ids.is_empty() {
        return Ok(HashMap::new());
    }
    let rows = sqlx::query_as::<
        _,
        (
            String,
            String,
            Option<String>,
            String,
            String,
            Option<String>,
            String,
            Option<chrono::DateTime<Utc>>,
        ),
    >(
        r#"
        SELECT
            message_id,
            provider_record_id,
            conversation_id,
            subject,
            sender,
            sender_display_name,
            body_text,
            occurred_at
        FROM communication_messages
        WHERE message_id = ANY($1)
        "#,
    )
    .bind(&message_ids)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(
            |(
                message_id,
                provider_message_id,
                provider_chat_id,
                chat_title,
                sender,
                sender_display_name,
                text,
                occurred_at,
            )| {
                (
                    message_id.clone(),
                    TelegramMessageReferenceSummary {
                        message_id,
                        provider_message_id,
                        provider_chat_id,
                        chat_title,
                        sender,
                        sender_display_name,
                        text,
                        occurred_at,
                    },
                )
            },
        )
        .collect())
}
