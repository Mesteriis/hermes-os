use hermes_attachment_text_extraction_core::{
    AttachmentTextExtractionRequestV1, AttachmentTextExtractionStateV1,
    AttachmentTextExtractionStatusV1, validate_attachment_text_status_v1,
};
use sqlx::{Postgres, Row, Transaction};

use crate::{
    AttachmentTextExtractionPersistenceErrorV1, AttachmentTextExtractionPersistenceV1,
    CreateAttachmentTextExtractionRunOutcomeV1, CreateAttachmentTextExtractionRunV1,
    PersistedAttachmentTextArtifactV1, PersistedAttachmentTextExtractionRunV1,
    model::{
        attachment_text_extraction_request_fingerprint_v1, attachment_text_extraction_run_id_v1,
        error_code, error_from_code, format_code, format_from_code, state_code, state_from_code,
        valid_id16, valid_owner, valid_sha256, validate_create,
    },
};

impl AttachmentTextExtractionPersistenceV1 {
    pub async fn create_run(
        &self,
        create: &CreateAttachmentTextExtractionRunV1,
    ) -> Result<
        CreateAttachmentTextExtractionRunOutcomeV1,
        AttachmentTextExtractionPersistenceErrorV1,
    > {
        validate_create(create)?;
        let run_id =
            attachment_text_extraction_run_id_v1(&create.logical_owner_id, create.operation_id);
        let fingerprint =
            attachment_text_extraction_request_fingerprint_v1(create.attachment_anchor_id);
        let status = hermes_attachment_text_extraction_core::accepted_attachment_text_status_v1();
        let mut transaction = self.pool.begin().await.map_err(storage_unavailable)?;
        let inserted = sqlx::query(
            "INSERT INTO hermes_data.attachment_text_extraction_runs (logical_owner_id, run_id, operation_id, request_fingerprint, attachment_anchor_id, state, state_revision, format_code, extracted_size_bytes, extraction_truncated, error_code, created_at_unix_millis, updated_at_unix_millis) VALUES ($1, $2, $3, $4, $5, $6, $7, NULL, 0, FALSE, NULL, $8, $8) ON CONFLICT (logical_owner_id, operation_id) DO NOTHING",
        )
        .bind(&create.logical_owner_id)
        .bind(run_id.as_slice())
        .bind(create.operation_id.as_slice())
        .bind(fingerprint.as_slice())
        .bind(create.attachment_anchor_id.as_slice())
        .bind(state_code(status.state))
        .bind(i64::try_from(status.state_revision).map_err(invalid_input)?)
        .bind(create.created_at_unix_millis)
        .execute(&mut *transaction)
        .await
        .map_err(storage_unavailable)?;
        if inserted.rows_affected() == 1 {
            append_realtime(
                &mut transaction,
                &create.logical_owner_id,
                run_id,
                &status,
                create.created_at_unix_millis,
            )
            .await?;
            transaction.commit().await.map_err(storage_unavailable)?;
            return Ok(CreateAttachmentTextExtractionRunOutcomeV1::Created(
                PersistedAttachmentTextExtractionRunV1 {
                    logical_owner_id: create.logical_owner_id.clone(),
                    request: AttachmentTextExtractionRequestV1 {
                        run_id,
                        operation_id: create.operation_id,
                        attachment_anchor_id: create.attachment_anchor_id,
                    },
                    request_fingerprint: fingerprint,
                    status,
                    created_at_unix_millis: create.created_at_unix_millis,
                    updated_at_unix_millis: create.created_at_unix_millis,
                },
            ));
        }
        let existing = find_by_operation(
            &mut transaction,
            &create.logical_owner_id,
            create.operation_id,
        )
        .await?
        .ok_or(AttachmentTextExtractionPersistenceErrorV1::StorageUnavailable)?;
        transaction.commit().await.map_err(storage_unavailable)?;
        if existing.request_fingerprint != fingerprint {
            return Ok(CreateAttachmentTextExtractionRunOutcomeV1::OperationCollision);
        }
        Ok(CreateAttachmentTextExtractionRunOutcomeV1::Replayed(
            existing,
        ))
    }

    pub async fn find_run(
        &self,
        logical_owner_id: &str,
        run_id: [u8; 16],
    ) -> Result<
        Option<PersistedAttachmentTextExtractionRunV1>,
        AttachmentTextExtractionPersistenceErrorV1,
    > {
        if !valid_owner(logical_owner_id) || !valid_id16(&run_id) {
            return Err(AttachmentTextExtractionPersistenceErrorV1::InvalidInput);
        }
        let row = sqlx::query(
            "SELECT logical_owner_id, run_id, operation_id, request_fingerprint, attachment_anchor_id, state, state_revision, format_code, extracted_size_bytes, extraction_truncated, error_code, created_at_unix_millis, updated_at_unix_millis FROM hermes_data.attachment_text_extraction_runs WHERE logical_owner_id = $1 AND run_id = $2",
        )
        .bind(logical_owner_id)
        .bind(run_id.as_slice())
        .fetch_optional(&self.pool)
        .await
        .map_err(storage_unavailable)?;
        row.map(run_from_row).transpose()
    }

    pub async fn transition_run(
        &self,
        logical_owner_id: &str,
        run_id: [u8; 16],
        expected_revision: u64,
        next: AttachmentTextExtractionStatusV1,
        occurred_at_unix_millis: i64,
    ) -> Result<bool, AttachmentTextExtractionPersistenceErrorV1> {
        if !valid_owner(logical_owner_id)
            || !valid_id16(&run_id)
            || expected_revision == 0
            || next.state_revision != expected_revision.saturating_add(1)
            || !validate_attachment_text_status_v1(&next)
            || occurred_at_unix_millis <= 0
            || next.state == AttachmentTextExtractionStateV1::Ready
        {
            return Err(AttachmentTextExtractionPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.pool.begin().await.map_err(storage_unavailable)?;
        let changed = update_run_status(
            &mut transaction,
            logical_owner_id,
            run_id,
            expected_revision,
            &next,
            occurred_at_unix_millis,
        )
        .await?;
        if changed {
            append_realtime(
                &mut transaction,
                logical_owner_id,
                run_id,
                &next,
                occurred_at_unix_millis,
            )
            .await?;
        }
        transaction.commit().await.map_err(storage_unavailable)?;
        Ok(changed)
    }

    pub async fn commit_ready_artifact(
        &self,
        logical_owner_id: &str,
        expected_revision: u64,
        artifact: PersistedAttachmentTextArtifactV1,
        committed_at_unix_millis: i64,
    ) -> Result<bool, AttachmentTextExtractionPersistenceErrorV1> {
        if !valid_artifact(logical_owner_id, &artifact)
            || expected_revision == 0
            || committed_at_unix_millis <= 0
        {
            return Err(AttachmentTextExtractionPersistenceErrorV1::InvalidInput);
        }
        let next = AttachmentTextExtractionStatusV1 {
            state: AttachmentTextExtractionStateV1::Ready,
            state_revision: expected_revision.saturating_add(1),
            format: Some(artifact.format),
            extracted_size_bytes: artifact.extracted_size_bytes,
            extraction_truncated: artifact.extraction_truncated,
            error: None,
        };
        if !validate_attachment_text_status_v1(&next) {
            return Err(AttachmentTextExtractionPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.pool.begin().await.map_err(storage_unavailable)?;
        let changed = update_run_status(
            &mut transaction,
            logical_owner_id,
            artifact.run_id,
            expected_revision,
            &next,
            committed_at_unix_millis,
        )
        .await?;
        if !changed {
            transaction.rollback().await.map_err(storage_unavailable)?;
            return Ok(false);
        }
        sqlx::query(
            "INSERT INTO hermes_data.attachment_text_extraction_artifacts (logical_owner_id, run_id, derived_reference_id, derived_receipt_sha256, source_receipt_sha256, parser_identity_sha256, format_code, extracted_size_bytes, extraction_truncated, committed_at_unix_millis) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) ON CONFLICT (logical_owner_id, run_id) DO NOTHING",
        )
        .bind(logical_owner_id)
        .bind(artifact.run_id.as_slice())
        .bind(artifact.derived_reference_id.as_slice())
        .bind(artifact.derived_receipt_sha256.as_slice())
        .bind(artifact.source_receipt_sha256.as_slice())
        .bind(artifact.parser_identity_sha256.as_slice())
        .bind(format_code(artifact.format))
        .bind(i64::try_from(artifact.extracted_size_bytes).map_err(invalid_input)?)
        .bind(artifact.extraction_truncated)
        .bind(committed_at_unix_millis)
        .execute(&mut *transaction)
        .await
        .map_err(storage_unavailable)?;
        append_realtime(
            &mut transaction,
            logical_owner_id,
            artifact.run_id,
            &next,
            committed_at_unix_millis,
        )
        .await?;
        transaction.commit().await.map_err(storage_unavailable)?;
        Ok(true)
    }
}

async fn find_by_operation(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    operation_id: [u8; 16],
) -> Result<
    Option<PersistedAttachmentTextExtractionRunV1>,
    AttachmentTextExtractionPersistenceErrorV1,
> {
    let row = sqlx::query(
        "SELECT logical_owner_id, run_id, operation_id, request_fingerprint, attachment_anchor_id, state, state_revision, format_code, extracted_size_bytes, extraction_truncated, error_code, created_at_unix_millis, updated_at_unix_millis FROM hermes_data.attachment_text_extraction_runs WHERE logical_owner_id = $1 AND operation_id = $2",
    )
    .bind(logical_owner_id)
    .bind(operation_id.as_slice())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(storage_unavailable)?;
    row.map(run_from_row).transpose()
}

async fn update_run_status(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    run_id: [u8; 16],
    expected_revision: u64,
    next: &AttachmentTextExtractionStatusV1,
    occurred_at_unix_millis: i64,
) -> Result<bool, AttachmentTextExtractionPersistenceErrorV1> {
    let result = sqlx::query(
        "UPDATE hermes_data.attachment_text_extraction_runs SET state = $1, state_revision = $2, format_code = $3, extracted_size_bytes = $4, extraction_truncated = $5, error_code = $6, updated_at_unix_millis = $7 WHERE logical_owner_id = $8 AND run_id = $9 AND state_revision = $10",
    )
    .bind(state_code(next.state))
    .bind(i64::try_from(next.state_revision).map_err(invalid_input)?)
    .bind(next.format.map(format_code))
    .bind(i64::try_from(next.extracted_size_bytes).map_err(invalid_input)?)
    .bind(next.extraction_truncated)
    .bind(next.error.map(error_code))
    .bind(occurred_at_unix_millis)
    .bind(logical_owner_id)
    .bind(run_id.as_slice())
    .bind(i64::try_from(expected_revision).map_err(invalid_input)?)
    .execute(&mut **transaction)
    .await
    .map_err(storage_unavailable)?;
    Ok(result.rows_affected() == 1)
}

async fn append_realtime(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    run_id: [u8; 16],
    status: &AttachmentTextExtractionStatusV1,
    occurred_at_unix_millis: i64,
) -> Result<(), AttachmentTextExtractionPersistenceErrorV1> {
    sqlx::query(
        "INSERT INTO hermes_data.attachment_text_extraction_realtime (logical_owner_id, run_id, state, state_revision, format_code, extracted_size_bytes, extraction_truncated, error_code, occurred_at_unix_millis) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
    )
    .bind(logical_owner_id)
    .bind(run_id.as_slice())
    .bind(state_code(status.state))
    .bind(i64::try_from(status.state_revision).map_err(invalid_input)?)
    .bind(status.format.map(format_code))
    .bind(i64::try_from(status.extracted_size_bytes).map_err(invalid_input)?)
    .bind(status.extraction_truncated)
    .bind(status.error.map(error_code))
    .bind(occurred_at_unix_millis)
    .execute(&mut **transaction)
    .await
    .map_err(storage_unavailable)?;
    Ok(())
}

fn run_from_row(
    row: sqlx::postgres::PgRow,
) -> Result<PersistedAttachmentTextExtractionRunV1, AttachmentTextExtractionPersistenceErrorV1> {
    let run_id = id16(row.try_get::<Vec<u8>, _>("run_id").map_err(invalid_row)?)?;
    let operation_id = id16(
        row.try_get::<Vec<u8>, _>("operation_id")
            .map_err(invalid_row)?,
    )?;
    let attachment_anchor_id = id16(
        row.try_get::<Vec<u8>, _>("attachment_anchor_id")
            .map_err(invalid_row)?,
    )?;
    let status = AttachmentTextExtractionStatusV1 {
        state: state_from_code(row.try_get("state").map_err(invalid_row)?)?,
        state_revision: u64::try_from(
            row.try_get::<i64, _>("state_revision")
                .map_err(invalid_row)?,
        )
        .map_err(invalid_row)?,
        format: row
            .try_get::<Option<i16>, _>("format_code")
            .map_err(invalid_row)?
            .map(format_from_code)
            .transpose()?,
        extracted_size_bytes: u64::try_from(
            row.try_get::<i64, _>("extracted_size_bytes")
                .map_err(invalid_row)?,
        )
        .map_err(invalid_row)?,
        extraction_truncated: row.try_get("extraction_truncated").map_err(invalid_row)?,
        error: row
            .try_get::<Option<i16>, _>("error_code")
            .map_err(invalid_row)?
            .map(error_from_code)
            .transpose()?,
    };
    if !validate_attachment_text_status_v1(&status) {
        return Err(AttachmentTextExtractionPersistenceErrorV1::InvalidRow);
    }
    Ok(PersistedAttachmentTextExtractionRunV1 {
        logical_owner_id: row.try_get("logical_owner_id").map_err(invalid_row)?,
        request: AttachmentTextExtractionRequestV1 {
            run_id,
            operation_id,
            attachment_anchor_id,
        },
        request_fingerprint: id32(
            row.try_get::<Vec<u8>, _>("request_fingerprint")
                .map_err(invalid_row)?,
        )?,
        status,
        created_at_unix_millis: row.try_get("created_at_unix_millis").map_err(invalid_row)?,
        updated_at_unix_millis: row.try_get("updated_at_unix_millis").map_err(invalid_row)?,
    })
}

fn valid_artifact(logical_owner_id: &str, value: &PersistedAttachmentTextArtifactV1) -> bool {
    valid_owner(logical_owner_id)
        && valid_id16(&value.run_id)
        && valid_id16(&value.derived_reference_id)
        && valid_sha256(&value.derived_receipt_sha256)
        && valid_sha256(&value.source_receipt_sha256)
        && valid_sha256(&value.parser_identity_sha256)
        && (1..=1_048_576).contains(&value.extracted_size_bytes)
}

fn id16(value: Vec<u8>) -> Result<[u8; 16], AttachmentTextExtractionPersistenceErrorV1> {
    value
        .try_into()
        .map_err(|_| AttachmentTextExtractionPersistenceErrorV1::InvalidRow)
}

fn id32(value: Vec<u8>) -> Result<[u8; 32], AttachmentTextExtractionPersistenceErrorV1> {
    value
        .try_into()
        .map_err(|_| AttachmentTextExtractionPersistenceErrorV1::InvalidRow)
}

fn storage_unavailable<T>(_: T) -> AttachmentTextExtractionPersistenceErrorV1 {
    AttachmentTextExtractionPersistenceErrorV1::StorageUnavailable
}

fn invalid_input<T>(_: T) -> AttachmentTextExtractionPersistenceErrorV1 {
    AttachmentTextExtractionPersistenceErrorV1::InvalidInput
}

fn invalid_row<T>(_: T) -> AttachmentTextExtractionPersistenceErrorV1 {
    AttachmentTextExtractionPersistenceErrorV1::InvalidRow
}
