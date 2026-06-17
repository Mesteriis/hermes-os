use std::collections::HashMap;

use chrono::Utc;
use sha2::{Digest, Sha256};
use sqlx::PgPool;

use super::errors::TelegramError;
use super::models::messages::{
    TelegramForwardChainResponse, TelegramForwardRef, TelegramMessageReferenceSummary,
    TelegramReplyChainResponse, TelegramReplyRef,
};
use super::rows::{row_to_telegram_forward_ref, row_to_telegram_reply_ref};

fn stable_short_hash(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())[..12].to_owned()
}

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
