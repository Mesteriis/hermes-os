use sqlx::{Postgres, Row, Transaction};

use crate::{
    AcceptContactChangedForMailSyncOutcomeV1, AcceptContactChangedForMailSyncV1,
    CompleteContactMailSyncSourceOutcomeV1, CompleteContactMailSyncSourceV1,
    MailContactsSyncPersistenceErrorV1, MailContactsSyncPersistenceV1,
    MailContactsSyncReverseOperationV1,
    reverse_model::{validate_changed_input, validate_source_completion},
};

impl MailContactsSyncPersistenceV1 {
    pub async fn load_reverse_operation(
        &self,
        logical_owner_id: &str,
        operation_id: [u8; 16],
    ) -> Result<MailContactsSyncReverseOperationV1, MailContactsSyncPersistenceErrorV1> {
        if !crate::model::valid_identity(logical_owner_id)
            || operation_id.iter().all(|byte| *byte == 0)
        {
            return Err(MailContactsSyncPersistenceErrorV1::InvalidInput);
        }
        let row = sqlx::query(
            "SELECT configuration_instance_id, account_id, contact_id, contact_revision, state \
             FROM hermes_data.mail_contacts_sync_reverse_operations \
             WHERE logical_owner_id = $1 AND operation_id = $2",
        )
        .bind(logical_owner_id)
        .bind(operation_id.as_slice())
        .fetch_optional(&self.pool)
        .await
        .map_err(storage)?
        .ok_or(MailContactsSyncPersistenceErrorV1::NotFound)?;
        decode_operation(operation_id, &row)
    }

    pub async fn accept_contact_changed_for_mail_sync(
        &self,
        input: &AcceptContactChangedForMailSyncV1,
    ) -> Result<AcceptContactChangedForMailSyncOutcomeV1, MailContactsSyncPersistenceErrorV1> {
        validate_changed_input(input)?;
        let mut transaction = self.pool.begin().await.map_err(storage)?;
        let inserted = sqlx::query(
            "INSERT INTO hermes_data.mail_contacts_sync_reverse_inbox (logical_owner_id, \
             event_message_id, event_envelope_sha256, completed_at_unix_millis) \
             VALUES ($1,$2,$3,$4) ON CONFLICT DO NOTHING",
        )
        .bind(&input.logical_owner_id)
        .bind(input.event_message_id.as_slice())
        .bind(input.event_envelope_sha256.as_slice())
        .bind(input.occurred_at_unix_millis)
        .execute(&mut *transaction)
        .await
        .map_err(storage)?;
        if inserted.rows_affected() == 0 {
            validate_replay(&mut transaction, input).await?;
            transaction.commit().await.map_err(storage)?;
            return Ok(AcceptContactChangedForMailSyncOutcomeV1::Duplicate);
        }
        for operation in &input.operations {
            sqlx::query(
                "INSERT INTO hermes_data.mail_contacts_sync_reverse_operations \
                 (logical_owner_id, operation_id, source_event_message_id, \
                  configuration_instance_id, account_id, contact_id, contact_revision, state, \
                  source_command_message_id, created_at_unix_millis, updated_at_unix_millis) \
                 VALUES ($1,$2,$3,$4,$5,$6,$7,1,$8,$9,$9) ON CONFLICT DO NOTHING",
            )
            .bind(&input.logical_owner_id)
            .bind(operation.operation_id.as_slice())
            .bind(input.event_message_id.as_slice())
            .bind(&operation.configuration_instance_id)
            .bind(&operation.account_id)
            .bind(operation.contact_id.as_slice())
            .bind(
                i64::try_from(operation.contact_revision)
                    .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidInput)?,
            )
            .bind(operation.source_prepare_command.message_id.as_slice())
            .bind(input.occurred_at_unix_millis)
            .execute(&mut *transaction)
            .await
            .map_err(storage)?;
            super::repository::insert_outbox(
                &mut transaction,
                &input.logical_owner_id,
                &operation.source_prepare_command,
                input.occurred_at_unix_millis,
            )
            .await?;
        }
        transaction.commit().await.map_err(storage)?;
        Ok(AcceptContactChangedForMailSyncOutcomeV1::Applied {
            operations: u16::try_from(input.operations.len())
                .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidInput)?,
        })
    }

    pub async fn complete_contact_mail_sync_source(
        &self,
        input: &CompleteContactMailSyncSourceV1,
    ) -> Result<CompleteContactMailSyncSourceOutcomeV1, MailContactsSyncPersistenceErrorV1> {
        validate_source_completion(input)?;
        let mut transaction = self.pool.begin().await.map_err(storage)?;
        let inserted = reserve_result_inbox(&mut transaction, input).await?;
        if !inserted {
            validate_result_replay(&mut transaction, input).await?;
            transaction.commit().await.map_err(storage)?;
            return Ok(CompleteContactMailSyncSourceOutcomeV1::Duplicate);
        }
        let current = sqlx::query(
            "SELECT state FROM hermes_data.mail_contacts_sync_reverse_operations \
             WHERE logical_owner_id = $1 AND operation_id = $2 FOR UPDATE",
        )
        .bind(&input.logical_owner_id)
        .bind(input.operation_id.as_slice())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(storage)?
        .ok_or(MailContactsSyncPersistenceErrorV1::NotFound)?;
        if current.get::<i16, _>("state") != 1 {
            return Err(MailContactsSyncPersistenceErrorV1::InvalidTransition);
        }
        if let Some(command) = &input.mail_command {
            super::repository::insert_outbox(
                &mut transaction,
                &input.logical_owner_id,
                command,
                input.occurred_at_unix_millis,
            )
            .await?;
        }
        let updated = sqlx::query(
            "UPDATE hermes_data.mail_contacts_sync_reverse_operations SET state = $3, \
             mail_command_message_id = $4, terminal_message_id = $5, \
             updated_at_unix_millis = $6 WHERE logical_owner_id = $1 AND operation_id = $2 \
             AND state = 1",
        )
        .bind(&input.logical_owner_id)
        .bind(input.operation_id.as_slice())
        .bind(if input.rejected { 4_i16 } else { 2_i16 })
        .bind(
            input
                .mail_command
                .as_ref()
                .map(|value| value.message_id.to_vec()),
        )
        .bind(input.rejected.then_some(input.result_message_id.to_vec()))
        .bind(input.occurred_at_unix_millis)
        .execute(&mut *transaction)
        .await
        .map_err(storage)?;
        if updated.rows_affected() != 1 {
            return Err(MailContactsSyncPersistenceErrorV1::RevisionConflict);
        }
        transaction.commit().await.map_err(storage)?;
        Ok(CompleteContactMailSyncSourceOutcomeV1::Applied)
    }
}

async fn reserve_result_inbox(
    transaction: &mut Transaction<'_, Postgres>,
    input: &CompleteContactMailSyncSourceV1,
) -> Result<bool, MailContactsSyncPersistenceErrorV1> {
    sqlx::query(
        "INSERT INTO hermes_data.mail_contacts_sync_reverse_inbox (logical_owner_id, \
         event_message_id, event_envelope_sha256, completed_at_unix_millis) VALUES ($1,$2,$3,$4) \
         ON CONFLICT DO NOTHING",
    )
    .bind(&input.logical_owner_id)
    .bind(input.result_message_id.as_slice())
    .bind(input.result_envelope_sha256.as_slice())
    .bind(input.occurred_at_unix_millis)
    .execute(&mut **transaction)
    .await
    .map_err(storage)
    .map(|result| result.rows_affected() == 1)
}

async fn validate_result_replay(
    transaction: &mut Transaction<'_, Postgres>,
    input: &CompleteContactMailSyncSourceV1,
) -> Result<(), MailContactsSyncPersistenceErrorV1> {
    let row = sqlx::query(
        "SELECT event_envelope_sha256 FROM hermes_data.mail_contacts_sync_reverse_inbox \
         WHERE logical_owner_id = $1 AND event_message_id = $2 FOR UPDATE",
    )
    .bind(&input.logical_owner_id)
    .bind(input.result_message_id.as_slice())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(storage)?
    .ok_or(MailContactsSyncPersistenceErrorV1::InboxConflict)?;
    let hash: Vec<u8> = row.get("event_envelope_sha256");
    if hash.as_slice() != input.result_envelope_sha256 {
        return Err(MailContactsSyncPersistenceErrorV1::InboxConflict);
    }
    Ok(())
}

fn decode_operation(
    operation_id: [u8; 16],
    row: &sqlx::postgres::PgRow,
) -> Result<MailContactsSyncReverseOperationV1, MailContactsSyncPersistenceErrorV1> {
    let contact_id: Vec<u8> = row.get("contact_id");
    let contact_id = contact_id
        .try_into()
        .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidRow)?;
    let contact_revision = u64::try_from(row.get::<i64, _>("contact_revision"))
        .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidRow)?;
    let state = u8::try_from(row.get::<i16, _>("state"))
        .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidRow)?;
    if !(1..=5).contains(&state) {
        return Err(MailContactsSyncPersistenceErrorV1::InvalidRow);
    }
    Ok(MailContactsSyncReverseOperationV1 {
        operation_id,
        configuration_instance_id: row.get("configuration_instance_id"),
        account_id: row.get("account_id"),
        contact_id,
        contact_revision,
        state,
    })
}

async fn validate_replay(
    transaction: &mut Transaction<'_, Postgres>,
    input: &AcceptContactChangedForMailSyncV1,
) -> Result<(), MailContactsSyncPersistenceErrorV1> {
    let row = sqlx::query(
        "SELECT event_envelope_sha256 FROM hermes_data.mail_contacts_sync_reverse_inbox \
         WHERE logical_owner_id = $1 AND event_message_id = $2 FOR UPDATE",
    )
    .bind(&input.logical_owner_id)
    .bind(input.event_message_id.as_slice())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(storage)?
    .ok_or(MailContactsSyncPersistenceErrorV1::InboxConflict)?;
    let hash: Vec<u8> = row.get("event_envelope_sha256");
    if hash.as_slice() != input.event_envelope_sha256 {
        return Err(MailContactsSyncPersistenceErrorV1::InboxConflict);
    }
    Ok(())
}

fn storage(_: sqlx::Error) -> MailContactsSyncPersistenceErrorV1 {
    MailContactsSyncPersistenceErrorV1::StorageUnavailable
}
