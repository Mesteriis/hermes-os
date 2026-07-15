use super::*;

impl ProviderChannelMessageStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub(super) fn clone_pool(&self) -> PgPool {
        self.pool.clone()
    }

    pub async fn message_by_id(
        &self,
        message_id: &str,
        channel_kinds: &[&str],
    ) -> Result<Option<ProviderChannelMessage>, ProviderCommunicationMessagePortError> {
        let row = sqlx::query(
            r#"
            SELECT
                message_id,
                raw_record_id,
                account_id,
                provider_record_id,
                subject,
                sender,
                body_text,
                occurred_at,
                projected_at,
                channel_kind,
                conversation_id,
                sender_display_name,
                delivery_state,
                message_metadata
            FROM communication_messages
            WHERE message_id = $1
              AND channel_kind = ANY($2)
            "#,
        )
        .bind(message_id.trim())
        .bind(channel_kinds)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_provider_channel_message).transpose()
    }

    pub async fn message_by_provider_record_id(
        &self,
        account_id: &str,
        provider_record_id: &str,
        channel_kinds: &[&str],
    ) -> Result<Option<ProviderChannelMessage>, ProviderCommunicationMessagePortError> {
        let row = sqlx::query(
            r#"
            SELECT
                message_id,
                raw_record_id,
                account_id,
                provider_record_id,
                subject,
                sender,
                body_text,
                occurred_at,
                projected_at,
                channel_kind,
                conversation_id,
                sender_display_name,
                delivery_state,
                message_metadata
            FROM communication_messages
            WHERE account_id = $1
              AND provider_record_id = $2
              AND channel_kind = ANY($3)
            "#,
        )
        .bind(account_id.trim())
        .bind(provider_record_id.trim())
        .bind(channel_kinds)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_provider_channel_message).transpose()
    }

    pub async fn recent_messages(
        &self,
        account_id: Option<&str>,
        conversation_id: Option<&str>,
        channel_kinds: &[&str],
        limit: i64,
    ) -> Result<Vec<ProviderChannelMessage>, ProviderCommunicationMessagePortError> {
        let account_id = account_id.map(str::trim).filter(|value| !value.is_empty());
        let conversation_id = conversation_id
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let rows = sqlx::query(
            r#"
            SELECT
                message_id,
                raw_record_id,
                account_id,
                provider_record_id,
                subject,
                sender,
                body_text,
                occurred_at,
                projected_at,
                channel_kind,
                conversation_id,
                sender_display_name,
                delivery_state,
                message_metadata
            FROM communication_messages message
            WHERE message.channel_kind = ANY($1)
              AND ($2::text IS NULL OR message.account_id = $2)
              AND (
                  $3::text IS NULL
                  OR message.conversation_id = $3
                  OR EXISTS (
                      SELECT 1
                      FROM communication_conversations conversation
                      WHERE conversation.conversation_id = message.conversation_id
                        AND conversation.account_id = message.account_id
                        AND conversation.provider_conversation_id = $3
                        AND conversation.channel_kind = ANY($1)
                  )
              )
            ORDER BY COALESCE(occurred_at, projected_at) DESC, message_id ASC
            LIMIT $4
            "#,
        )
        .bind(channel_kinds)
        .bind(account_id)
        .bind(conversation_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(row_to_provider_channel_message)
            .collect()
    }

    pub async fn messages_by_ids(
        &self,
        message_ids: &[String],
        channel_kinds: &[&str],
    ) -> Result<Vec<ProviderChannelMessage>, ProviderCommunicationMessagePortError> {
        if message_ids.is_empty() {
            return Ok(vec![]);
        }
        let rows = sqlx::query(
            r#"
            SELECT
                message_id,
                raw_record_id,
                account_id,
                provider_record_id,
                subject,
                sender,
                body_text,
                occurred_at,
                projected_at,
                channel_kind,
                conversation_id,
                sender_display_name,
                delivery_state,
                message_metadata
            FROM communication_messages
            WHERE message_id = ANY($1)
              AND channel_kind = ANY($2)
            ORDER BY COALESCE(occurred_at, projected_at) DESC, message_id ASC
            "#,
        )
        .bind(message_ids)
        .bind(channel_kinds)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(row_to_provider_channel_message)
            .collect()
    }

    pub async fn search_messages(
        &self,
        account_id: Option<&str>,
        conversation_id: Option<&str>,
        query: &str,
        channel_kinds: &[&str],
        limit: i64,
    ) -> Result<Vec<ProviderChannelMessage>, ProviderCommunicationMessagePortError> {
        let like_pattern = format!("%{query}%");
        let account_id = account_id.map(str::trim).filter(|value| !value.is_empty());
        let conversation_id = conversation_id
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let rows = sqlx::query(
            r#"
            SELECT
                message_id,
                raw_record_id,
                account_id,
                provider_record_id,
                subject,
                sender,
                body_text,
                occurred_at,
                projected_at,
                channel_kind,
                conversation_id,
                sender_display_name,
                delivery_state,
                message_metadata
            FROM communication_messages
            WHERE channel_kind = ANY($1)
              AND body_text ILIKE $2
              AND ($3::text IS NULL OR account_id = $3)
              AND ($4::text IS NULL OR conversation_id = $4)
            ORDER BY COALESCE(occurred_at, projected_at) DESC
            LIMIT $5
            "#,
        )
        .bind(channel_kinds)
        .bind(&like_pattern)
        .bind(account_id)
        .bind(conversation_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(row_to_provider_channel_message)
            .collect()
    }

    pub async fn pinned_messages(
        &self,
        account_id: &str,
        conversation_id: &str,
        channel_kinds: &[&str],
        limit: i64,
    ) -> Result<Vec<ProviderChannelMessage>, ProviderCommunicationMessagePortError> {
        let rows = sqlx::query(
            r#"
            SELECT
                message_id,
                raw_record_id,
                account_id,
                provider_record_id,
                subject,
                sender,
                body_text,
                occurred_at,
                projected_at,
                channel_kind,
                conversation_id,
                sender_display_name,
                delivery_state,
                message_metadata
            FROM communication_messages
            WHERE channel_kind = ANY($1)
              AND account_id = $2
              AND conversation_id = $3
              AND (
                COALESCE(message_metadata->>'is_pinned', 'false') = 'true'
                OR COALESCE(message_metadata->>'pinned', 'false') = 'true'
              )
            ORDER BY COALESCE(occurred_at, projected_at) DESC
            LIMIT $4
            "#,
        )
        .bind(channel_kinds)
        .bind(account_id)
        .bind(conversation_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(row_to_provider_channel_message)
            .collect()
    }

    pub async fn body_text(
        &self,
        message_id: &str,
    ) -> Result<Option<String>, ProviderCommunicationMessagePortError> {
        Ok(sqlx::query_scalar::<_, Option<String>>(
            "SELECT body_text FROM communication_messages WHERE message_id = $1",
        )
        .bind(message_id)
        .fetch_optional(&self.pool)
        .await?
        .flatten())
    }

    pub async fn message_ids_by_metadata_string(
        &self,
        metadata_key: &str,
        metadata_value: &str,
        channel_kinds: &[&str],
        limit: i64,
    ) -> Result<Vec<String>, ProviderCommunicationMessagePortError> {
        let rows: Vec<(String,)> = sqlx::query_as(
            r#"
            SELECT message_id
            FROM communication_messages
            WHERE message_metadata ->> $1 = $2
              AND channel_kind = ANY($3)
            ORDER BY COALESCE(occurred_at, projected_at) DESC NULLS LAST, message_id ASC
            LIMIT $4
            "#,
        )
        .bind(metadata_key)
        .bind(metadata_value)
        .bind(channel_kinds)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(id,)| id).collect())
    }

    pub async fn message_id_by_provider_record_id(
        &self,
        account_id: &str,
        provider_record_id: &str,
        channel_kinds: &[&str],
    ) -> Result<Option<String>, ProviderCommunicationMessagePortError> {
        sqlx::query_scalar(
            r#"
            SELECT message_id
            FROM communication_messages
            WHERE account_id = $1
              AND provider_record_id = $2
              AND channel_kind = ANY($3)
            LIMIT 1
            "#,
        )
        .bind(account_id)
        .bind(provider_record_id)
        .bind(channel_kinds)
        .fetch_optional(&self.pool)
        .await
        .map_err(ProviderCommunicationMessagePortError::from)
    }

    pub async fn reference_summaries(
        &self,
        message_ids: &[String],
    ) -> Result<Vec<ProviderMessageReferenceSummary>, ProviderCommunicationMessagePortError> {
        if message_ids.is_empty() {
            return Ok(Vec::new());
        }
        sqlx::query_as::<
            _,
            (
                String,
                String,
                Option<String>,
                String,
                String,
                Option<String>,
                String,
                Option<DateTime<Utc>>,
            ),
        >(
            r#"
            SELECT
                message_id,
                provider_record_id,
                conversation_id,
                subject,
                sender,
                sender_display_name,
                body_text,
                occurred_at
            FROM communication_messages
            WHERE message_id = ANY($1)
            "#,
        )
        .bind(message_ids)
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(
            |(
                message_id,
                provider_record_id,
                conversation_id,
                subject,
                sender,
                sender_display_name,
                body_text,
                occurred_at,
            )| {
                Ok(ProviderMessageReferenceSummary {
                    message_id,
                    provider_record_id,
                    conversation_id,
                    subject,
                    sender,
                    sender_display_name,
                    body_text,
                    occurred_at,
                })
            },
        )
        .collect()
    }

    pub async fn heuristic_members(
        &self,
        account_id: &str,
        conversation_id: &str,
        query: Option<&str>,
        channel_kinds: &[&str],
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ProviderHeuristicMember>, ProviderCommunicationMessagePortError> {
        let pattern = query.map(|value| format!("%{value}%"));
        let rows = sqlx::query_as::<_, (String, Option<String>, i64, Option<DateTime<Utc>>)>(
            r#"
            SELECT
                sender,
                MAX(NULLIF(BTRIM(sender_display_name), '')) AS sender_display_name,
                COUNT(*)::bigint AS message_count,
                MAX(COALESCE(occurred_at, projected_at)) AS last_message_at
            FROM communication_messages
            WHERE account_id = $1
              AND conversation_id = $2
              AND channel_kind = ANY($3)
              AND (
                  $4::TEXT IS NULL
                  OR lower(sender) LIKE $4
                  OR lower(coalesce(sender_display_name, '')) LIKE $4
              )
            GROUP BY sender
            ORDER BY message_count DESC, last_message_at DESC NULLS LAST, sender ASC
            LIMIT $5 OFFSET $6
            "#,
        )
        .bind(account_id)
        .bind(conversation_id)
        .bind(channel_kinds)
        .bind(pattern.as_deref())
        .bind(limit)
        .bind(offset.max(0))
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(
                |(sender_id, sender_display_name, message_count, last_message_at)| {
                    ProviderHeuristicMember {
                        sender_id,
                        sender_display_name,
                        message_count,
                        last_message_at,
                    }
                },
            )
            .collect())
    }

    pub async fn attachment_anchor(
        &self,
        account_id: &str,
        conversation_id: &str,
        provider_record_id: &str,
        channel_kinds: &[&str],
    ) -> Result<Option<ProviderMessageAttachmentAnchor>, ProviderCommunicationMessagePortError>
    {
        let row = sqlx::query(
            r#"
            SELECT message_id, raw_record_id
            FROM communication_messages
            WHERE account_id = $1
              AND conversation_id = $2
              AND provider_record_id = $3
              AND channel_kind = ANY($4)
            ORDER BY COALESCE(occurred_at, projected_at) DESC, message_id ASC
            LIMIT 1
            "#,
        )
        .bind(account_id.trim())
        .bind(conversation_id.trim())
        .bind(provider_record_id.trim())
        .bind(channel_kinds)
        .fetch_optional(&self.pool)
        .await?;

        row.map(|row| {
            Ok(ProviderMessageAttachmentAnchor {
                message_id: row.try_get("message_id")?,
                raw_record_id: row.try_get("raw_record_id")?,
            })
        })
        .transpose()
    }

    pub async fn unread_counts(
        &self,
        account_id: &str,
        conversation_id: &str,
        channel_kinds: &[&str],
        last_read_at: Option<DateTime<Utc>>,
    ) -> Result<(i64, i64), ProviderCommunicationMessagePortError> {
        let unread_count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)::bigint
            FROM communication_messages
            WHERE account_id = $1
              AND conversation_id = $2
              AND channel_kind = ANY($3)
              AND delivery_state = 'received'
              AND ($4::timestamptz IS NULL OR COALESCE(occurred_at, projected_at) > $4)
            "#,
        )
        .bind(account_id)
        .bind(conversation_id)
        .bind(channel_kinds)
        .bind(last_read_at)
        .fetch_one(&self.pool)
        .await?;
        let mention_count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COALESCE(SUM(
                CASE
                    WHEN jsonb_typeof(message_metadata->'mention_count') = 'number'
                        THEN (message_metadata->>'mention_count')::bigint
                    ELSE 0
                END
            ), 0)::bigint
            FROM communication_messages
            WHERE account_id = $1
              AND conversation_id = $2
              AND channel_kind = ANY($3)
              AND delivery_state = 'received'
              AND ($4::timestamptz IS NULL OR COALESCE(occurred_at, projected_at) > $4)
            "#,
        )
        .bind(account_id)
        .bind(conversation_id)
        .bind(channel_kinds)
        .bind(last_read_at)
        .fetch_one(&self.pool)
        .await?;

        Ok((unread_count, mention_count))
    }

    pub async fn apply_metadata(
        &self,
        message_id: &str,
        metadata: &Value,
        context: ProviderMessageProjectionObservationContext<'_>,
    ) -> Result<Option<ProviderChannelMessage>, ProviderCommunicationMessagePortError> {
        if !metadata.is_object() {
            return Err(ProviderCommunicationMessagePortError::InvalidRequest(
                "provider message metadata must be a JSON object".to_owned(),
            ));
        }
        let Some(current) = self
            .message_by_id(message_id, context.channel_kinds)
            .await?
        else {
            return Ok(None);
        };
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            UPDATE communication_messages
            SET message_metadata = $2,
                projected_at = now()
            WHERE message_id = $1
            RETURNING
                message_id,
                raw_record_id,
                account_id,
                provider_record_id,
                subject,
                sender,
                body_text,
                occurred_at,
                projected_at,
                channel_kind,
                conversation_id,
                sender_display_name,
                delivery_state,
                message_metadata
            "#,
        )
        .bind(message_id.trim())
        .bind(metadata)
        .fetch_optional(&mut *transaction)
        .await?;
        let Some(row) = row else {
            return Ok(None);
        };
        let updated = row_to_provider_channel_message(row)?;
        capture_projection_observation_in_transaction(
            &mut transaction,
            &updated,
            updated.projected_at,
            context.relationship_kind,
            json!({
                "message_id": updated.message_id,
                "account_id": updated.account_id,
                "provider_message_id": updated.provider_record_id,
                "provider_chat_id": updated.conversation_id,
                "previous_metadata": current.message_metadata,
                "message_metadata": updated.message_metadata,
            }),
            context.actor,
        )
        .await?;
        transaction.commit().await?;
        Ok(Some(updated))
    }

    pub async fn set_delivery_state(
        &self,
        message_id: &str,
        delivery_state: &str,
        observed_at: DateTime<Utc>,
        context: ProviderMessageProjectionObservationContext<'_>,
    ) -> Result<Option<ProviderChannelMessage>, ProviderCommunicationMessagePortError> {
        let Some(current) = self
            .message_by_id(message_id, context.channel_kinds)
            .await?
        else {
            return Ok(None);
        };
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            UPDATE communication_messages
            SET delivery_state = $2,
                projected_at = $3
            WHERE message_id = $1
            RETURNING
                message_id,
                raw_record_id,
                account_id,
                provider_record_id,
                subject,
                sender,
                body_text,
                occurred_at,
                projected_at,
                channel_kind,
                conversation_id,
                sender_display_name,
                delivery_state,
                message_metadata
            "#,
        )
        .bind(message_id.trim())
        .bind(delivery_state.trim())
        .bind(observed_at)
        .fetch_optional(&mut *transaction)
        .await?;
        let Some(row) = row else {
            return Ok(None);
        };
        let updated = row_to_provider_channel_message(row)?;
        capture_projection_observation_in_transaction(
            &mut transaction,
            &updated,
            observed_at,
            context.relationship_kind,
            json!({
                "message_id": updated.message_id,
                "account_id": updated.account_id,
                "provider_message_id": updated.provider_record_id,
                "provider_chat_id": updated.conversation_id,
                "previous_delivery_state": current.delivery_state,
                "delivery_state": updated.delivery_state,
            }),
            context.actor,
        )
        .await?;
        transaction.commit().await?;
        Ok(Some(updated))
    }

    pub async fn rebind_provider_record_id(
        &self,
        message_id: &str,
        provider_record_id: &str,
        observed_at: DateTime<Utc>,
        context: ProviderMessageProjectionObservationContext<'_>,
    ) -> Result<Option<ProviderChannelMessage>, ProviderCommunicationMessagePortError> {
        let provider_record_id = provider_record_id.trim();
        if provider_record_id.is_empty() {
            return Err(ProviderCommunicationMessagePortError::InvalidRequest(
                "provider record id must not be empty".to_owned(),
            ));
        }

        let mut transaction = self.pool.begin().await?;
        let current = sqlx::query(
            r#"
            SELECT
                message_id, raw_record_id, account_id, provider_record_id, subject, sender,
                body_text, occurred_at, projected_at, channel_kind, conversation_id,
                sender_display_name, delivery_state, message_metadata
            FROM communication_messages
            WHERE message_id = $1
              AND channel_kind = ANY($2)
            FOR UPDATE
            "#,
        )
        .bind(message_id.trim())
        .bind(context.channel_kinds)
        .fetch_optional(&mut *transaction)
        .await?;
        let Some(current) = current.map(row_to_provider_channel_message).transpose()? else {
            return Ok(None);
        };

        // A replay of the same accepted provider event must retain the original
        // temporary locator evidence rather than treating the final id as prior state.
        if current.provider_record_id == provider_record_id && current.delivery_state == "sent" {
            transaction.commit().await?;
            return Ok(Some(current));
        }

        let conflicting_message_id = sqlx::query_scalar::<_, String>(
            r#"
            SELECT message_id
            FROM communication_messages
            WHERE account_id = $1
              AND provider_record_id = $2
              AND message_id <> $3
              AND channel_kind = ANY($4)
            FOR UPDATE
            "#,
        )
        .bind(&current.account_id)
        .bind(provider_record_id)
        .bind(&current.message_id)
        .bind(context.channel_kinds)
        .fetch_optional(&mut *transaction)
        .await?;
        if let Some(conflicting_message_id) = conflicting_message_id {
            return Err(ProviderCommunicationMessagePortError::InvalidRequest(
                format!(
                    "provider record `{provider_record_id}` is already mapped to message `{conflicting_message_id}`"
                ),
            ));
        }

        let mut metadata = current
            .message_metadata
            .as_object()
            .cloned()
            .ok_or_else(|| {
                ProviderCommunicationMessagePortError::InvalidRequest(
                    "provider message metadata must be a JSON object".to_owned(),
                )
            })?;
        metadata.insert(
            "previous_provider_record_id".to_owned(),
            json!(current.provider_record_id),
        );
        metadata.insert(
            "provider_identity_source".to_owned(),
            json!(context.relationship_kind),
        );
        let metadata = Value::Object(metadata);

        let row = sqlx::query(
            r#"
            UPDATE communication_messages
            SET provider_record_id = $2,
                delivery_state = 'sent',
                message_metadata = $3,
                projected_at = $4
            WHERE message_id = $1
            RETURNING
                message_id, raw_record_id, account_id, provider_record_id, subject, sender,
                body_text, occurred_at, projected_at, channel_kind, conversation_id,
                sender_display_name, delivery_state, message_metadata
            "#,
        )
        .bind(&current.message_id)
        .bind(provider_record_id)
        .bind(&metadata)
        .bind(observed_at)
        .fetch_one(&mut *transaction)
        .await?;
        let updated = row_to_provider_channel_message(row)?;
        capture_projection_observation_in_transaction(
            &mut transaction,
            &updated,
            observed_at,
            context.relationship_kind,
            json!({
                "message_id": updated.message_id,
                "account_id": updated.account_id,
                "previous_provider_message_id": current.provider_record_id,
                "provider_message_id": updated.provider_record_id,
                "provider_chat_id": updated.conversation_id,
                "previous_delivery_state": current.delivery_state,
                "delivery_state": updated.delivery_state,
            }),
            context.actor,
        )
        .await?;
        transaction.commit().await?;
        Ok(Some(updated))
    }

    pub async fn apply_content_update(
        &self,
        message_id: &str,
        body_text: &str,
        metadata: &Value,
        observed_at: DateTime<Utc>,
        context: ProviderMessageProjectionObservationContext<'_>,
    ) -> Result<Option<ProviderChannelMessage>, ProviderCommunicationMessagePortError> {
        if !metadata.is_object() {
            return Err(ProviderCommunicationMessagePortError::InvalidRequest(
                "provider message metadata must be a JSON object".to_owned(),
            ));
        }
        let Some(current) = self
            .message_by_id(message_id, context.channel_kinds)
            .await?
        else {
            return Ok(None);
        };
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            UPDATE communication_messages
            SET body_text = $2,
                message_metadata = $3,
                projected_at = $4
            WHERE message_id = $1
            RETURNING
                message_id,
                raw_record_id,
                account_id,
                provider_record_id,
                subject,
                sender,
                body_text,
                occurred_at,
                projected_at,
                channel_kind,
                conversation_id,
                sender_display_name,
                delivery_state,
                message_metadata
            "#,
        )
        .bind(message_id.trim())
        .bind(body_text)
        .bind(metadata)
        .bind(observed_at)
        .fetch_optional(&mut *transaction)
        .await?;
        let Some(row) = row else {
            return Ok(None);
        };
        let updated = row_to_provider_channel_message(row)?;
        capture_projection_observation_in_transaction(
            &mut transaction,
            &updated,
            observed_at,
            context.relationship_kind,
            json!({
                "message_id": updated.message_id,
                "account_id": updated.account_id,
                "provider_message_id": updated.provider_record_id,
                "provider_chat_id": updated.conversation_id,
                "previous_body_text": current.body_text,
                "body_text": updated.body_text,
                "previous_metadata": current.message_metadata,
                "message_metadata": updated.message_metadata,
            }),
            context.actor,
        )
        .await?;
        transaction.commit().await?;
        Ok(Some(updated))
    }

    pub async fn apply_pinned_state(
        &self,
        message_id: &str,
        is_pinned: bool,
        observed_at: DateTime<Utc>,
        context: ProviderMessageProjectionObservationContext<'_>,
    ) -> Result<Option<ProviderChannelMessage>, ProviderCommunicationMessagePortError> {
        let Some(current) = self
            .message_by_id(message_id, context.channel_kinds)
            .await?
        else {
            return Ok(None);
        };
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            UPDATE communication_messages
            SET message_metadata = jsonb_set(
                    jsonb_set(
                        COALESCE(message_metadata, '{}'::jsonb),
                        '{pinned}',
                        to_jsonb($2::boolean),
                        true
                    ),
                    '{is_pinned}',
                    to_jsonb($2::boolean),
                    true
                ),
                projected_at = $3
            WHERE message_id = $1
            RETURNING
                message_id,
                raw_record_id,
                account_id,
                provider_record_id,
                subject,
                sender,
                body_text,
                occurred_at,
                projected_at,
                channel_kind,
                conversation_id,
                sender_display_name,
                delivery_state,
                message_metadata
            "#,
        )
        .bind(message_id.trim())
        .bind(is_pinned)
        .bind(observed_at)
        .fetch_optional(&mut *transaction)
        .await?;
        let Some(row) = row else {
            return Ok(None);
        };
        let updated = row_to_provider_channel_message(row)?;
        capture_projection_observation_in_transaction(
            &mut transaction,
            &updated,
            observed_at,
            context.relationship_kind,
            json!({
                "message_id": updated.message_id,
                "account_id": updated.account_id,
                "provider_message_id": updated.provider_record_id,
                "provider_chat_id": updated.conversation_id,
                "previous_is_pinned": current.message_metadata.get("is_pinned").cloned().unwrap_or(Value::Bool(false)),
                "is_pinned": is_pinned,
                "message_metadata": updated.message_metadata,
            }),
            context.actor,
        )
        .await?;
        transaction.commit().await?;
        Ok(Some(updated))
    }

    pub async fn update_attachment_download_state(
        &self,
        update: ProviderAttachmentDownloadStateUpdate<'_>,
    ) -> Result<Option<ProviderChannelMessage>, ProviderCommunicationMessagePortError> {
        let Some(current) = self
            .message_by_id(update.message_id, update.context.channel_kinds)
            .await?
        else {
            return Ok(None);
        };
        let mut metadata_object = current
            .message_metadata
            .as_object()
            .cloned()
            .unwrap_or_default();
        let attachments = metadata_object
            .entry("attachments".to_owned())
            .or_insert_with(|| Value::Array(Vec::new()));
        let attachment_array = attachments.as_array_mut().ok_or_else(|| {
            ProviderCommunicationMessagePortError::InvalidRequest(
                "provider attachment metadata must be an array".to_owned(),
            )
        })?;

        let mut updated_attachment = false;
        for attachment in attachment_array.iter_mut() {
            let Some(object) = attachment.as_object_mut() else {
                continue;
            };
            let attachment_id_matches = object
                .get("provider_attachment_id")
                .or_else(|| object.get("attachment_id"))
                .and_then(Value::as_str)
                .map(|value| value == update.provider_attachment_id)
                .unwrap_or(false);
            let provider_file_matches = object
                .get("tdlib_file_id")
                .and_then(Value::as_i64)
                .map(|value| value == update.provider_file_id)
                .unwrap_or(false);
            if !attachment_id_matches && !provider_file_matches {
                continue;
            }

            object.insert(
                "provider_attachment_id".to_owned(),
                json!(update.provider_attachment_id.to_owned()),
            );
            if let Some(attachment_id) = update.communication_attachment_id {
                object.insert("attachment_id".to_owned(), json!(attachment_id));
            } else if !object.contains_key("attachment_id") {
                object.insert(
                    "attachment_id".to_owned(),
                    json!(update.provider_attachment_id.to_owned()),
                );
            }
            object.insert("tdlib_file_id".to_owned(), json!(update.provider_file_id));
            object.insert("download_state".to_owned(), json!(update.download_state));
            object.insert("content_type".to_owned(), json!(update.content_type));
            if let Some(path) = update.local_path {
                object.insert("local_path".to_owned(), json!(path));
            }
            if let Some(size) = update.size_bytes {
                object.insert("size".to_owned(), json!(size));
            }
            if let Some(name) = update.filename {
                object.insert("filename".to_owned(), json!(name));
            }
            updated_attachment = true;
        }

        if !updated_attachment {
            let mut attachment = json!({
                "provider_attachment_id": update.provider_attachment_id,
                "attachment_type": "file",
                "content_type": update.content_type,
                "tdlib_file_id": update.provider_file_id,
                "download_state": update.download_state,
                "local_path": update.local_path,
                "size": update.size_bytes,
                "filename": update.filename,
            });
            if let (Some(object), Some(attachment_id)) = (
                attachment.as_object_mut(),
                update.communication_attachment_id,
            ) {
                object.insert("attachment_id".to_owned(), json!(attachment_id));
            }
            attachment_array.push(attachment);
        }

        let updated_metadata = Value::Object(metadata_object);
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            UPDATE communication_messages
            SET message_metadata = $2::jsonb,
                projected_at = $3
            WHERE message_id = $1
            RETURNING
                message_id,
                raw_record_id,
                account_id,
                provider_record_id,
                subject,
                sender,
                body_text,
                occurred_at,
                projected_at,
                channel_kind,
                conversation_id,
                sender_display_name,
                delivery_state,
                message_metadata
            "#,
        )
        .bind(update.message_id.trim())
        .bind(&updated_metadata)
        .bind(update.observed_at)
        .fetch_optional(&mut *transaction)
        .await?;

        let Some(row) = row else {
            return Ok(None);
        };
        let updated = row_to_provider_channel_message(row)?;
        capture_projection_observation_in_transaction(
            &mut transaction,
            &updated,
            update.observed_at,
            update.context.relationship_kind,
            json!({
                "message_id": updated.message_id,
                "account_id": updated.account_id,
                "provider_message_id": updated.provider_record_id,
                "provider_chat_id": updated.conversation_id,
                "provider_attachment_id": update.provider_attachment_id,
                "communication_attachment_id": update.communication_attachment_id,
                "tdlib_file_id": update.provider_file_id,
                "download_state": update.download_state,
                "local_path": update.local_path,
                "size_bytes": update.size_bytes,
                "content_type": update.content_type,
                "filename": update.filename,
                "previous_metadata": current.message_metadata,
                "message_metadata": updated.message_metadata,
            }),
            update.context.actor,
        )
        .await?;
        transaction.commit().await?;

        Ok(Some(updated))
    }
}
