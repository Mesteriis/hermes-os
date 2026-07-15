use super::*;

impl ProviderChannelMessageCommandPort for ProviderChannelMessageStore {
    fn apply_metadata<'a>(
        &'a self,
        message_id: &'a str,
        metadata: &'a Value,
        context: ProviderMessageProjectionObservationContext<'a>,
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
            ProviderChannelMessageStore::apply_metadata(self, message_id, metadata, context).await
        })
    }

    fn set_delivery_state<'a>(
        &'a self,
        message_id: &'a str,
        delivery_state: &'a str,
        observed_at: DateTime<Utc>,
        context: ProviderMessageProjectionObservationContext<'a>,
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
            ProviderChannelMessageStore::set_delivery_state(
                self,
                message_id,
                delivery_state,
                observed_at,
                context,
            )
            .await
        })
    }

    fn apply_content_update<'a>(
        &'a self,
        message_id: &'a str,
        body_text: &'a str,
        metadata: &'a Value,
        observed_at: DateTime<Utc>,
        context: ProviderMessageProjectionObservationContext<'a>,
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
            ProviderChannelMessageStore::apply_content_update(
                self,
                message_id,
                body_text,
                metadata,
                observed_at,
                context,
            )
            .await
        })
    }

    fn apply_pinned_state<'a>(
        &'a self,
        message_id: &'a str,
        is_pinned: bool,
        observed_at: DateTime<Utc>,
        context: ProviderMessageProjectionObservationContext<'a>,
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
            ProviderChannelMessageStore::apply_pinned_state(
                self,
                message_id,
                is_pinned,
                observed_at,
                context,
            )
            .await
        })
    }

    fn update_attachment_download_state<'a>(
        &'a self,
        update: ProviderAttachmentDownloadStateUpdate<'a>,
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
            ProviderChannelMessageStore::update_attachment_download_state(self, update).await
        })
    }
}
