use super::*;

impl ProviderChannelMessageLookupPort for ProviderChannelMessageStore {
    fn message_by_id<'a>(
        &'a self,
        message_id: &'a str,
        channel_kinds: &'a [&'a str],
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<
                        Option<ProviderChannelMessage>,
                        ProviderCommunicationMessagePortError,
                    >,
                > + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            ProviderChannelMessageStore::message_by_id(self, message_id, channel_kinds).await
        })
    }

    fn message_by_provider_record_id<'a>(
        &'a self,
        account_id: &'a str,
        provider_record_id: &'a str,
        channel_kinds: &'a [&'a str],
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<
                        Option<ProviderChannelMessage>,
                        ProviderCommunicationMessagePortError,
                    >,
                > + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            ProviderChannelMessageStore::message_by_provider_record_id(
                self,
                account_id,
                provider_record_id,
                channel_kinds,
            )
            .await
        })
    }

    fn recent_messages<'a>(
        &'a self,
        account_id: Option<&'a str>,
        conversation_id: Option<&'a str>,
        channel_kinds: &'a [&'a str],
        limit: i64,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<
                        Vec<ProviderChannelMessage>,
                        ProviderCommunicationMessagePortError,
                    >,
                > + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            ProviderChannelMessageStore::recent_messages(
                self,
                account_id,
                conversation_id,
                channel_kinds,
                limit,
            )
            .await
        })
    }

    fn messages_by_ids<'a>(
        &'a self,
        message_ids: &'a [String],
        channel_kinds: &'a [&'a str],
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<
                        Vec<ProviderChannelMessage>,
                        ProviderCommunicationMessagePortError,
                    >,
                > + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            ProviderChannelMessageStore::messages_by_ids(self, message_ids, channel_kinds).await
        })
    }

    fn search_messages<'a>(
        &'a self,
        account_id: Option<&'a str>,
        conversation_id: Option<&'a str>,
        query: &'a str,
        channel_kinds: &'a [&'a str],
        limit: i64,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<
                        Vec<ProviderChannelMessage>,
                        ProviderCommunicationMessagePortError,
                    >,
                > + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            ProviderChannelMessageStore::search_messages(
                self,
                account_id,
                conversation_id,
                query,
                channel_kinds,
                limit,
            )
            .await
        })
    }

    fn pinned_messages<'a>(
        &'a self,
        account_id: &'a str,
        conversation_id: &'a str,
        channel_kinds: &'a [&'a str],
        limit: i64,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<
                        Vec<ProviderChannelMessage>,
                        ProviderCommunicationMessagePortError,
                    >,
                > + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            ProviderChannelMessageStore::pinned_messages(
                self,
                account_id,
                conversation_id,
                channel_kinds,
                limit,
            )
            .await
        })
    }

    fn body_text<'a>(
        &'a self,
        message_id: &'a str,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<Option<String>, ProviderCommunicationMessagePortError>,
                > + Send
                + 'a,
        >,
    > {
        Box::pin(async move { ProviderChannelMessageStore::body_text(self, message_id).await })
    }

    fn message_ids_by_metadata_string<'a>(
        &'a self,
        metadata_key: &'a str,
        metadata_value: &'a str,
        channel_kinds: &'a [&'a str],
        limit: i64,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<Vec<String>, ProviderCommunicationMessagePortError>,
                > + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            ProviderChannelMessageStore::message_ids_by_metadata_string(
                self,
                metadata_key,
                metadata_value,
                channel_kinds,
                limit,
            )
            .await
        })
    }

    fn message_id_by_provider_record_id<'a>(
        &'a self,
        account_id: &'a str,
        provider_record_id: &'a str,
        channel_kinds: &'a [&'a str],
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<Option<String>, ProviderCommunicationMessagePortError>,
                > + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            ProviderChannelMessageStore::message_id_by_provider_record_id(
                self,
                account_id,
                provider_record_id,
                channel_kinds,
            )
            .await
        })
    }

    fn reference_summaries<'a>(
        &'a self,
        message_ids: &'a [String],
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<
                        Vec<ProviderMessageReferenceSummary>,
                        ProviderCommunicationMessagePortError,
                    >,
                > + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            ProviderChannelMessageStore::reference_summaries(self, message_ids).await
        })
    }

    fn heuristic_members<'a>(
        &'a self,
        account_id: &'a str,
        conversation_id: &'a str,
        query: Option<&'a str>,
        channel_kinds: &'a [&'a str],
        limit: i64,
        offset: i64,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<
                        Vec<ProviderHeuristicMember>,
                        ProviderCommunicationMessagePortError,
                    >,
                > + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            ProviderChannelMessageStore::heuristic_members(
                self,
                account_id,
                conversation_id,
                query,
                channel_kinds,
                limit,
                offset,
            )
            .await
        })
    }

    fn attachment_anchor<'a>(
        &'a self,
        account_id: &'a str,
        conversation_id: &'a str,
        provider_record_id: &'a str,
        channel_kinds: &'a [&'a str],
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<
                        Option<ProviderMessageAttachmentAnchor>,
                        ProviderCommunicationMessagePortError,
                    >,
                > + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            ProviderChannelMessageStore::attachment_anchor(
                self,
                account_id,
                conversation_id,
                provider_record_id,
                channel_kinds,
            )
            .await
        })
    }

    fn unread_counts<'a>(
        &'a self,
        account_id: &'a str,
        conversation_id: &'a str,
        channel_kinds: &'a [&'a str],
        last_read_at: Option<DateTime<Utc>>,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<(i64, i64), ProviderCommunicationMessagePortError>,
                > + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            ProviderChannelMessageStore::unread_counts(
                self,
                account_id,
                conversation_id,
                channel_kinds,
                last_read_at,
            )
            .await
        })
    }
}
