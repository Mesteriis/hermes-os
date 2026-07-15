use super::*;

pub(super) async fn ensure_canonical_account_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    account_id: Option<&str>,
) -> Result<(), CommunicationFolderError> {
    let Some(account_id) = account_id else {
        return Ok(());
    };

    sqlx::query(
        r#"
        INSERT INTO communication_accounts (
            account_id, provider_kind, display_name, external_account_id, config, metadata, created_at, updated_at
        )
        SELECT
            account_id,
            provider_kind,
            display_name,
            external_account_id,
            config,
            '{}'::jsonb,
            created_at,
            updated_at
        FROM communication_provider_accounts
        WHERE account_id = $1
        ON CONFLICT (account_id) DO NOTHING
        "#,
    )
    .bind(account_id)
    .execute(&mut **transaction)
    .await?;

    Ok(())
}

pub(super) async fn insert_folder(
    transaction: &mut Transaction<'_, Postgres>,
    input: &NormalizedCommunicationFolderInput,
) -> Result<CommunicationFolder, CommunicationFolderError> {
    let row = sqlx::query(
        r#"
        WITH inserted AS (
            INSERT INTO communication_folders (
                folder_id, account_id, name, description, color, sort_order
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING folder_id, account_id, name, description, color, sort_order, created_at, updated_at
        )
        SELECT
            f.folder_id,
            f.account_id,
            f.name,
            f.description,
            f.color,
            f.sort_order,
            count(fm.message_id)::BIGINT AS message_count,
            f.created_at,
            f.updated_at
        FROM inserted f
        LEFT JOIN communication_folder_messages fm ON fm.folder_id = f.folder_id
        GROUP BY f.folder_id, f.account_id, f.name, f.description, f.color, f.sort_order, f.created_at, f.updated_at
        "#,
    )
    .bind(&input.folder_id)
    .bind(input.account_id.as_deref())
    .bind(&input.name)
    .bind(input.description.as_deref())
    .bind(input.color.as_deref())
    .bind(input.sort_order)
    .fetch_one(&mut **transaction)
    .await?;

    row_to_folder(row)
}

pub(super) async fn update_folder(
    transaction: &mut Transaction<'_, Postgres>,
    folder_id: &str,
    update: &NormalizedCommunicationFolderUpdate,
) -> Result<Option<CommunicationFolder>, CommunicationFolderError> {
    let row = sqlx::query(
        r#"
        WITH updated AS (
            UPDATE communication_folders
            SET account_id = COALESCE($2, account_id),
                name = COALESCE($3, name),
                description = COALESCE($4, description),
                color = COALESCE($5, color),
                sort_order = COALESCE($6, sort_order),
                updated_at = now()
            WHERE folder_id = $1
            RETURNING folder_id, account_id, name, description, color, sort_order, created_at, updated_at
        )
        SELECT
            f.folder_id,
            f.account_id,
            f.name,
            f.description,
            f.color,
            f.sort_order,
            count(fm.message_id)::BIGINT AS message_count,
            f.created_at,
            f.updated_at
        FROM updated f
        LEFT JOIN communication_folder_messages fm ON fm.folder_id = f.folder_id
        GROUP BY f.folder_id, f.account_id, f.name, f.description, f.color, f.sort_order, f.created_at, f.updated_at
        "#,
    )
    .bind(folder_id)
    .bind(update.account_id.as_deref())
    .bind(update.name.as_deref())
    .bind(update.description.as_deref())
    .bind(update.color.as_deref())
    .bind(update.sort_order)
    .fetch_optional(&mut **transaction)
    .await?;

    row.map(row_to_folder).transpose()
}

pub(super) async fn delete_folder(
    transaction: &mut Transaction<'_, Postgres>,
    folder_id: &str,
) -> Result<Option<CommunicationFolder>, CommunicationFolderError> {
    let row = sqlx::query(
        r#"
        WITH target AS (
            SELECT
                f.folder_id,
                f.account_id,
                f.name,
                f.description,
                f.color,
                f.sort_order,
                count(fm.message_id)::BIGINT AS message_count,
                f.created_at,
                f.updated_at
            FROM communication_folders f
            LEFT JOIN communication_folder_messages fm ON fm.folder_id = f.folder_id
            WHERE f.folder_id = $1
            GROUP BY f.folder_id, f.account_id, f.name, f.description, f.color, f.sort_order, f.created_at, f.updated_at
        ),
        deleted AS (
            DELETE FROM communication_folders WHERE folder_id = $1 RETURNING folder_id
        )
        SELECT target.*
        FROM target
        JOIN deleted ON deleted.folder_id = target.folder_id
        "#,
    )
    .bind(folder_id)
    .fetch_optional(&mut **transaction)
    .await?;

    row.map(row_to_folder).transpose()
}

pub(super) async fn folder_exists(
    transaction: &mut Transaction<'_, Postgres>,
    folder_id: &str,
) -> Result<bool, CommunicationFolderError> {
    Ok(sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM communication_folders WHERE folder_id = $1)",
    )
    .bind(folder_id)
    .fetch_one(&mut **transaction)
    .await?)
}

pub(super) async fn message_exists(
    transaction: &mut Transaction<'_, Postgres>,
    message_id: &str,
) -> Result<bool, CommunicationFolderError> {
    Ok(sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM communication_messages WHERE message_id = $1)",
    )
    .bind(message_id)
    .fetch_one(&mut **transaction)
    .await?)
}

pub(super) async fn upsert_folder_message(
    transaction: &mut Transaction<'_, Postgres>,
    folder_id: &str,
    message_id: &str,
    operation: &str,
) -> Result<(), CommunicationFolderError> {
    sqlx::query(
        r#"
        INSERT INTO communication_folder_messages (
            folder_id, message_id, added_at, last_operation, metadata
        )
        VALUES ($1, $2, now(), $3, '{"source":"local_user"}'::jsonb)
        ON CONFLICT (folder_id, message_id)
        DO UPDATE SET added_at = EXCLUDED.added_at,
                      last_operation = EXCLUDED.last_operation,
                      metadata = CASE
                          WHEN communication_folder_messages.metadata->>'source'
                              = 'provider_resource_mapping'
                          THEN EXCLUDED.metadata
                          ELSE communication_folder_messages.metadata
                      END
        "#,
    )
    .bind(folder_id)
    .bind(message_id)
    .bind(operation)
    .execute(&mut **transaction)
    .await?;

    Ok(())
}

pub(super) async fn load_folder_message(
    transaction: &mut Transaction<'_, Postgres>,
    folder_id: &str,
    message_id: &str,
) -> Result<FolderMessage, CommunicationFolderError> {
    let row = sqlx::query(
        r#"
        SELECT
            fm.folder_id,
            fm.message_id,
            fm.added_at,
            m.account_id,
            m.subject,
            m.sender,
            m.occurred_at,
            m.projected_at,
            m.workflow_state,
            m.local_state,
            count(a.attachment_id)::BIGINT AS attachment_count
        FROM communication_folder_messages fm
        JOIN communication_messages m ON m.message_id = fm.message_id
        LEFT JOIN communication_attachments a ON a.message_id = m.message_id
        WHERE fm.folder_id = $1 AND fm.message_id = $2
        GROUP BY
            fm.folder_id,
            fm.message_id,
            fm.added_at,
            m.account_id,
            m.subject,
            m.sender,
            m.occurred_at,
            m.projected_at,
            m.workflow_state,
            m.local_state
        "#,
    )
    .bind(folder_id)
    .bind(message_id)
    .fetch_one(&mut **transaction)
    .await?;

    row_to_folder_message(row)
}
