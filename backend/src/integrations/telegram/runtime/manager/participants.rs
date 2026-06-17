use chrono::Utc;
use serde_json::Value;

use crate::integrations::telegram::client::participants::{
    reconcile_join_commands_from_provider_roster, telegram_self_provider_member_id,
};
use crate::integrations::telegram::client::{
    NewTelegramChatParticipant, TelegramChatMember, TelegramError, TelegramStore,
};

use super::super::commands::request_actor_get_supergroup_members;
use super::super::status::account_runtime_kind;
use super::account::load_active_account;
use super::realtime_events::publish_command_reconciled_events;
use super::{TelegramMemberSyncContext, TelegramRuntimeManager};

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

        let Some(supergroup_id) = tdlib_supergroup_id(&chat.metadata) else {
            return Ok(Vec::new());
        };

        let account = load_active_account(context.communication_store, &chat.account_id).await?;
        let runtime_kind = account_runtime_kind(&account);
        if runtime_kind != "tdlib_qr_authorized" {
            return Ok(Vec::new());
        }

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
            request_actor_get_supergroup_members(command_tx, supergroup_id, 100).await?;
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
            items.push(
                crate::integrations::telegram::client::participants::upsert_chat_participant(
                    context.telegram_store.pool(),
                    &participant,
                )
                .await?,
            );
        }

        if let Some(provider_member_id) =
            telegram_self_provider_member_id(&account.external_account_id)
            && items
                .iter()
                .any(|item| is_active_provider_member(item, &provider_member_id))
        {
            let commands = reconcile_join_commands_from_provider_roster(
                context.telegram_store.pool(),
                &chat.account_id,
                &chat.provider_chat_id,
                &provider_member_id,
                Utc::now(),
            )
            .await?;
            for command in commands {
                publish_command_reconciled_events(
                    context.event_bridge.as_ref(),
                    &command,
                    "tdlib.getSupergroupMembers",
                )
                .await;
            }
        }

        Ok(items)
    }
}

fn tdlib_supergroup_id(metadata: &Value) -> Option<i64> {
    metadata
        .get("tdlib_supergroup_id")
        .and_then(|value| value.as_i64().or_else(|| value.as_str()?.parse().ok()))
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
