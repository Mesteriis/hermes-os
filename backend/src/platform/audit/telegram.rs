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
            "/api/v1/telegram/messages/send",
            "telegram_message",
            Some(message_id.into()),
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
                path_template: "/api/v1/telegram/accounts/{account_id}/logout",
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
                path_template: "/api/v1/telegram/accounts/{account_id}",
                capability: "telegram.account.remove",
                account_id: account_id.into(),
                provider_kind: provider_kind.into(),
                lifecycle_state: lifecycle_state.into(),
            },
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
