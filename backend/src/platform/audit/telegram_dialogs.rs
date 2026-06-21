use crate::platform::capabilities::{CapabilityActionClass, CapabilityDecision};

use super::helpers::{insert_non_empty, insert_optional};
use super::models::NewApiAuditRecord;

impl NewApiAuditRecord {
    pub fn telegram_chat_action(
        actor_id: impl Into<String>,
        telegram_chat_id: Option<&str>,
        account_id: impl Into<String>,
        provider_chat_id: impl Into<String>,
        provider_message_id: Option<&str>,
        command_kind: &str,
    ) -> Self {
        let capability = match command_kind {
            "pin" | "unpin" => "telegram.dialog.pin",
            "archive" | "unarchive" => "telegram.dialog.archive",
            "mute" | "unmute" => "telegram.dialog.mute",
            "folder_add" => "telegram.dialog.folder_add",
            "folder_remove" => "telegram.dialog.folder_remove",
            "folder_reassign" => "telegram.dialog.folder_reassign",
            "mark_read" | "mark_unread" => "telegram.dialog.mark_read",
            "join" => "telegram.participants.join",
            "leave" => "telegram.participants.leave",
            _ => "telegram.dialog.action",
        };
        let path_template = match command_kind {
            "join" => "/api/v1/integrations/telegram/provider-commands/conversations/join",
            "leave" => {
                "/api/v1/integrations/telegram/provider-commands/conversations/{telegram_chat_id}/leave"
            }
            "pin" => {
                "/api/v1/integrations/telegram/provider-commands/conversations/{telegram_chat_id}/pin"
            }
            "unpin" => {
                "/api/v1/integrations/telegram/provider-commands/conversations/{telegram_chat_id}/unpin"
            }
            "archive" => {
                "/api/v1/integrations/telegram/provider-commands/conversations/{telegram_chat_id}/archive"
            }
            "unarchive" => {
                "/api/v1/integrations/telegram/provider-commands/conversations/{telegram_chat_id}/unarchive"
            }
            "mute" => {
                "/api/v1/integrations/telegram/provider-commands/conversations/{telegram_chat_id}/mute"
            }
            "unmute" => {
                "/api/v1/integrations/telegram/provider-commands/conversations/{telegram_chat_id}/unmute"
            }
            "folder_add" => {
                "/api/v1/integrations/telegram/provider-commands/conversations/{telegram_chat_id}/folders/{provider_folder_id}"
            }
            "folder_remove" => {
                "/api/v1/integrations/telegram/provider-commands/conversations/{telegram_chat_id}/folders/{provider_folder_id}/remove"
            }
            "folder_reassign" => {
                "/api/v1/integrations/telegram/provider-commands/conversations/{telegram_chat_id}/folders/reassign"
            }
            "mark_read" => {
                "/api/v1/integrations/telegram/provider-commands/conversations/{telegram_chat_id}/read"
            }
            "mark_unread" => {
                "/api/v1/integrations/telegram/provider-commands/conversations/{telegram_chat_id}/unread"
            }
            _ => {
                "/api/v1/integrations/telegram/provider-commands/conversations/{telegram_chat_id}/action"
            }
        };
        let operation = match command_kind {
            "pin" => "telegram.chat.pin",
            "unpin" => "telegram.chat.unpin",
            "archive" => "telegram.chat.archive",
            "unarchive" => "telegram.chat.unarchive",
            "mute" => "telegram.chat.mute",
            "unmute" => "telegram.chat.unmute",
            "folder_add" => "telegram.chat.folder_add",
            "folder_remove" => "telegram.chat.folder_remove",
            "folder_reassign" => "telegram.chat.folder_reassign",
            "mark_read" => "telegram.chat.mark_read",
            "mark_unread" => "telegram.chat.mark_unread",
            "join" => "telegram.chat.join",
            "leave" => "telegram.chat.leave",
            _ => "telegram.chat.action",
        };
        let action_class = match command_kind {
            "pin" | "unpin" | "archive" | "unarchive" | "mute" | "unmute" | "folder_add"
            | "folder_remove" | "folder_reassign" | "mark_read" | "mark_unread" | "join"
            | "leave" => CapabilityActionClass::ProviderWrite,
            _ => CapabilityActionClass::LocalWrite,
        };

        let provider_chat_id = provider_chat_id.into();
        let target_id = telegram_chat_id
            .map(ToOwned::to_owned)
            .or_else(|| Some(provider_chat_id.clone()));
        let mut metadata = CapabilityDecision::explicit_user_allowed(
            action_class,
            capability,
            "explicit_user_confirmation",
        )
        .audit_metadata();
        let metadata_object = metadata
            .as_object_mut()
            .expect("capability decision metadata must be an object");
        insert_non_empty(metadata_object, "account_id", account_id.into());
        insert_non_empty(metadata_object, "provider_chat_id", provider_chat_id);
        insert_non_empty(metadata_object, "command_kind", command_kind.to_owned());
        insert_optional(
            metadata_object,
            "provider_message_id",
            provider_message_id.map(ToOwned::to_owned),
        );

        Self::new(
            actor_id,
            operation,
            "POST",
            path_template,
            "telegram_chat",
            target_id,
            metadata,
        )
    }

    pub fn telegram_chat_folder_add(
        actor_id: impl Into<String>,
        telegram_chat_id: &str,
        account_id: impl Into<String>,
        provider_chat_id: impl Into<String>,
        provider_folder_id: i64,
    ) -> Self {
        Self::telegram_chat_folder_mutation(
            actor_id,
            telegram_chat_id,
            account_id,
            provider_chat_id,
            provider_folder_id,
            "folder_add",
            "telegram.dialog.folder_add",
            "telegram.chat.folder_add",
            "POST",
        )
    }

    pub fn telegram_chat_folder_remove(
        actor_id: impl Into<String>,
        telegram_chat_id: &str,
        account_id: impl Into<String>,
        provider_chat_id: impl Into<String>,
        provider_folder_id: i64,
    ) -> Self {
        Self::telegram_chat_folder_mutation(
            actor_id,
            telegram_chat_id,
            account_id,
            provider_chat_id,
            provider_folder_id,
            "folder_remove",
            "telegram.dialog.folder_remove",
            "telegram.chat.folder_remove",
            "POST",
        )
    }

    pub fn telegram_chat_folder_reassign(
        actor_id: impl Into<String>,
        telegram_chat_id: &str,
        account_id: impl Into<String>,
        provider_chat_id: impl Into<String>,
        target_provider_folder_ids: &[i64],
        added_provider_folder_ids: &[i64],
        removed_provider_folder_ids: &[i64],
    ) -> Self {
        let mut metadata = CapabilityDecision::explicit_user_allowed(
            CapabilityActionClass::ProviderWrite,
            "telegram.dialog.folder_reassign",
            "explicit_user_confirmation",
        )
        .audit_metadata();
        let metadata_object = metadata
            .as_object_mut()
            .expect("capability decision metadata must be an object");
        insert_non_empty(metadata_object, "account_id", account_id.into());
        insert_non_empty(metadata_object, "provider_chat_id", provider_chat_id.into());
        insert_non_empty(
            metadata_object,
            "command_kind",
            "folder_reassign".to_owned(),
        );
        metadata_object.insert(
            "target_provider_folder_ids".to_owned(),
            serde_json::Value::Array(
                target_provider_folder_ids
                    .iter()
                    .copied()
                    .map(|value| serde_json::Value::Number(value.into()))
                    .collect(),
            ),
        );
        metadata_object.insert(
            "added_provider_folder_ids".to_owned(),
            serde_json::Value::Array(
                added_provider_folder_ids
                    .iter()
                    .copied()
                    .map(|value| serde_json::Value::Number(value.into()))
                    .collect(),
            ),
        );
        metadata_object.insert(
            "removed_provider_folder_ids".to_owned(),
            serde_json::Value::Array(
                removed_provider_folder_ids
                    .iter()
                    .copied()
                    .map(|value| serde_json::Value::Number(value.into()))
                    .collect(),
            ),
        );

        Self::new(
            actor_id,
            "telegram.chat.folder_reassign",
            "POST",
            "/api/v1/integrations/telegram/provider-commands/conversations/{telegram_chat_id}/folders/reassign",
            "telegram_chat",
            Some(telegram_chat_id.to_owned()),
            metadata,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn telegram_chat_folder_mutation(
        actor_id: impl Into<String>,
        telegram_chat_id: &str,
        account_id: impl Into<String>,
        provider_chat_id: impl Into<String>,
        provider_folder_id: i64,
        command_kind: &str,
        capability: &str,
        operation: &str,
        method: &str,
    ) -> Self {
        let mut metadata = CapabilityDecision::explicit_user_allowed(
            CapabilityActionClass::ProviderWrite,
            capability,
            "explicit_user_confirmation",
        )
        .audit_metadata();
        let metadata_object = metadata
            .as_object_mut()
            .expect("capability decision metadata must be an object");
        insert_non_empty(metadata_object, "account_id", account_id.into());
        insert_non_empty(metadata_object, "provider_chat_id", provider_chat_id.into());
        insert_non_empty(metadata_object, "command_kind", command_kind.to_owned());
        insert_non_empty(
            metadata_object,
            "provider_folder_id",
            provider_folder_id.to_string(),
        );

        Self::new(
            actor_id,
            operation,
            method,
            if command_kind == "folder_remove" {
                "/api/v1/integrations/telegram/provider-commands/conversations/{telegram_chat_id}/folders/{provider_folder_id}/remove"
            } else {
                "/api/v1/integrations/telegram/provider-commands/conversations/{telegram_chat_id}/folders/{provider_folder_id}"
            },
            "telegram_chat",
            Some(telegram_chat_id.to_owned()),
            metadata,
        )
    }
}
