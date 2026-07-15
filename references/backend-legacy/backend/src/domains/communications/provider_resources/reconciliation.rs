use super::*;

pub(super) async fn lock_account(
    transaction: &mut Transaction<'_, Postgres>,
    account_id: &str,
) -> Result<(), MailProviderResourceError> {
    let exists = sqlx::query_scalar::<_, String>(
        "SELECT account_id FROM communication_accounts WHERE account_id = $1 FOR UPDATE",
    )
    .bind(account_id)
    .fetch_optional(&mut **transaction)
    .await?;
    if exists.is_none() {
        return Err(MailProviderResourceError::AccountNotFound(
            account_id.to_owned(),
        ));
    }
    Ok(())
}

pub(super) async fn clear_discovered_role(
    transaction: &mut Transaction<'_, Postgres>,
    account_id: &str,
    resource_kind: MailProviderResourceKind,
    role: MailProviderSemanticRole,
    except_mapping_id: Option<&str>,
) -> Result<(), MailProviderResourceError> {
    sqlx::query(
        r#"
        UPDATE communication_mail_provider_resources
        SET semantic_role = NULL, updated_at = now()
        WHERE account_id = $1
          AND resource_kind = $2
          AND semantic_role = $3
          AND mapping_source = 'discovered'
          AND ($4::text IS NULL OR mapping_id <> $4)
        "#,
    )
    .bind(account_id)
    .bind(resource_kind.as_str())
    .bind(role.as_str())
    .bind(except_mapping_id)
    .execute(&mut **transaction)
    .await?;
    Ok(())
}

pub(super) async fn reconcile_imap_sent_delivery_states(
    transaction: &mut Transaction<'_, Postgres>,
    account_id: &str,
) -> Result<(), MailProviderResourceError> {
    sqlx::query(
        r#"
        WITH reconciled AS (
            SELECT
                message.message_id,
                CASE
                    WHEN EXISTS (
                        SELECT 1
                        FROM communication_mail_provider_resources AS resource
                        WHERE resource.account_id = message.account_id
                          AND resource.resource_kind = 'folder'
                          AND resource.provider_resource_id = message.message_metadata->>'mailbox'
                          AND resource.semantic_role = 'sent'
                    )
                    THEN 'sent'
                    ELSE 'received'
                END AS delivery_state
            FROM communication_messages AS message
            WHERE message.account_id = $1
              AND message.channel_kind = 'email'
              AND message.message_metadata->>'transport' = 'imap'
        )
        UPDATE communication_messages AS message
        SET delivery_state = reconciled.delivery_state,
            projected_at = now()
        FROM reconciled
        WHERE message.message_id = reconciled.message_id
          AND message.delivery_state IS DISTINCT FROM reconciled.delivery_state
        "#,
    )
    .bind(account_id)
    .execute(&mut **transaction)
    .await?;
    Ok(())
}

pub(super) async fn reconcile_provider_folder_memberships_for_mapping(
    transaction: &mut Transaction<'_, Postgres>,
    resource: &MailProviderResource,
) -> Result<(), MailProviderResourceError> {
    // A mapping is the sole owner of its derived membership. Explicit local
    // folder actions use different metadata and remain untouched here.
    sqlx::query(
        r#"
        DELETE FROM communication_folder_messages
        WHERE metadata->>'source' = 'provider_resource_mapping'
          AND metadata->>'mapping_id' = $1
        "#,
    )
    .bind(&resource.mapping_id)
    .execute(&mut **transaction)
    .await?;

    let Some(local_folder_id) = resource.local_folder_id.as_deref() else {
        return Ok(());
    };

    let matching_messages_sql = match resource.resource_kind {
        MailProviderResourceKind::Label => {
            r#"
            INSERT INTO communication_folder_messages (
                folder_id, message_id, added_at, last_operation, metadata
            )
            SELECT
                $1,
                message.message_id,
                now(),
                'copy',
                jsonb_build_object(
                    'source', 'provider_resource_mapping',
                    'mapping_id', $2
                )
            FROM communication_messages AS message
            WHERE message.account_id = $3
              AND message.message_metadata->>'provider' = 'gmail'
              AND COALESCE(message.message_metadata->'label_ids', '[]'::jsonb) ? $4
            ON CONFLICT (folder_id, message_id) DO NOTHING
            "#
        }
        MailProviderResourceKind::Folder => {
            r#"
            INSERT INTO communication_folder_messages (
                folder_id, message_id, added_at, last_operation, metadata
            )
            SELECT
                $1,
                message.message_id,
                now(),
                'copy',
                jsonb_build_object(
                    'source', 'provider_resource_mapping',
                    'mapping_id', $2
                )
            FROM communication_messages AS message
            WHERE message.account_id = $3
              AND message.message_metadata->>'transport' = 'imap'
              AND message.message_metadata->>'mailbox' = $4
            ON CONFLICT (folder_id, message_id) DO NOTHING
            "#
        }
    };
    sqlx::query(matching_messages_sql)
        .bind(local_folder_id)
        .bind(&resource.mapping_id)
        .bind(&resource.account_id)
        .bind(&resource.provider_resource_id)
        .execute(&mut **transaction)
        .await?;
    Ok(())
}

pub(super) async fn validate_local_folder(
    transaction: &mut Transaction<'_, Postgres>,
    account_id: &str,
    local_folder_id: Option<&str>,
) -> Result<(), MailProviderResourceError> {
    let Some(local_folder_id) = local_folder_id else {
        return Ok(());
    };
    validate_non_empty("local_folder_id", local_folder_id)?;
    let belongs_to_account = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1 FROM communication_folders
            WHERE folder_id = $1 AND account_id = $2
        )
        "#,
    )
    .bind(local_folder_id.trim())
    .bind(account_id)
    .fetch_one(&mut **transaction)
    .await?;
    if !belongs_to_account {
        return Err(MailProviderResourceError::LocalFolderAccountMismatch(
            local_folder_id.trim().to_owned(),
        ));
    }
    Ok(())
}
