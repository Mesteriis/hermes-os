use chrono::Utc;
use serde_json::{Value, json};

use crate::integrations::telegram::client::participants::{
    inactive_roster_membership_state, reconcile_join_commands_from_provider_roster_with_source,
    reconcile_leave_commands_from_exhaustive_absence,
    reconcile_leave_commands_from_provider_roster_with_source, telegram_self_provider_member_id,
};
use crate::integrations::telegram::client::{
    NewTelegramChatParticipant, TelegramChat, TelegramChatMember, TelegramError,
    mark_absent_members_from_exhaustive_roster,
};

use super::super::participant_commands::{
    request_actor_get_basic_group_members, request_actor_get_supergroup_administrators,
    request_actor_get_supergroup_members,
};
use super::super::status::account_runtime_kind;
use super::account::load_active_account;
use super::participant_events::publish_participant_updated_event;
use super::realtime_events::publish_command_reconciled_events;
use super::{TelegramMemberSyncContext, TelegramRuntimeManager};

const TELEGRAM_MEMBER_SYNC_TARGET_LIMIT: i32 = 500;

impl TelegramRuntimeManager {
    pub(crate) async fn sync_chat_members<S>(
        &self,
        context: TelegramMemberSyncContext<'_, S>,
        telegram_chat_id: &str,
    ) -> Result<Vec<TelegramChatMember>, TelegramError>
    where
        S: crate::platform::secrets::SecretResolver + Sync + ?Sized,
    {
        let chat = context
            .telegram_store
            .telegram_chat_by_id(telegram_chat_id)
            .await?
            .ok_or(TelegramError::InvalidRequest(format!(
                "chat {telegram_chat_id} not found"
            )))?;

        let account = load_active_account(context.communication_store, &chat.account_id).await?;
        let runtime_kind = account_runtime_kind(&account);
        if runtime_kind != "tdlib_qr_authorized" {
            return Ok(Vec::new());
        }

        if let Some(private_items) = sync_private_chat_members(
            context.telegram_store.pool(),
            &chat,
            &account.external_account_id,
        )
        .await?
        {
            for item in &private_items {
                publish_participant_updated_event(
                    context.event_bridge.as_ref(),
                    &chat.account_id,
                    &chat.telegram_chat_id,
                    &chat.provider_chat_id,
                    item,
                    "tdlib.chat.metadata",
                )
                .await;
            }
            return Ok(private_items);
        }

        if let Some(basic_group_id) = tdlib_basic_group_id(&chat.metadata) {
            let command_tx = self
                .ensure_tdlib_actor(
                    context.communication_store,
                    context.secret_store,
                    context.secret_resolver,
                    context.config,
                    &account,
                    context.event_bridge.clone(),
                )
                .await?;
            let snapshots =
                request_actor_get_basic_group_members(command_tx, basic_group_id).await?;
            return sync_provider_roster_snapshots(
                context,
                &chat,
                &account.external_account_id,
                snapshots,
                "tdlib.getBasicGroupFullInfo",
                true,
            )
            .await;
        }

        let Some(supergroup_id) = tdlib_supergroup_id(&chat.metadata) else {
            return Ok(Vec::new());
        };

        let command_tx = self
            .ensure_tdlib_actor(
                context.communication_store,
                context.secret_store,
                context.secret_resolver,
                context.config,
                &account,
                context.event_bridge.clone(),
            )
            .await?;

        let snapshots = request_actor_get_supergroup_members(
            command_tx.clone(),
            supergroup_id,
            TELEGRAM_MEMBER_SYNC_TARGET_LIMIT,
        )
        .await?;
        let roster_is_exhaustive = supergroup_roster_is_exhaustive(&snapshots);
        let admin_snapshots = request_actor_get_supergroup_administrators(
            command_tx,
            supergroup_id,
            TELEGRAM_MEMBER_SYNC_TARGET_LIMIT,
        )
        .await?;
        sync_provider_roster_snapshots(
            context,
            &chat,
            &account.external_account_id,
            merge_supergroup_member_snapshots(snapshots, admin_snapshots),
            "tdlib.getSupergroupMembers",
            roster_is_exhaustive,
        )
        .await
    }
}

fn tdlib_supergroup_id(metadata: &Value) -> Option<i64> {
    metadata
        .get("tdlib_supergroup_id")
        .and_then(|value| value.as_i64().or_else(|| value.as_str()?.parse().ok()))
}

fn tdlib_basic_group_id(metadata: &Value) -> Option<i64> {
    metadata
        .get("tdlib_basic_group_id")
        .and_then(|value| value.as_i64().or_else(|| value.as_str()?.parse().ok()))
}

async fn sync_provider_roster_snapshots<S>(
    context: TelegramMemberSyncContext<'_, S>,
    chat: &TelegramChat,
    external_account_id: &str,
    snapshots: Vec<crate::integrations::telegram::tdjson::TelegramTdlibChatMemberSnapshot>,
    observed_via: &str,
    roster_is_exhaustive: bool,
) -> Result<Vec<TelegramChatMember>, TelegramError>
where
    S: crate::platform::secrets::SecretResolver + Sync + ?Sized,
{
    let mut items = Vec::with_capacity(snapshots.len());
    for snapshot in snapshots {
        let participant = NewTelegramChatParticipant {
            participant_id: telegram_participant_id(
                &chat.telegram_chat_id,
                &snapshot.provider_member_id,
            ),
            telegram_chat_id: chat.telegram_chat_id.clone(),
            account_id: chat.account_id.clone(),
            provider_chat_id: chat.provider_chat_id.clone(),
            provider_member_id: snapshot.provider_member_id,
            display_name: snapshot.display_name,
            username: snapshot.username,
            role: snapshot.role,
            status: snapshot.status,
            is_admin: snapshot.is_admin,
            is_owner: snapshot.is_owner,
            permissions: snapshot.permissions,
            raw_payload: snapshot.raw,
            source: "tdlib".to_owned(),
        };
        let item = crate::integrations::telegram::client::participants::upsert_chat_participant(
            context.telegram_store.pool(),
            &participant,
        )
        .await?;
        publish_participant_updated_event(
            context.event_bridge.as_ref(),
            &chat.account_id,
            &chat.telegram_chat_id,
            &chat.provider_chat_id,
            &item,
            observed_via,
        )
        .await;
        items.push(item);
    }

    if roster_is_exhaustive {
        let observed_member_ids = items
            .iter()
            .map(|item| item.provider_member_id.clone())
            .collect::<Vec<_>>();
        let absent_members = mark_absent_members_from_exhaustive_roster(
            context.telegram_store.pool(),
            &chat.telegram_chat_id,
            &observed_member_ids,
            &format!("{observed_via}.exhaustive_absence"),
        )
        .await?;
        for member in absent_members {
            publish_participant_updated_event(
                context.event_bridge.as_ref(),
                &chat.account_id,
                &chat.telegram_chat_id,
                &chat.provider_chat_id,
                &member,
                &format!("{observed_via}.exhaustive_absence"),
            )
            .await;
        }
    }

    reconcile_self_membership_from_provider_roster(
        context,
        chat,
        external_account_id,
        observed_via,
        &items,
        roster_is_exhaustive,
    )
    .await?;

    Ok(items)
}

async fn sync_private_chat_members(
    pool: &sqlx::PgPool,
    chat: &TelegramChat,
    external_account_id: &str,
) -> Result<Option<Vec<TelegramChatMember>>, TelegramError> {
    let Some(private_user_id) = tdlib_private_user_id(&chat.metadata) else {
        return Ok(None);
    };

    let mut participants = Vec::new();
    if chat
        .metadata
        .get("is_saved_messages")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        if let Some(provider_member_id) = telegram_self_provider_member_id(external_account_id) {
            participants.push(build_private_chat_participant(
                chat,
                provider_member_id,
                Some(chat.title.clone()),
                chat.username.clone(),
            ));
        }
    } else {
        participants.push(build_private_chat_participant(
            chat,
            format!("user:{private_user_id}"),
            Some(chat.title.clone()),
            chat.username.clone(),
        ));
    }

    let mut items = Vec::with_capacity(participants.len());
    for participant in participants {
        items.push(
            crate::integrations::telegram::client::participants::upsert_chat_participant(
                pool,
                &participant,
            )
            .await?,
        );
    }

    Ok(Some(items))
}

fn tdlib_private_user_id(metadata: &Value) -> Option<String> {
    metadata
        .get("tdlib_private_user_id")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn build_private_chat_participant(
    chat: &TelegramChat,
    provider_member_id: String,
    display_name: Option<String>,
    username: Option<String>,
) -> NewTelegramChatParticipant {
    NewTelegramChatParticipant {
        participant_id: telegram_participant_id(&chat.telegram_chat_id, &provider_member_id),
        telegram_chat_id: chat.telegram_chat_id.clone(),
        account_id: chat.account_id.clone(),
        provider_chat_id: chat.provider_chat_id.clone(),
        provider_member_id,
        display_name,
        username,
        role: "member".to_owned(),
        status: "member".to_owned(),
        is_admin: false,
        is_owner: false,
        permissions: json!({
            "observed_via": "tdlib.chat.metadata",
            "tdlib_chat_type": chat.metadata.get("tdlib_chat_type").cloned().unwrap_or(Value::Null),
            "is_saved_messages": chat.metadata.get("is_saved_messages").and_then(Value::as_bool).unwrap_or(false),
        }),
        raw_payload: json!({
            "observed_via": "tdlib.chat.metadata",
            "tdlib_private_user_id": chat.metadata.get("tdlib_private_user_id").cloned().unwrap_or(Value::Null),
            "tdlib_chat_type": chat.metadata.get("tdlib_chat_type").cloned().unwrap_or(Value::Null),
            "is_saved_messages": chat.metadata.get("is_saved_messages").and_then(Value::as_bool).unwrap_or(false),
        }),
        source: "tdlib".to_owned(),
    }
}

fn telegram_participant_id(telegram_chat_id: &str, provider_member_id: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(telegram_chat_id.as_bytes());
    hasher.update(b"\0");
    hasher.update(provider_member_id.as_bytes());
    format!("telegram_participant:v1:{:x}", hasher.finalize())
}

fn is_active_provider_member(item: &TelegramChatMember, provider_member_id: &str) -> bool {
    if item.provider_member_id != provider_member_id {
        return false;
    }
    let inactive_status = matches!(item.status.as_deref(), Some("left" | "banned"));
    let inactive_role = matches!(item.role.as_deref(), Some("left" | "banned"));
    !(inactive_status || inactive_role)
}

async fn reconcile_self_membership_from_provider_roster<S>(
    context: TelegramMemberSyncContext<'_, S>,
    chat: &TelegramChat,
    external_account_id: &str,
    observed_via: &str,
    items: &[TelegramChatMember],
    roster_is_exhaustive: bool,
) -> Result<(), TelegramError>
where
    S: crate::platform::secrets::SecretResolver + Sync + ?Sized,
{
    if let Some(provider_member_id) = telegram_self_provider_member_id(external_account_id) {
        let commands = if let Some(self_member) = items
            .iter()
            .find(|item| item.provider_member_id == provider_member_id)
        {
            if is_active_provider_member(self_member, &provider_member_id) {
                reconcile_join_commands_from_provider_roster_with_source(
                    context.telegram_store.pool(),
                    &chat.account_id,
                    &chat.provider_chat_id,
                    &provider_member_id,
                    Utc::now(),
                    observed_via,
                )
                .await?
            } else if let Some(membership_state) = inactive_roster_membership_state(self_member) {
                reconcile_leave_commands_from_provider_roster_with_source(
                    context.telegram_store.pool(),
                    &chat.account_id,
                    &chat.provider_chat_id,
                    &provider_member_id,
                    membership_state,
                    self_member.status.as_deref(),
                    self_member.role.as_deref(),
                    Utc::now(),
                    observed_via,
                )
                .await?
            } else {
                Vec::new()
            }
        } else if roster_is_exhaustive {
            let observed_via = format!("{observed_via}.exhaustive_absence");
            reconcile_leave_commands_from_exhaustive_absence(
                context.telegram_store.pool(),
                &chat.account_id,
                &chat.provider_chat_id,
                &provider_member_id,
                Utc::now(),
                &observed_via,
            )
            .await?
        } else {
            Vec::new()
        };
        for command in commands {
            publish_command_reconciled_events(
                context.event_bridge.as_ref(),
                &command,
                observed_via,
            )
            .await;
        }
    }

    Ok(())
}

fn supergroup_roster_is_exhaustive(
    snapshots: &[crate::integrations::telegram::tdjson::TelegramTdlibChatMemberSnapshot],
) -> bool {
    snapshots.len() < TELEGRAM_MEMBER_SYNC_TARGET_LIMIT as usize
}

fn merge_supergroup_member_snapshots(
    recent: Vec<crate::integrations::telegram::tdjson::TelegramTdlibChatMemberSnapshot>,
    administrators: Vec<crate::integrations::telegram::tdjson::TelegramTdlibChatMemberSnapshot>,
) -> Vec<crate::integrations::telegram::tdjson::TelegramTdlibChatMemberSnapshot> {
    let mut merged = recent;
    let mut indexes = merged
        .iter()
        .enumerate()
        .map(|(index, item)| (item.provider_member_id.clone(), index))
        .collect::<std::collections::HashMap<_, _>>();

    for item in administrators {
        if let Some(index) = indexes.get(&item.provider_member_id).copied() {
            merged[index] = item;
        } else {
            indexes.insert(item.provider_member_id.clone(), merged.len());
            merged.push(item);
        }
    }

    merged
}

#[cfg(test)]
#[path = "participants_runtime_tests.rs"]
mod participants_runtime_tests;

#[cfg(test)]
mod tests {
    use super::{
        TELEGRAM_MEMBER_SYNC_TARGET_LIMIT, build_private_chat_participant,
        merge_supergroup_member_snapshots, supergroup_roster_is_exhaustive, tdlib_private_user_id,
    };
    use crate::integrations::telegram::client::TelegramChat;
    use chrono::Utc;
    use serde_json::json;

    #[test]
    fn tdlib_private_user_id_reads_private_chat_metadata() {
        assert_eq!(
            tdlib_private_user_id(&json!({"tdlib_private_user_id": "888"})).as_deref(),
            Some("888")
        );
        assert_eq!(tdlib_private_user_id(&json!({})), None);
    }

    #[test]
    fn build_private_chat_participant_marks_saved_messages_as_tdlib_projection() {
        let chat = TelegramChat {
            telegram_chat_id: "telegram-chat-1".to_owned(),
            account_id: "account-1".to_owned(),
            provider_chat_id: "777".to_owned(),
            chat_kind: "private".to_owned(),
            title: "Saved Messages".to_owned(),
            username: None,
            sync_state: "synced".to_owned(),
            last_message_at: None,
            metadata: json!({
                "tdlib_private_user_id": "777",
                "tdlib_chat_type": "chatTypePrivate",
                "is_saved_messages": true,
            }),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let participant = build_private_chat_participant(
            &chat,
            "user:777".to_owned(),
            Some("Saved Messages".to_owned()),
            None,
        );

        assert_eq!(participant.provider_member_id, "user:777");
        assert_eq!(participant.role, "member");
        assert_eq!(participant.status, "member");
        assert_eq!(participant.source, "tdlib");
        assert_eq!(
            participant.permissions["observed_via"],
            "tdlib.chat.metadata"
        );
        assert_eq!(participant.permissions["is_saved_messages"], true);
        assert_eq!(participant.raw_payload["tdlib_private_user_id"], "777");
    }

    #[test]
    fn supergroup_roster_is_exhaustive_only_below_target_limit() {
        let under_limit =
            vec![json!({}); (TELEGRAM_MEMBER_SYNC_TARGET_LIMIT as usize).saturating_sub(1)];
        let at_limit = vec![json!({}); TELEGRAM_MEMBER_SYNC_TARGET_LIMIT as usize];
        let under_limit = under_limit
            .into_iter()
            .map(
                |raw| crate::integrations::telegram::tdjson::TelegramTdlibChatMemberSnapshot {
                    provider_member_id: "user:1".to_owned(),
                    display_name: None,
                    username: None,
                    role: "member".to_owned(),
                    status: "member".to_owned(),
                    is_admin: false,
                    is_owner: false,
                    permissions: json!({}),
                    raw,
                },
            )
            .collect::<Vec<_>>();
        let at_limit = at_limit
            .into_iter()
            .enumerate()
            .map(|(index, raw)| {
                crate::integrations::telegram::tdjson::TelegramTdlibChatMemberSnapshot {
                    provider_member_id: format!("user:{index}"),
                    display_name: None,
                    username: None,
                    role: "member".to_owned(),
                    status: "member".to_owned(),
                    is_admin: false,
                    is_owner: false,
                    permissions: json!({}),
                    raw,
                }
            })
            .collect::<Vec<_>>();

        assert!(supergroup_roster_is_exhaustive(&under_limit));
        assert!(!supergroup_roster_is_exhaustive(&at_limit));
    }

    #[test]
    fn merge_supergroup_member_snapshots_prefers_administrator_snapshot() {
        let recent = vec![
            crate::integrations::telegram::tdjson::TelegramTdlibChatMemberSnapshot {
                provider_member_id: "user:1".to_owned(),
                display_name: Some("Owner User".to_owned()),
                username: None,
                role: "member".to_owned(),
                status: "member".to_owned(),
                is_admin: false,
                is_owner: false,
                permissions: json!({"source": "recent"}),
                raw: json!({"@type": "chatMember"}),
            },
            crate::integrations::telegram::tdjson::TelegramTdlibChatMemberSnapshot {
                provider_member_id: "user:2".to_owned(),
                display_name: Some("Recent User".to_owned()),
                username: None,
                role: "member".to_owned(),
                status: "member".to_owned(),
                is_admin: false,
                is_owner: false,
                permissions: json!({"source": "recent"}),
                raw: json!({"@type": "chatMember"}),
            },
        ];
        let administrators = vec![
            crate::integrations::telegram::tdjson::TelegramTdlibChatMemberSnapshot {
                provider_member_id: "user:1".to_owned(),
                display_name: Some("Owner User".to_owned()),
                username: None,
                role: "admin".to_owned(),
                status: "administrator".to_owned(),
                is_admin: true,
                is_owner: false,
                permissions: json!({"source": "administrators"}),
                raw: json!({"@type": "chatMember"}),
            },
            crate::integrations::telegram::tdjson::TelegramTdlibChatMemberSnapshot {
                provider_member_id: "user:3".to_owned(),
                display_name: Some("Admin Only".to_owned()),
                username: None,
                role: "admin".to_owned(),
                status: "administrator".to_owned(),
                is_admin: true,
                is_owner: false,
                permissions: json!({"source": "administrators"}),
                raw: json!({"@type": "chatMember"}),
            },
        ];

        let merged = merge_supergroup_member_snapshots(recent, administrators);

        assert_eq!(merged.len(), 3);
        assert_eq!(merged[0].provider_member_id, "user:1");
        assert_eq!(merged[0].role, "admin");
        assert_eq!(merged[0].permissions["source"], "administrators");
        assert_eq!(merged[1].provider_member_id, "user:2");
        assert_eq!(merged[2].provider_member_id, "user:3");
        assert!(merged[2].is_admin);
    }
}
