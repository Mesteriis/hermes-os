use chrono::Utc;
use serde_json::json;
use sqlx::{PgPool, Row};

use super::errors::TelegramError;
use super::models::TelegramChatMember;
use super::participants::capture_chat_participant_observation_in_transaction;

fn row_to_provider_member(row: sqlx::postgres::PgRow) -> Result<TelegramChatMember, TelegramError> {
    Ok(TelegramChatMember {
        sender_id: row.try_get("provider_member_id")?,
        sender_display_name: row.try_get("display_name")?,
        message_count: 0,
        last_message_at: None,
        source: row.try_get("source")?,
        provider_member_id: row.try_get("provider_member_id")?,
        username: row.try_get("username")?,
        role: row.try_get("role")?,
        status: row.try_get("status")?,
        is_admin: row.try_get("is_admin")?,
        is_owner: row.try_get("is_owner")?,
        permissions: row.try_get("permissions")?,
        observed_at: row.try_get("observed_at")?,
    })
}

pub async fn mark_absent_members_from_exhaustive_roster(
    pool: &PgPool,
    telegram_chat_id: &str,
    observed_member_ids: &[String],
    observed_via: &str,
) -> Result<Vec<TelegramChatMember>, TelegramError> {
    let observed_at = Utc::now();
    let permissions_patch = json!({
        "membership_state": "absent_exhaustive",
        "observed_via": observed_via,
    });
    let raw_payload_patch = json!({
        "membership_state": "absent_exhaustive",
        "observed_via": observed_via,
    });
    let mut transaction = pool.begin().await?;
    let rows = sqlx::query(
        r#"
        UPDATE telegram_chat_participants
        SET status = 'absent_exhaustive',
            permissions = COALESCE(permissions, '{}'::jsonb) || $3::jsonb,
            raw_payload = COALESCE(raw_payload, '{}'::jsonb) || $4::jsonb,
            observed_at = $2,
            updated_at = $2
        WHERE telegram_chat_id = $1
          AND source = 'tdlib'
          AND provider_member_id <> ALL($5)
          AND status IS DISTINCT FROM 'absent_exhaustive'
        RETURNING account_id, provider_chat_id, provider_member_id, display_name, username, role,
                  status, is_admin, is_owner, permissions, raw_payload, source, observed_at
        "#,
    )
    .bind(telegram_chat_id)
    .bind(observed_at)
    .bind(&permissions_patch)
    .bind(&raw_payload_patch)
    .bind(observed_member_ids)
    .fetch_all(&mut *transaction)
    .await
    .map_err(TelegramError::from)?;

    let mut members = Vec::with_capacity(rows.len());
    for row in rows {
        let account_id: String = row.try_get("account_id")?;
        let provider_chat_id: String = row.try_get("provider_chat_id")?;
        let raw_payload: serde_json::Value = row.try_get("raw_payload")?;
        let member = row_to_provider_member(row)?;
        capture_chat_participant_observation_in_transaction(
            &mut transaction,
            telegram_chat_id,
            &account_id,
            &provider_chat_id,
            &member,
            &raw_payload,
            "absent_exhaustive",
            "telegram.client.participant_roster.mark_absent_members_from_exhaustive_roster",
            member.observed_at.unwrap_or(observed_at),
        )
        .await?;
        members.push(member);
    }
    transaction.commit().await?;
    Ok(members)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::storage::Database;
    use crate::vault::CommunicationProviderAccountStore;
    use serde_json::json;
    use testkit::context::TestContext;

    #[tokio::test]
    async fn marks_stale_tdlib_participants_as_absent_from_exhaustive_roster() {
        let ctx = TestContext::new().await;
        let database = Database::connect(Some(&ctx.connection_string()))
            .await
            .expect("database connection");
        let pool = database.pool().expect("pool").clone();

        CommunicationProviderAccountStore::new(pool.clone())
            .upsert_runtime_account(
                "acct-1",
                "telegram_user",
                "Telegram Test Account",
                "telegram:1",
                json!({}),
            )
            .await
            .expect("insert provider account");

        sqlx::query(
            r#"
            INSERT INTO telegram_chats (
                telegram_chat_id,
                account_id,
                provider_chat_id,
                chat_kind,
                title,
                username,
                sync_state,
                last_message_at,
                metadata,
                created_at,
                updated_at
            )
            VALUES (
                'tgchat-1',
                'acct-1',
                'provider-chat-1',
                'group',
                'Roster Room',
                NULL,
                'synced',
                NULL,
                '{}'::jsonb,
                NOW(),
                NOW()
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("insert telegram chat");

        sqlx::query(
            r#"
            INSERT INTO telegram_chat_participants (
                participant_id, telegram_chat_id, account_id, provider_chat_id, provider_member_id,
                display_name, username, role, status, is_admin, is_owner, permissions, raw_payload,
                source, observed_at, created_at, updated_at
            )
            VALUES
                ('participant-1', 'tgchat-1', 'acct-1', 'provider-chat-1', 'user:1', 'User One', NULL, 'member', 'member', false, false, '{}'::jsonb, '{}'::jsonb, 'tdlib', NOW(), NOW(), NOW()),
                ('participant-2', 'tgchat-1', 'acct-1', 'provider-chat-1', 'user:2', 'User Two', NULL, 'member', 'member', false, false, '{}'::jsonb, '{}'::jsonb, 'tdlib', NOW(), NOW(), NOW())
            "#,
        )
        .execute(&pool)
        .await
        .expect("insert participants");

        let updated = mark_absent_members_from_exhaustive_roster(
            &pool,
            "tgchat-1",
            &[String::from("user:1")],
            "tdlib.getSupergroupMembers.exhaustive_absence",
        )
        .await
        .expect("mark absent");

        assert_eq!(updated.len(), 1);
        assert_eq!(updated[0].provider_member_id, "user:2");
        assert_eq!(updated[0].status.as_deref(), Some("absent_exhaustive"));
        assert_eq!(
            updated[0].permissions["membership_state"],
            "absent_exhaustive"
        );
    }
}
