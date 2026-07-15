use serde_json::{Value, json};
use sqlx::PgPool;

pub(crate) async fn ensure_whatsapp_channel(
    pool: &PgPool,
    account_id: &str,
    channel_id: &str,
    channel_kind: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO communication_channels (
            channel_id, account_id, channel_kind, provider_kind, display_name,
            runtime_state, config, metadata, created_at, updated_at
        )
        SELECT
            $2, account_id, $3, provider_kind, display_name, 'fixture', config,
            jsonb_build_object('source_table', 'communication_provider_accounts'),
            created_at, updated_at
        FROM communication_provider_accounts
        WHERE account_id = $1
        ON CONFLICT (channel_id)
        DO UPDATE SET
            display_name = EXCLUDED.display_name,
            provider_kind = EXCLUDED.provider_kind,
            runtime_state = EXCLUDED.runtime_state,
            config = EXCLUDED.config,
            updated_at = EXCLUDED.updated_at
        "#,
    )
    .bind(account_id)
    .bind(channel_id)
    .bind(channel_kind)
    .execute(pool)
    .await?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn upsert_whatsapp_conversation(
    pool: &PgPool,
    account_id: &str,
    conversation_id: &str,
    channel_id: &str,
    provider_chat_id: &str,
    chat_title: &str,
    chat_kind: &str,
    provider_kind: &str,
    channel_kind: &str,
    is_archived: Option<bool>,
    is_pinned: Option<bool>,
    is_muted: Option<bool>,
    is_unread: Option<bool>,
    unread_count: Option<i64>,
    participant_count: Option<i64>,
    community_parent_chat_id: Option<&str>,
    community_parent_title: Option<&str>,
    invite_link: Option<&str>,
    is_community_root: Option<bool>,
    is_broadcast: Option<bool>,
    is_newsletter: Option<bool>,
    avatar_metadata: &Value,
    provider_labels: &[String],
    observed_at: chrono::DateTime<chrono::Utc>,
    raw_record_id: &str,
    accepted_event_id: &str,
) -> Result<String, sqlx::Error> {
    let mut metadata = json!({
        "provider": provider_kind,
        "chat_kind": chat_kind,
        "raw_record_id": raw_record_id,
        "accepted_signal_event_id": accepted_event_id,
    });
    macro_rules! add {
        ($name:literal, $value:expr) => {
            if let Some(value) = $value {
                metadata[$name] = json!(value);
            }
        };
    }
    add!("is_archived", is_archived);
    add!("is_pinned", is_pinned);
    add!("is_muted", is_muted);
    add!("is_unread", is_unread);
    add!("community_parent_chat_id", community_parent_chat_id);
    add!("community_parent_title", community_parent_title);
    add!("invite_link", invite_link);
    add!("is_community_root", is_community_root);
    add!("is_broadcast", is_broadcast);
    add!("is_newsletter", is_newsletter);
    add!("unread_count", unread_count);
    add!("participant_count", participant_count);
    if avatar_metadata.is_object() && avatar_metadata != &json!({}) {
        metadata["avatar_metadata"] = avatar_metadata.clone();
    }
    if !provider_labels.is_empty() {
        metadata["provider_labels"] = json!(provider_labels);
    }
    sqlx::query(r#"
        INSERT INTO communication_conversations (
            conversation_id, account_id, channel_id, channel_kind,
            provider_conversation_id, title, last_message_at, metadata, created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, now(), now())
        ON CONFLICT (conversation_id) DO UPDATE SET
            channel_id = EXCLUDED.channel_id, title = EXCLUDED.title,
            last_message_at = GREATEST(COALESCE(communication_conversations.last_message_at, EXCLUDED.last_message_at), EXCLUDED.last_message_at),
            metadata = communication_conversations.metadata || EXCLUDED.metadata, updated_at = now()
    "#)
    .bind(conversation_id).bind(account_id).bind(channel_id).bind(channel_kind)
    .bind(provider_chat_id).bind(chat_title).bind(observed_at).bind(metadata)
    .execute(pool).await?;
    Ok(conversation_id.to_owned())
}

pub(crate) struct WhatsappIdentityUpsert<'a> {
    pub identity_id: &'a str,
    pub account_id: &'a str,
    pub channel_id: &'a str,
    pub identity_kind: &'a str,
    pub provider_identity_id: &'a str,
    pub display_name: &'a str,
    pub address: Option<&'a str>,
    pub metadata: Value,
}

pub(crate) async fn upsert_whatsapp_identity(
    pool: &PgPool,
    input: WhatsappIdentityUpsert<'_>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO communication_identities (
            identity_id, account_id, channel_id, identity_kind, provider_identity_id,
            display_name, address, metadata, created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, now(), now())
        ON CONFLICT (account_id, identity_kind, provider_identity_id) DO UPDATE SET
            channel_id = EXCLUDED.channel_id,
            display_name = EXCLUDED.display_name,
            address = COALESCE(EXCLUDED.address, communication_identities.address),
            metadata = communication_identities.metadata || EXCLUDED.metadata,
            updated_at = now()
    "#,
    )
    .bind(input.identity_id)
    .bind(input.account_id)
    .bind(input.channel_id)
    .bind(input.identity_kind)
    .bind(input.provider_identity_id)
    .bind(input.display_name)
    .bind(input.address)
    .bind(input.metadata)
    .execute(pool)
    .await?;
    Ok(())
}

pub(crate) struct WhatsappConversationParticipantUpsert<'a> {
    pub participant_id: &'a str,
    pub conversation_id: &'a str,
    pub identity_id: &'a str,
    pub role: &'a str,
    pub display_name: &'a str,
    pub address: Option<&'a str>,
    pub metadata: Value,
}

pub(crate) async fn upsert_whatsapp_conversation_participant(
    pool: &PgPool,
    input: WhatsappConversationParticipantUpsert<'_>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO communication_conversation_participants (
            participant_id, conversation_id, identity_id, persona_id, role,
            display_name, address, metadata, created_at, updated_at
        ) VALUES ($1, $2, $3, NULL, $4, $5, $6, $7, now(), now())
        ON CONFLICT (participant_id) DO UPDATE SET
            identity_id = EXCLUDED.identity_id,
            role = EXCLUDED.role,
            display_name = EXCLUDED.display_name,
            address = COALESCE(EXCLUDED.address, communication_conversation_participants.address),
            metadata = communication_conversation_participants.metadata || EXCLUDED.metadata,
            updated_at = now()
    "#,
    )
    .bind(input.participant_id)
    .bind(input.conversation_id)
    .bind(input.identity_id)
    .bind(input.role)
    .bind(input.display_name)
    .bind(input.address)
    .bind(input.metadata)
    .execute(pool)
    .await?;
    Ok(())
}

pub(crate) async fn upsert_whatsapp_status_feed_conversation(
    pool: &PgPool,
    conversation_id: &str,
    account_id: &str,
    channel_id: &str,
    channel_kind: &str,
    occurred_at: chrono::DateTime<chrono::Utc>,
    metadata: Value,
) -> Result<(), sqlx::Error> {
    sqlx::query(r#"
        INSERT INTO communication_conversations (
            conversation_id, account_id, channel_id, channel_kind,
            provider_conversation_id, title, last_message_at, metadata, created_at, updated_at
        ) VALUES ($1, $2, $3, $4, 'status-feed', 'WhatsApp Status', $5, $6, now(), now())
        ON CONFLICT (conversation_id) DO UPDATE SET
            channel_id = EXCLUDED.channel_id,
            title = EXCLUDED.title,
            last_message_at = GREATEST(COALESCE(communication_conversations.last_message_at, EXCLUDED.last_message_at), EXCLUDED.last_message_at),
            metadata = communication_conversations.metadata || EXCLUDED.metadata,
            updated_at = now()
    "#)
    .bind(conversation_id).bind(account_id).bind(channel_id).bind(channel_kind)
    .bind(occurred_at).bind(metadata).execute(pool).await?;
    Ok(())
}
