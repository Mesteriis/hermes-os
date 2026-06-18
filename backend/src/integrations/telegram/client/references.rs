use std::collections::{HashMap, HashSet, VecDeque};

use chrono::Utc;
use sha2::{Digest, Sha256};
use sqlx::PgPool;

use super::errors::TelegramError;
use super::models::messages::{
    TelegramForwardChainResponse, TelegramForwardRef, TelegramMessageReferenceSummary,
    TelegramReplyChainResponse, TelegramReplyRef,
};
use super::rows::{row_to_telegram_forward_ref, row_to_telegram_reply_ref};

const MAX_REFERENCE_CHAIN_DEPTH: usize = 16;
const MAX_REFERENCE_CHAIN_EDGES: usize = 128;

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
    let row = sqlx::query(
        r#"
        WITH inserted AS (
        INSERT INTO telegram_message_reply_refs
            (reply_ref_id, source_message_id, target_message_id, account_id,
             provider_chat_id, source_provider_id, target_provider_id,
             reply_depth, is_topic_reply)
        VALUES ($1, $2, $3, $4, $5, $6, $7, 1, $8)
        ON CONFLICT (source_message_id, target_message_id) DO NOTHING
        RETURNING *
        )
        SELECT * FROM inserted
        UNION ALL
        SELECT * FROM telegram_message_reply_refs
        WHERE source_message_id = $2
          AND target_message_id = $3
          AND NOT EXISTS (SELECT 1 FROM inserted)
        LIMIT 1
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
    .fetch_one(pool)
    .await?;

    row_to_telegram_reply_ref(row)
}

pub async fn reply_chain(
    pool: &PgPool,
    message_id: &str,
) -> Result<TelegramReplyChainResponse, TelegramError> {
    let mut replies = collect_reply_descendants(pool, message_id).await?;
    let mut reply_to = collect_reply_ancestors(pool, message_id).await?;

    let mut summary_ids = Vec::new();
    for item in replies.iter().chain(reply_to.iter()) {
        summary_ids.push(item.source_message_id.as_str());
        summary_ids.push(item.target_message_id.as_str());
    }
    let summaries = reference_message_summaries(pool, summary_ids).await?;
    for item in &mut replies {
        item.source_message_summary = summaries.get(&item.source_message_id).cloned();
        item.target_message_summary = summaries.get(&item.target_message_id).cloned();
    }
    for item in &mut reply_to {
        item.source_message_summary = summaries.get(&item.source_message_id).cloned();
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
    let row = sqlx::query(
        r#"
        WITH inserted AS (
        INSERT INTO telegram_message_forward_refs
            (forward_ref_id, source_message_id, account_id, provider_chat_id,
             source_provider_id, forward_origin_chat_id, forward_origin_message_id,
             forward_origin_sender_id, forward_origin_sender_name, forward_date, forward_depth)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, 1)
        ON CONFLICT (source_message_id, account_id) DO NOTHING
        RETURNING *
        )
        SELECT * FROM inserted
        UNION ALL
        SELECT * FROM telegram_message_forward_refs
        WHERE source_message_id = $2
          AND account_id = $3
          AND NOT EXISTS (SELECT 1 FROM inserted)
        LIMIT 1
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
    .fetch_one(pool)
    .await?;

    row_to_telegram_forward_ref(row)
}

pub async fn forward_chain(
    pool: &PgPool,
    message_id: &str,
) -> Result<TelegramForwardChainResponse, TelegramError> {
    let mut forwards = collect_forward_ancestors(pool, message_id).await?;

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

async fn collect_reply_descendants(
    pool: &PgPool,
    message_id: &str,
) -> Result<Vec<TelegramReplyRef>, TelegramError> {
    let mut refs = Vec::new();
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    visited.insert(message_id.to_owned());
    queue.push_back((message_id.to_owned(), 0usize));

    while let Some((current_id, depth)) = queue.pop_front() {
        if depth >= MAX_REFERENCE_CHAIN_DEPTH {
            continue;
        }
        for mut item in reply_refs_by_target(pool, &current_id).await? {
            let next_id = item.source_message_id.clone();
            if !visited.insert(next_id.clone()) {
                continue;
            }
            item.reply_depth = (depth + 1) as i32;
            refs.push(item);
            if refs.len() >= MAX_REFERENCE_CHAIN_EDGES {
                return Ok(refs);
            }
            queue.push_back((next_id, depth + 1));
        }
    }

    Ok(refs)
}

async fn collect_reply_ancestors(
    pool: &PgPool,
    message_id: &str,
) -> Result<Vec<TelegramReplyRef>, TelegramError> {
    let mut refs = Vec::new();
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    visited.insert(message_id.to_owned());
    queue.push_back((message_id.to_owned(), 0usize));

    while let Some((current_id, depth)) = queue.pop_front() {
        if depth >= MAX_REFERENCE_CHAIN_DEPTH {
            continue;
        }
        for mut item in reply_refs_by_source(pool, &current_id).await? {
            let next_id = item.target_message_id.clone();
            if !visited.insert(next_id.clone()) {
                continue;
            }
            item.reply_depth = (depth + 1) as i32;
            refs.push(item);
            if refs.len() >= MAX_REFERENCE_CHAIN_EDGES {
                return Ok(refs);
            }
            queue.push_back((next_id, depth + 1));
        }
    }

    Ok(refs)
}

async fn reply_refs_by_target(
    pool: &PgPool,
    message_id: &str,
) -> Result<Vec<TelegramReplyRef>, TelegramError> {
    sqlx::query(
        "SELECT * FROM telegram_message_reply_refs WHERE target_message_id = $1 ORDER BY created_at DESC",
    )
    .bind(message_id)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(row_to_telegram_reply_ref)
    .collect::<Result<_, _>>()
}

async fn reply_refs_by_source(
    pool: &PgPool,
    message_id: &str,
) -> Result<Vec<TelegramReplyRef>, TelegramError> {
    sqlx::query(
        "SELECT * FROM telegram_message_reply_refs WHERE source_message_id = $1 ORDER BY created_at DESC",
    )
    .bind(message_id)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(row_to_telegram_reply_ref)
    .collect::<Result<_, _>>()
}

async fn collect_forward_ancestors(
    pool: &PgPool,
    message_id: &str,
) -> Result<Vec<TelegramForwardRef>, TelegramError> {
    let mut refs = Vec::new();
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    visited.insert(message_id.to_owned());
    queue.push_back((message_id.to_owned(), 0usize));

    while let Some((current_id, depth)) = queue.pop_front() {
        if depth >= MAX_REFERENCE_CHAIN_DEPTH {
            continue;
        }
        for mut item in forward_refs_by_source(pool, &current_id).await? {
            let origin_message_id = local_forward_origin_message_id(pool, &item).await?;
            if let Some(next_id) = origin_message_id {
                if !visited.insert(next_id.clone()) {
                    continue;
                }
                item.forward_depth = (depth + 1) as i32;
                refs.push(item);
                if refs.len() >= MAX_REFERENCE_CHAIN_EDGES {
                    return Ok(refs);
                }
                queue.push_back((next_id, depth + 1));
            } else {
                item.forward_depth = (depth + 1) as i32;
                refs.push(item);
                if refs.len() >= MAX_REFERENCE_CHAIN_EDGES {
                    return Ok(refs);
                }
            }
        }
    }

    Ok(refs)
}

async fn forward_refs_by_source(
    pool: &PgPool,
    message_id: &str,
) -> Result<Vec<TelegramForwardRef>, TelegramError> {
    sqlx::query(
        "SELECT * FROM telegram_message_forward_refs WHERE source_message_id = $1 ORDER BY created_at DESC",
    )
    .bind(message_id)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(row_to_telegram_forward_ref)
    .collect::<Result<_, _>>()
}

async fn local_forward_origin_message_id(
    pool: &PgPool,
    item: &TelegramForwardRef,
) -> Result<Option<String>, TelegramError> {
    let Some(origin_provider_message_id) = item.forward_origin_message_id.as_deref() else {
        return Ok(None);
    };
    let message_id = sqlx::query_scalar::<_, String>(
        r#"
        SELECT message_id
        FROM communication_messages
        WHERE account_id = $1 AND provider_record_id = $2
        LIMIT 1
        "#,
    )
    .bind(&item.account_id)
    .bind(origin_provider_message_id)
    .fetch_optional(pool)
    .await?;
    Ok(message_id)
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
