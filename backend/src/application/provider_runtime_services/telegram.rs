use super::*;

impl TelegramProviderRuntimeApplicationService {
    pub(crate) fn new(store: TelegramProviderRuntimeStore) -> Self {
        Self { store }
    }

    pub(crate) async fn setup_fixture_account(
        &self,
        request: &TelegramAccountSetupRequest,
    ) -> Result<TelegramAccountSetupResponse, TelegramError> {
        self.store.setup_fixture_account(request).await
    }

    pub(crate) async fn setup_live_blocked_account(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &TelegramSecretVault,
        request: &TelegramLiveAccountSetupRequest,
    ) -> Result<TelegramAccountSetupResponse, TelegramError> {
        self.store
            .setup_live_blocked_account(secret_store, vault, request)
            .await
    }

    pub(crate) async fn list_accounts(
        &self,
        include_removed: bool,
    ) -> Result<Vec<TelegramAccount>, TelegramError> {
        self.store.list_accounts(include_removed).await
    }

    pub(crate) async fn telegram_account_record(
        &self,
        account_id: &str,
    ) -> Result<ProviderAccount, TelegramError> {
        self.store.telegram_account_record(account_id).await
    }

    pub(crate) async fn logout_account(
        &self,
        account_id: &str,
    ) -> Result<TelegramAccount, TelegramError> {
        self.store.logout_account(account_id).await
    }

    pub(crate) async fn remove_account(
        &self,
        account_id: &str,
    ) -> Result<TelegramAccount, TelegramError> {
        self.store.remove_account(account_id).await
    }

    pub(crate) async fn list_chats(
        &self,
        account_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<TelegramChat>, TelegramError> {
        self.store.list_chats(account_id, limit).await
    }

    pub(crate) async fn list_chat_group_filters(
        &self,
        account_id: Option<&str>,
    ) -> Result<
        Vec<crate::integrations::telegram::client::models::chats::TelegramChatGroupFilter>,
        TelegramError,
    > {
        self.store.list_chat_group_filters(account_id).await
    }

    pub(crate) async fn telegram_chat_by_id(
        &self,
        telegram_chat_id: &str,
    ) -> Result<Option<TelegramChat>, TelegramError> {
        self.store.telegram_chat_by_id(telegram_chat_id).await
    }

    pub(crate) async fn apply_local_telegram_chat_avatar(
        &self,
        telegram_chat_id: &str,
        avatar: &crate::integrations::telegram::client::chat_state::TelegramLocalChatAvatarUpdate,
    ) -> Result<serde_json::Value, TelegramError> {
        self.store
            .apply_local_chat_avatar(telegram_chat_id, avatar)
            .await
    }

    pub(crate) async fn list_chat_members(
        &self,
        telegram_chat_id: &str,
        query: Option<&str>,
        role: Option<&str>,
        limit: i64,
        cursor: Option<&str>,
    ) -> Result<Vec<TelegramChatMember>, TelegramError> {
        self.store
            .list_chat_members(telegram_chat_id, query, role, limit, cursor)
            .await
    }

    pub(crate) async fn search_messages(
        &self,
        account_id: Option<&str>,
        provider_chat_id: Option<&str>,
        query: &str,
        limit: i64,
    ) -> Result<Vec<TelegramMessage>, TelegramError> {
        self.store
            .search_messages(account_id, provider_chat_id, query, limit)
            .await
    }

    pub(crate) async fn search_chats(
        &self,
        account_id: Option<&str>,
        query: &str,
        limit: i64,
    ) -> Result<Vec<TelegramChat>, TelegramError> {
        self.store.search_chats(account_id, query, limit).await
    }

    pub(crate) async fn pinned_messages(
        &self,
        conversation_id: &str,
        limit: i64,
    ) -> Result<Vec<TelegramMessage>, TelegramError> {
        self.store.pinned_messages(conversation_id, limit).await
    }

    pub(crate) async fn recent_messages(
        &self,
        account_id: Option<&str>,
        provider_chat_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<TelegramMessage>, TelegramError> {
        self.store
            .recent_messages(account_id, provider_chat_id, limit)
            .await
    }

    pub(crate) async fn messages_by_ids(
        &self,
        message_ids: &[String],
    ) -> Result<Vec<TelegramMessage>, TelegramError> {
        self.store.messages_by_ids(message_ids).await
    }

    pub(crate) async fn message_by_id(
        &self,
        message_id: &str,
    ) -> Result<Option<TelegramMessage>, TelegramError> {
        self.store.message_by_id(message_id).await
    }

    pub(crate) async fn set_chat_metadata_bool(
        &self,
        telegram_chat_id: &str,
        key: &str,
        value: bool,
    ) -> Result<Value, TelegramError> {
        self.store
            .set_chat_metadata_bool(telegram_chat_id, key, value)
            .await
    }

    pub(crate) async fn set_chat_last_read_at(
        &self,
        telegram_chat_id: &str,
        last_read_at: Option<DateTime<Utc>>,
    ) -> Result<Value, TelegramError> {
        self.store
            .set_chat_last_read_at(telegram_chat_id, last_read_at)
            .await
    }

    pub(crate) async fn recompute_chat_unread_count(
        &self,
        telegram_chat_id: &str,
    ) -> Result<Value, TelegramError> {
        self.store
            .recompute_chat_unread_count(telegram_chat_id)
            .await
    }

    pub(crate) async fn list_commands(
        &self,
        account_id: &str,
        provider_chat_id: Option<&str>,
        provider_message_id: Option<&str>,
        command_kinds: &[String],
        limit: i64,
    ) -> Result<TelegramCommandListResponse, TelegramError> {
        let items = list_telegram_commands_filtered(
            self.store.pool(),
            account_id,
            provider_chat_id,
            provider_message_id,
            command_kinds,
            limit,
        )
        .await?;
        Ok(TelegramCommandListResponse { items })
    }

    pub(crate) async fn list_topics(
        &self,
        telegram_chat_id: &str,
        limit: i64,
    ) -> Result<TelegramTopicListResponse, TelegramError> {
        let items = list_telegram_topics(self.store.pool(), telegram_chat_id, limit).await?;
        Ok(TelegramTopicListResponse {
            telegram_chat_id: telegram_chat_id.to_owned(),
            items,
        })
    }

    pub(crate) async fn get_topic(
        &self,
        topic_id: &str,
    ) -> Result<Option<TelegramTopic>, TelegramError> {
        get_telegram_topic(self.store.pool(), topic_id).await
    }

    pub(crate) async fn list_topic_message_ids(
        &self,
        topic_id: &str,
        limit: i64,
    ) -> Result<Vec<String>, TelegramError> {
        list_telegram_topic_message_ids(&self.store, topic_id, limit).await
    }

    pub(crate) async fn search_topics(
        &self,
        telegram_chat_id: &str,
        query: &str,
        limit: i64,
    ) -> Result<Vec<TelegramTopic>, TelegramError> {
        search_telegram_topics_projection(self.store.pool(), telegram_chat_id, query, limit).await
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn insert_command(
        &self,
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
        payload: Value,
        target_ref: Value,
        audit_metadata: Value,
    ) -> Result<TelegramProviderWriteCommand, TelegramError> {
        commands::insert_command(
            self.store.pool(),
            command_id,
            account_id,
            command_kind,
            idempotency_key,
            provider_chat_id,
            provider_message_id,
            capability_state,
            action_class,
            confirmation_decision,
            actor_id,
            payload,
            target_ref,
            audit_metadata,
        )
        .await
    }

    pub(crate) async fn find_command_by_idempotency(
        &self,
        account_id: &str,
        idempotency_key: &str,
    ) -> Result<Option<TelegramProviderWriteCommand>, TelegramError> {
        commands::queries::find_command_by_idempotency(
            self.store.pool(),
            account_id,
            idempotency_key,
        )
        .await
    }

    pub(crate) async fn manual_retry_command(
        &self,
        command_id: &str,
        now: DateTime<Utc>,
    ) -> Result<Option<TelegramProviderWriteCommand>, TelegramError> {
        commands::manual_retry_command(self.store.pool(), command_id, now).await
    }

    pub(crate) async fn attachment_anchor_for_message(
        &self,
        account_id: &str,
        provider_chat_id: &str,
        provider_message_id: &str,
    ) -> Result<
        crate::integrations::telegram::client::models::messages::TelegramAttachmentAnchor,
        TelegramError,
    > {
        self.store
            .attachment_anchor_for_message(account_id, provider_chat_id, provider_message_id)
            .await
    }

    pub(crate) async fn update_message_attachment_download_state(
        &self,
        update: TelegramAttachmentDownloadStateUpdate<'_>,
    ) -> Result<(), TelegramError> {
        self.store
            .update_message_attachment_download_state(update)
            .await
    }

    pub(crate) async fn telegram_message_snapshot_payload(
        &self,
        message_id: &str,
        base_payload: Value,
    ) -> Result<Value, TelegramError> {
        crate::application::telegram_fixture_snapshot::message_snapshot_payload(
            &self.store,
            message_id,
            base_payload,
        )
        .await
    }
}
