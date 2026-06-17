use chrono::Utc;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::PgPool;

use super::commands::insert_command;
use super::errors::TelegramError;
use super::messages::reaction_metadata::TdlibProviderReaction;
use super::models::messages::{
    TelegramCommandKind, TelegramProviderWriteCommand, TelegramReaction, TelegramReactionGroup,
    TelegramReactionRequest, TelegramReactionResponse, TelegramReactionSummary,
};
use super::rows::{row_to_telegram_provider_write_command, row_to_telegram_reaction};

const REACTION_PROVIDER_MISMATCH_ERROR: &str =
    "Provider observed a different reaction state than requested";

fn stable_short_hash(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())[..12].to_owned()
}

fn new_reaction_id() -> String {
    let now = Utc::now();
    format!(
        "tmsgreact_{}_{}",
        now.timestamp_millis(),
        stable_short_hash(&format!("react_{}", now.timestamp_nanos_opt().unwrap_or(0)))
    )
}

fn provider_reaction_id(message_id: &str, sender_id: &str, reaction_emoji: &str) -> String {
    format!(
        "tmsgreact_provider_{}",
        stable_short_hash(&format!("{message_id}\0{sender_id}\0{reaction_emoji}"))
    )
}

pub(in crate::integrations::telegram) struct TelegramReactionMessageRef<'a> {
    pub(in crate::integrations::telegram) message_id: &'a str,
    pub(in crate::integrations::telegram) account_id: &'a str,
    pub(in crate::integrations::telegram) provider_chat_id: &'a str,
    pub(in crate::integrations::telegram) provider_message_id: &'a str,
}

struct TelegramSelfReactionSync<'a> {
    sender_id: &'a str,
    chosen_reactions: &'a [String],
    observed_at: chrono::DateTime<Utc>,
}

pub(in crate::integrations::telegram) async fn sync_provider_reactions(
    pool: &PgPool,
    message_ref: TelegramReactionMessageRef<'_>,
    reactions: &[TdlibProviderReaction],
    self_sender_id: Option<&str>,
    chosen_reactions: &[String],
) -> Result<(), TelegramError> {
    let now = Utc::now();
    for reaction in reactions {
        let reaction_id = provider_reaction_id(
            message_ref.message_id,
            &reaction.sender_id,
            &reaction.reaction_emoji,
        );
        sqlx::query(
            r#"
            INSERT INTO telegram_message_reactions
                (reaction_id, message_id, account_id, provider_message_id, provider_chat_id,
                 sender_id, sender_display_name, reaction_emoji, is_active, observed_at,
                 source_event, provider_actor_id, metadata, provenance)
            VALUES ($1, $2, $3, $4, $5, $6, NULL, $7, true, $8,
                    'tdlib_interaction_info', $9, $10, $11)
            ON CONFLICT (message_id, sender_id, reaction_emoji)
            DO UPDATE SET
                is_active = true,
                observed_at = EXCLUDED.observed_at,
                source_event = EXCLUDED.source_event,
                provider_actor_id = EXCLUDED.provider_actor_id,
                metadata = EXCLUDED.metadata,
                provenance = EXCLUDED.provenance,
                updated_at = now()
            "#,
        )
        .bind(&reaction_id)
        .bind(message_ref.message_id)
        .bind(message_ref.account_id)
        .bind(message_ref.provider_message_id)
        .bind(message_ref.provider_chat_id)
        .bind(&reaction.sender_id)
        .bind(&reaction.reaction_emoji)
        .bind(now)
        .bind(&reaction.sender_id)
        .bind(json!({
            "is_outgoing": reaction.is_outgoing,
            "source": "tdlib_interaction_info",
        }))
        .bind(json!({
            "provider": "telegram",
            "runtime": "tdlib",
            "source": "interaction_info.reactions.recent_reactions",
        }))
        .execute(pool)
        .await?;
    }
    if let Some(self_sender_id) = self_sender_id {
        sync_self_provider_reactions(
            pool,
            &message_ref,
            TelegramSelfReactionSync {
                sender_id: self_sender_id,
                chosen_reactions,
                observed_at: now,
            },
        )
        .await?;
    }
    Ok(())
}

async fn sync_self_provider_reactions(
    pool: &PgPool,
    message_ref: &TelegramReactionMessageRef<'_>,
    sync: TelegramSelfReactionSync<'_>,
) -> Result<(), TelegramError> {
    sqlx::query(
        r#"
        UPDATE telegram_message_reactions
        SET is_active = false,
            observed_at = $4,
            source_event = 'tdlib_interaction_info',
            provider_actor_id = $3,
            metadata = jsonb_build_object(
                'source', 'tdlib_interaction_info',
                'is_chosen', false,
                'is_outgoing', true
            ),
            provenance = jsonb_build_object(
                'provider', 'telegram',
                'runtime', 'tdlib',
                'source', 'interaction_info.reactions.reactions'
            ),
            updated_at = now()
        WHERE message_id = $1
          AND sender_id = $2
          AND (
              COALESCE(array_length($5::text[], 1), 0) = 0
              OR reaction_emoji <> ALL($5::text[])
          )
          AND is_active = true
        "#,
    )
    .bind(message_ref.message_id)
    .bind(sync.sender_id)
    .bind(sync.sender_id)
    .bind(sync.observed_at)
    .bind(sync.chosen_reactions)
    .execute(pool)
    .await?;

    for reaction_emoji in sync.chosen_reactions {
        let reaction_id =
            provider_reaction_id(message_ref.message_id, sync.sender_id, reaction_emoji);
        sqlx::query(
            r#"
            INSERT INTO telegram_message_reactions
                (reaction_id, message_id, account_id, provider_message_id, provider_chat_id,
                 sender_id, sender_display_name, reaction_emoji, is_active, observed_at,
                 source_event, provider_actor_id, metadata, provenance)
            VALUES ($1, $2, $3, $4, $5, $6, NULL, $7, true, $8,
                    'tdlib_interaction_info', $6, $9, $10)
            ON CONFLICT (message_id, sender_id, reaction_emoji)
            DO UPDATE SET
                is_active = true,
                observed_at = EXCLUDED.observed_at,
                source_event = EXCLUDED.source_event,
                provider_actor_id = EXCLUDED.provider_actor_id,
                metadata = EXCLUDED.metadata,
                provenance = EXCLUDED.provenance,
                updated_at = now()
            "#,
        )
        .bind(&reaction_id)
        .bind(message_ref.message_id)
        .bind(message_ref.account_id)
        .bind(message_ref.provider_message_id)
        .bind(message_ref.provider_chat_id)
        .bind(sync.sender_id)
        .bind(reaction_emoji)
        .bind(sync.observed_at)
        .bind(json!({
            "is_outgoing": true,
            "is_chosen": true,
            "source": "tdlib_interaction_info",
        }))
        .bind(json!({
            "provider": "telegram",
            "runtime": "tdlib",
            "source": "interaction_info.reactions.reactions",
        }))
        .execute(pool)
        .await?;
    }

    Ok(())
}

pub async fn reconcile_reaction_commands_from_provider_message_state(
    pool: &PgPool,
    account_id: &str,
    provider_chat_id: &str,
    provider_message_id: &str,
    chosen_reactions: &[String],
    observed_at: chrono::DateTime<Utc>,
    observed_via: &str,
) -> Result<Vec<TelegramProviderWriteCommand>, TelegramError> {
    let chosen_reactions = normalized_reaction_emojis(chosen_reactions);
    let rows = sqlx::query(
        r#"
        SELECT *
        FROM telegram_provider_write_commands
        WHERE account_id = $1
          AND provider_chat_id = $2
          AND provider_message_id = $3
          AND command_kind IN ('react', 'unreact')
          AND status IN ('queued', 'retrying', 'executing')
          AND confirmation_decision IN ('confirmed', 'not_required')
          AND capability_state IN ('available', 'degraded')
        ORDER BY created_at ASC, command_id ASC
        "#,
    )
    .bind(account_id)
    .bind(provider_chat_id)
    .bind(provider_message_id)
    .fetch_all(pool)
    .await?;

    let mut reconciled = Vec::new();
    for row in rows {
        let command = row_to_telegram_provider_write_command(row)?;
        let Some(reaction_emoji) = command
            .payload
            .get("reaction_emoji")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
        else {
            continue;
        };
        let is_chosen = chosen_reactions.iter().any(|emoji| emoji == reaction_emoji);
        let should_reconcile = match command.command_kind.as_str() {
            "react" => is_chosen,
            "unreact" => !is_chosen,
            _ => false,
        };
        if !should_reconcile {
            let expected_is_chosen = match command.command_kind.as_str() {
                "react" => Some(true),
                "unreact" => Some(false),
                _ => None,
            };
            let Some(expected_is_chosen) = expected_is_chosen else {
                continue;
            };

            let provider_state = json!({
                "provider_chat_id": provider_chat_id,
                "provider_message_id": provider_message_id,
                "reaction_emoji": reaction_emoji,
                "expected_is_chosen": expected_is_chosen,
                "observed_is_chosen": is_chosen,
                "chosen_reactions": chosen_reactions,
                "observed_via": observed_via,
            });
            let result_payload = json!({
                "source": observed_via,
                "provider_chat_id": provider_chat_id,
                "provider_message_id": provider_message_id,
                "reaction_emoji": reaction_emoji,
                "expected_is_chosen": expected_is_chosen,
                "observed_is_chosen": is_chosen,
                "chosen_reactions": chosen_reactions,
                "provider_observed_at": observed_at,
                "mismatch": true,
            });
            let row = sqlx::query(
                r#"
                UPDATE telegram_provider_write_commands
                SET status = 'failed',
                    result_payload = $3,
                    last_error = $4,
                    provider_observed_at = $2,
                    provider_state = $5,
                    reconciliation_status = 'mismatch',
                    reconciled_at = $2,
                    completed_at = NULL,
                    locked_at = NULL,
                    locked_by = NULL,
                    next_attempt_at = NULL,
                    dead_lettered_at = NULL,
                    updated_at = $2
                WHERE command_id = $1
                RETURNING *
                "#,
            )
            .bind(&command.command_id)
            .bind(observed_at)
            .bind(&result_payload)
            .bind(REACTION_PROVIDER_MISMATCH_ERROR)
            .bind(&provider_state)
            .fetch_one(pool)
            .await?;
            reconciled.push(row_to_telegram_provider_write_command(row)?);
            continue;
        }

        let provider_state = json!({
            "provider_chat_id": provider_chat_id,
            "provider_message_id": provider_message_id,
            "reaction_emoji": reaction_emoji,
            "is_chosen": is_chosen,
            "chosen_reactions": chosen_reactions,
            "observed_via": observed_via,
        });
        let result_payload = json!({
            "source": observed_via,
            "provider_chat_id": provider_chat_id,
            "provider_message_id": provider_message_id,
            "reaction_emoji": reaction_emoji,
            "is_chosen": is_chosen,
            "chosen_reactions": chosen_reactions,
            "provider_observed_at": observed_at,
        });
        let row = sqlx::query(
            r#"
            UPDATE telegram_provider_write_commands
            SET status = 'completed',
                result_payload = $3,
                last_error = NULL,
                provider_observed_at = $2,
                provider_state = $4,
                reconciliation_status = 'observed',
                reconciled_at = $2,
                completed_at = $2,
                locked_at = NULL,
                locked_by = NULL,
                next_attempt_at = NULL,
                dead_lettered_at = NULL,
                updated_at = $2
            WHERE command_id = $1
            RETURNING *
            "#,
        )
        .bind(&command.command_id)
        .bind(observed_at)
        .bind(&result_payload)
        .bind(&provider_state)
        .fetch_one(pool)
        .await?;
        reconciled.push(row_to_telegram_provider_write_command(row)?);
    }

    Ok(reconciled)
}

fn normalized_reaction_emojis(reactions: &[String]) -> Vec<String> {
    let mut normalized = reactions
        .iter()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    normalized.sort();
    normalized.dedup();
    normalized
}

/// Add or update a reaction (sets is_active = true).
pub async fn add_reaction(
    pool: &PgPool,
    request: &TelegramReactionRequest,
    message_id: &str,
) -> Result<TelegramReactionResponse, TelegramError> {
    let now = Utc::now();
    let reaction_id = new_reaction_id();

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

pub async fn reaction_summary(
    pool: &PgPool,
    message_id: &str,
) -> Result<TelegramReactionSummary, TelegramError> {
    let reactions = list_reactions(pool, message_id).await?;

    let mut groups: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    for reaction in &reactions {
        groups
            .entry(reaction.reaction_emoji.clone())
            .or_default()
            .push(
                reaction
                    .sender_display_name
                    .clone()
                    .unwrap_or_else(|| reaction.sender_id.clone()),
            );
    }

    let reaction_groups = groups
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

#[cfg(test)]
mod tests {
    use sqlx::Row;
    use testkit::context::TestContext;

    use super::*;
    use crate::domains::mail::core::{
        CommunicationIngestionStore, CommunicationProviderKind, NewProviderAccount,
    };
    use crate::integrations::telegram::client::models::messages::TelegramReactionRequest;

    #[tokio::test]
    async fn provider_state_sync_deactivates_absent_self_reactions() {
        let ctx = TestContext::new().await;
        let pool = ctx.pool().clone();
        let account_id = create_telegram_account(&pool, "reaction-sync", "telegram:123").await;
        let message_id = "msg_reaction_sync";
        let provider_chat_id = "-100reaction-sync";
        let provider_message_id = "-100reaction-sync:77";
        let self_sender_id = "user:123";

        let add_request = TelegramReactionRequest {
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.to_owned(),
            provider_message_id: provider_message_id.to_owned(),
            sender_id: self_sender_id.to_owned(),
            sender_display_name: Some("Owner".to_owned()),
            reaction_emoji: "👍".to_owned(),
            command_id: None,
        };
        add_reaction(&pool, &add_request, message_id)
            .await
            .expect("add chosen reaction");

        let remove_request = TelegramReactionRequest {
            reaction_emoji: "🔥".to_owned(),
            ..add_request.clone()
        };
        add_reaction(&pool, &remove_request, message_id)
            .await
            .expect("add stale reaction");

        sync_provider_reactions(
            &pool,
            TelegramReactionMessageRef {
                message_id,
                account_id: &account_id,
                provider_chat_id,
                provider_message_id,
            },
            &[],
            Some(self_sender_id),
            &["👍".to_owned()],
        )
        .await
        .expect("sync provider reactions");

        let rows = sqlx::query(
            r#"
            SELECT reaction_emoji, is_active
            FROM telegram_message_reactions
            WHERE message_id = $1 AND sender_id = $2
            ORDER BY reaction_emoji ASC
            "#,
        )
        .bind(message_id)
        .bind(self_sender_id)
        .fetch_all(&pool)
        .await
        .expect("reaction rows");

        let states = rows
            .into_iter()
            .map(|row| {
                (
                    row.try_get::<String, _>("reaction_emoji")
                        .expect("reaction_emoji"),
                    row.try_get::<bool, _>("is_active").expect("is_active"),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            states,
            vec![("👍".to_owned(), true), ("🔥".to_owned(), false)]
        );
    }

    async fn create_telegram_account(
        pool: &sqlx::PgPool,
        suffix: &str,
        external_account_id: &str,
    ) -> String {
        let account_id = format!("telegram-reactions-{suffix}");
        CommunicationIngestionStore::new(pool.clone())
            .upsert_provider_account(
                &NewProviderAccount::new(
                    &account_id,
                    CommunicationProviderKind::TelegramUser,
                    format!("Telegram Reactions {suffix}"),
                    external_account_id.to_owned(),
                )
                .config(json!({"runtime": "tdlib_qr_authorized"})),
            )
            .await
            .expect("provider account");
        account_id
    }
}
