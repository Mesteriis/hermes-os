use crate::platform::capabilities::{CapabilityActionClass, CapabilityDecision};

use super::helpers::{insert_non_empty, insert_optional, non_empty_optional};
use super::models::NewApiAuditRecord;

impl NewApiAuditRecord {
    pub fn automation_telegram_send_dry_run(
        actor_id: impl Into<String>,
        outbound_message_id: impl Into<String>,
        policy_id: impl Into<String>,
        template_id: impl Into<String>,
        account_id: impl Into<String>,
        provider_chat_id: impl Into<String>,
        rendered_preview_hash: impl Into<String>,
    ) -> Self {
        let outbound_message_id = outbound_message_id.into();
        let policy_id = policy_id.into();
        let template_id = template_id.into();
        let account_id = account_id.into();
        let provider_chat_id = provider_chat_id.into();
        let rendered_preview_hash = rendered_preview_hash.into();
        let decision =
            CapabilityDecision::scoped_automation_allowed("telegram.send", policy_id.clone());

        Self::automation_telegram_send_dry_run_decision(
            actor_id,
            TelegramSendDryRunAuditDecision {
                target_kind: "telegram_outbound_message",
                target_id: Some(outbound_message_id),
                policy_id,
                template_id: Some(template_id),
                account_id: Some(account_id),
                provider_chat_id,
                rendered_preview_hash: Some(rendered_preview_hash),
                decision: &decision,
            },
        )
    }

    pub fn automation_telegram_send_dry_run_rejected(
        actor_id: impl Into<String>,
        command_id: impl Into<String>,
        policy_id: impl Into<String>,
        provider_chat_id: impl Into<String>,
        decision: &CapabilityDecision,
    ) -> Self {
        Self::automation_telegram_send_dry_run_decision(
            actor_id,
            TelegramSendDryRunAuditDecision {
                target_kind: "telegram_send_request",
                target_id: non_empty_optional(command_id.into()),
                policy_id: policy_id.into(),
                template_id: None,
                account_id: None,
                provider_chat_id: provider_chat_id.into(),
                rendered_preview_hash: None,
                decision,
            },
        )
    }

    fn automation_telegram_send_dry_run_decision(
        actor_id: impl Into<String>,
        audit_decision: TelegramSendDryRunAuditDecision<'_>,
    ) -> Self {
        let mut metadata = audit_decision.decision.audit_metadata();
        let metadata_object = metadata
            .as_object_mut()
            .expect("capability decision metadata must be an object");
        insert_non_empty(metadata_object, "policy_id", audit_decision.policy_id);
        insert_optional(metadata_object, "template_id", audit_decision.template_id);
        insert_optional(metadata_object, "account_id", audit_decision.account_id);
        insert_non_empty(
            metadata_object,
            "provider_chat_id",
            audit_decision.provider_chat_id,
        );
        insert_optional(
            metadata_object,
            "rendered_preview_hash",
            audit_decision.rendered_preview_hash,
        );

        Self::new(
            actor_id,
            "automation.telegram_send.dry_run",
            "POST",
            "/api/v1/policies/telegram-send/dry-run",
            audit_decision.target_kind,
            audit_decision.target_id,
            metadata,
        )
    }

    pub fn telegram_message_send(
        actor_id: impl Into<String>,
        message_id: impl Into<String>,
        account_id: impl Into<String>,
        provider_chat_id: impl Into<String>,
        rendered_preview_hash: impl Into<String>,
    ) -> Self {
        let mut metadata = CapabilityDecision::explicit_user_allowed(
            CapabilityActionClass::ProviderWrite,
            "telegram.message.send",
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
            "rendered_preview_hash",
            rendered_preview_hash.into(),
        );

        Self::new(
            actor_id,
            "telegram.message.send",
            "POST",
            "/api/v1/integrations/telegram/messages/send",
            "telegram_message",
            Some(message_id.into()),
            metadata,
        )
    }

    pub fn telegram_media_upload(
        actor_id: impl Into<String>,
        command_id: impl Into<String>,
        account_id: impl Into<String>,
        provider_chat_id: impl Into<String>,
        attachment_id: Option<&str>,
        blob_id: Option<&str>,
        media_type: Option<&str>,
    ) -> Self {
        let command_id = command_id.into();
        let mut metadata = CapabilityDecision::explicit_user_allowed(
            CapabilityActionClass::ProviderWrite,
            "telegram.media.upload",
            "explicit_user_confirmation",
        )
        .audit_metadata();
        let metadata_object = metadata
            .as_object_mut()
            .expect("capability decision metadata must be an object");
        insert_non_empty(metadata_object, "account_id", account_id.into());
        insert_non_empty(metadata_object, "provider_chat_id", provider_chat_id.into());
        insert_optional(
            metadata_object,
            "attachment_id",
            attachment_id.map(ToOwned::to_owned),
        );
        insert_optional(metadata_object, "blob_id", blob_id.map(ToOwned::to_owned));
        insert_optional(
            metadata_object,
            "media_type",
            media_type.map(ToOwned::to_owned),
        );

        Self::new(
            actor_id,
            "telegram.media.upload",
            "POST",
            "/api/v1/integrations/telegram/media/upload",
            "telegram_media_upload_command",
            Some(command_id),
            metadata,
        )
    }

    pub fn telegram_account_logout(
        actor_id: impl Into<String>,
        account_id: impl Into<String>,
        provider_kind: impl Into<String>,
        lifecycle_state: impl Into<String>,
    ) -> Self {
        Self::telegram_account_lifecycle(
            actor_id,
            TelegramAccountLifecycleAudit {
                operation: "telegram.account.logout",
                method: "POST",
                path_template: "/api/v1/integrations/telegram/accounts/{account_id}/logout",
                capability: "telegram.account.logout",
                account_id: account_id.into(),
                provider_kind: provider_kind.into(),
                lifecycle_state: lifecycle_state.into(),
            },
        )
    }

    pub fn telegram_account_remove(
        actor_id: impl Into<String>,
        account_id: impl Into<String>,
        provider_kind: impl Into<String>,
        lifecycle_state: impl Into<String>,
    ) -> Self {
        Self::telegram_account_lifecycle(
            actor_id,
            TelegramAccountLifecycleAudit {
                operation: "telegram.account.remove",
                method: "DELETE",
                path_template: "/api/v1/integrations/telegram/accounts/{account_id}",
                capability: "telegram.account.remove",
                account_id: account_id.into(),
                provider_kind: provider_kind.into(),
                lifecycle_state: lifecycle_state.into(),
            },
        )
    }

    pub fn telegram_runtime_stop(
        actor_id: impl Into<String>,
        account_id: impl Into<String>,
        provider_kind: impl Into<String>,
        runtime_kind: impl Into<String>,
        status: impl Into<String>,
    ) -> Self {
        let account_id = account_id.into();
        let mut metadata = CapabilityDecision::explicit_user_allowed(
            CapabilityActionClass::LocalWrite,
            "telegram.runtime.stop",
            "explicit_user_confirmation",
        )
        .audit_metadata();
        let metadata_object = metadata
            .as_object_mut()
            .expect("capability decision metadata must be an object");
        insert_non_empty(metadata_object, "account_id", account_id.clone());
        insert_non_empty(metadata_object, "provider_kind", provider_kind.into());
        insert_non_empty(metadata_object, "runtime_kind", runtime_kind.into());
        insert_non_empty(metadata_object, "status", status.into());

        Self::new(
            actor_id,
            "telegram.runtime.stop",
            "POST",
            "/api/v1/integrations/telegram/runtime/stop",
            "communication_provider_account",
            Some(account_id),
            metadata,
        )
    }

    pub fn telegram_runtime_restart(
        actor_id: impl Into<String>,
        account_id: impl Into<String>,
        provider_kind: impl Into<String>,
        runtime_kind: impl Into<String>,
        status: impl Into<String>,
    ) -> Self {
        let account_id = account_id.into();
        let mut metadata = CapabilityDecision::explicit_user_allowed(
            CapabilityActionClass::LocalWrite,
            "telegram.runtime.restart",
            "explicit_user_confirmation",
        )
        .audit_metadata();
        let metadata_object = metadata
            .as_object_mut()
            .expect("capability decision metadata must be an object");
        insert_non_empty(metadata_object, "account_id", account_id.clone());
        insert_non_empty(metadata_object, "provider_kind", provider_kind.into());
        insert_non_empty(metadata_object, "runtime_kind", runtime_kind.into());
        insert_non_empty(metadata_object, "status", status.into());

        Self::new(
            actor_id,
            "telegram.runtime.restart",
            "POST",
            "/api/v1/integrations/telegram/runtime/restart",
            "communication_provider_account",
            Some(account_id),
            metadata,
        )
    }

    fn telegram_account_lifecycle(
        actor_id: impl Into<String>,
        audit: TelegramAccountLifecycleAudit,
    ) -> Self {
        let mut metadata = CapabilityDecision::explicit_user_allowed(
            CapabilityActionClass::LocalWrite,
            audit.capability,
            "explicit_user_confirmation",
        )
        .audit_metadata();
        let metadata_object = metadata
            .as_object_mut()
            .expect("capability decision metadata must be an object");
        insert_non_empty(metadata_object, "account_id", audit.account_id.clone());
        insert_non_empty(metadata_object, "provider_kind", audit.provider_kind);
        insert_non_empty(metadata_object, "lifecycle_state", audit.lifecycle_state);

        Self::new(
            actor_id,
            audit.operation,
            audit.method,
            audit.path_template,
            "communication_provider_account",
            Some(audit.account_id),
            metadata,
        )
    }
    pub fn telegram_message_edit(
        actor_id: impl Into<String>,
        message_id: impl Into<String>,
        account_id: impl Into<String>,
        provider_chat_id: impl Into<String>,
    ) -> Self {
        let mut metadata = CapabilityDecision::explicit_user_allowed(
            CapabilityActionClass::ProviderWrite,
            "telegram.message.edit",
            "explicit_user_confirmation",
        )
        .audit_metadata();
        let metadata_object = metadata
            .as_object_mut()
            .expect("capability decision metadata must be an object");
        insert_non_empty(metadata_object, "account_id", account_id.into());
        insert_non_empty(metadata_object, "provider_chat_id", provider_chat_id.into());

        Self::new(
            actor_id,
            "telegram.message.edit",
            "POST",
            "/api/v1/integrations/telegram/messages/{message_id}/edit",
            "telegram_message",
            Some(message_id.into()),
            metadata,
        )
    }

    pub fn telegram_message_delete(
        actor_id: impl Into<String>,
        message_id: impl Into<String>,
        account_id: impl Into<String>,
        provider_chat_id: impl Into<String>,
    ) -> Self {
        let mut metadata = CapabilityDecision::explicit_user_allowed(
            CapabilityActionClass::Destructive,
            "telegram.message.delete",
            "explicit_user_confirmation",
        )
        .audit_metadata();
        let metadata_object = metadata
            .as_object_mut()
            .expect("capability decision metadata must be an object");
        insert_non_empty(metadata_object, "account_id", account_id.into());
        insert_non_empty(metadata_object, "provider_chat_id", provider_chat_id.into());

        Self::new(
            actor_id,
            "telegram.message.delete",
            "POST",
            "/api/v1/integrations/telegram/messages/{message_id}/delete",
            "telegram_message",
            Some(message_id.into()),
            metadata,
        )
    }

    pub fn telegram_topic_create(
        actor_id: impl Into<String>,
        telegram_chat_id: impl Into<String>,
        account_id: impl Into<String>,
        provider_chat_id: impl Into<String>,
    ) -> Self {
        let mut metadata = CapabilityDecision::explicit_user_allowed(
            CapabilityActionClass::ProviderWrite,
            "telegram.topic.create",
            "explicit_user_confirmation",
        )
        .audit_metadata();
        let metadata_object = metadata
            .as_object_mut()
            .expect("capability decision metadata must be an object");
        insert_non_empty(metadata_object, "account_id", account_id.into());
        insert_non_empty(metadata_object, "provider_chat_id", provider_chat_id.into());

        Self::new(
            actor_id,
            "telegram.topic.create",
            "POST",
            "/api/v1/integrations/telegram/conversations/{telegram_chat_id}/topics",
            "telegram_chat",
            Some(telegram_chat_id.into()),
            metadata,
        )
    }

    pub fn telegram_topic_close(
        actor_id: impl Into<String>,
        topic_id: impl Into<String>,
        account_id: impl Into<String>,
        provider_chat_id: impl Into<String>,
        is_closed: bool,
    ) -> Self {
        let capability = if is_closed {
            "telegram.topic.close"
        } else {
            "telegram.topic.reopen"
        };
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
        insert_non_empty(metadata_object, "is_closed", is_closed.to_string());

        Self::new(
            actor_id,
            capability,
            "POST",
            "/api/v1/integrations/telegram/topics/{topic_id}/close",
            "telegram_topic",
            Some(topic_id.into()),
            metadata,
        )
    }

    pub fn telegram_message_restore_visibility(
        actor_id: impl Into<String>,
        message_id: impl Into<String>,
        account_id: impl Into<String>,
        provider_chat_id: impl Into<String>,
    ) -> Self {
        let mut metadata = CapabilityDecision::explicit_user_allowed(
            CapabilityActionClass::LocalWrite,
            "telegram.message.restore_visibility",
            "explicit_user_confirmation",
        )
        .audit_metadata();
        let metadata_object = metadata
            .as_object_mut()
            .expect("capability decision metadata must be an object");
        insert_non_empty(metadata_object, "account_id", account_id.into());
        insert_non_empty(metadata_object, "provider_chat_id", provider_chat_id.into());

        Self::new(
            actor_id,
            "telegram.message.restore_visibility",
            "POST",
            "/api/v1/integrations/telegram/messages/{message_id}/restore-visibility",
            "telegram_message",
            Some(message_id.into()),
            metadata,
        )
    }

    pub fn telegram_message_pin(
        actor_id: impl Into<String>,
        message_id: impl Into<String>,
        account_id: impl Into<String>,
        provider_chat_id: impl Into<String>,
        is_pinned: bool,
    ) -> Self {
        let mut metadata = CapabilityDecision::explicit_user_allowed(
            CapabilityActionClass::LocalWrite,
            "telegram.message.pin",
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
            "operation",
            if is_pinned { "pin" } else { "unpin" }.to_owned(),
        );

        Self::new(
            actor_id,
            if is_pinned {
                "telegram.message.pin"
            } else {
                "telegram.message.unpin"
            },
            "POST",
            "/api/v1/integrations/telegram/messages/{message_id}/pin",
            "telegram_message",
            Some(message_id.into()),
            metadata,
        )
    }

    pub fn telegram_message_mark_read(
        actor_id: impl Into<String>,
        message_id: impl Into<String>,
        account_id: impl Into<String>,
        provider_chat_id: impl Into<String>,
        provider_message_id: impl Into<String>,
    ) -> Self {
        let mut metadata = CapabilityDecision::explicit_user_allowed(
            CapabilityActionClass::ProviderWrite,
            "telegram.message.mark_read",
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
            "provider_message_id",
            provider_message_id.into(),
        );

        Self::new(
            actor_id,
            "telegram.message.mark_read",
            "POST",
            "/api/v1/integrations/telegram/messages/{message_id}/mark-read",
            "telegram_message",
            Some(message_id.into()),
            metadata,
        )
    }

    pub fn telegram_message_reaction(
        actor_id: impl Into<String>,
        message_id: impl Into<String>,
        account_id: impl Into<String>,
        provider_chat_id: impl Into<String>,
        reaction_emoji: impl Into<String>,
        is_active: bool,
    ) -> Self {
        let capability = if is_active {
            "telegram.message.react"
        } else {
            "telegram.message.unreact"
        };
        let path_template = "/api/v1/integrations/telegram/messages/{message_id}/reactions";
        let mut metadata = CapabilityDecision::explicit_user_allowed(
            CapabilityActionClass::LocalWrite,
            capability,
            "explicit_user_confirmation",
        )
        .audit_metadata();
        let metadata_object = metadata
            .as_object_mut()
            .expect("capability decision metadata must be an object");
        insert_non_empty(metadata_object, "account_id", account_id.into());
        insert_non_empty(metadata_object, "provider_chat_id", provider_chat_id.into());
        insert_non_empty(metadata_object, "reaction_emoji", reaction_emoji.into());
        insert_non_empty(
            metadata_object,
            "operation",
            if is_active {
                "add".to_owned()
            } else {
                "remove".to_owned()
            },
        );

        Self::new(
            actor_id,
            if is_active {
                "telegram.message.react"
            } else {
                "telegram.message.unreact"
            },
            if is_active { "POST" } else { "DELETE" },
            path_template,
            "telegram_message",
            Some(message_id.into()),
            metadata,
        )
    }
}

struct TelegramSendDryRunAuditDecision<'a> {
    target_kind: &'static str,
    target_id: Option<String>,
    policy_id: String,
    template_id: Option<String>,
    account_id: Option<String>,
    provider_chat_id: String,
    rendered_preview_hash: Option<String>,
    decision: &'a CapabilityDecision,
}

struct TelegramAccountLifecycleAudit {
    operation: &'static str,
    method: &'static str,
    path_template: &'static str,
    capability: &'static str,
    account_id: String,
    provider_kind: String,
    lifecycle_state: String,
}
