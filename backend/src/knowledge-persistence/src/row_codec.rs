use sqlx::{Row, postgres::PgRow};

use crate::{
    KnowledgeBlobCleanupV1, KnowledgeBlobReceiptV1, KnowledgeOutboxRecordV1,
    KnowledgePersistenceErrorV1, PersistedReviewedCandidateCommandV1,
};

pub(crate) fn decode_command(
    row: &PgRow,
) -> Result<PersistedReviewedCandidateCommandV1, KnowledgePersistenceErrorV1> {
    let materialized_reference: Option<Vec<u8>> = get(row, "materialized_blob_reference_id")?;
    let materialized_declared_bytes: Option<i64> = get(row, "materialized_blob_declared_bytes")?;
    let materialized_sha256: Option<Vec<u8>> = get(row, "materialized_blob_sha256")?;
    let materialized_custody_proof: Option<Vec<u8>> = get(row, "materialized_blob_custody_proof")?;
    let candidate_content = KnowledgeBlobReceiptV1 {
        reference_id: fixed(get(row, "candidate_blob_reference_id")?)?,
        declared_bytes: positive_u64(get(row, "candidate_blob_declared_bytes")?)?,
        sha256: fixed(get(row, "candidate_blob_sha256")?)?,
        custody_transfer_source_proof: get(row, "candidate_blob_custody_proof")?,
    };
    let materialization = match (
        materialized_reference,
        materialized_declared_bytes,
        materialized_sha256,
        materialized_custody_proof,
    ) {
        (None, None, None, None) => None,
        (Some(reference_id), Some(declared_bytes), Some(sha256), Some(custody_proof)) => {
            Some(KnowledgeBlobCleanupV1 {
                reference_id: fixed(reference_id)?,
                declared_bytes: positive_u64(declared_bytes)?,
                sha256: fixed(sha256)?,
                custody_proof,
            })
        }
        _ => return Err(KnowledgePersistenceErrorV1::InvalidRow),
    };
    Ok(PersistedReviewedCandidateCommandV1 {
        logical_owner_id: get(row, "logical_owner_id")?,
        command_message_id: fixed(get(row, "command_message_id")?)?,
        command_envelope_sha256: fixed(get(row, "command_envelope_sha256")?)?,
        command_id: fixed(get(row, "command_id")?)?,
        command_fingerprint: fixed(get(row, "command_fingerprint")?)?,
        approved_candidate_id: fixed(get(row, "approved_candidate_id")?)?,
        candidate_digest: fixed(get(row, "candidate_digest")?)?,
        source_evidence_id: fixed(get(row, "source_evidence_id")?)?,
        source_evidence_revision: positive_u64(get(row, "source_evidence_revision")?)?,
        review_id: fixed(get(row, "review_id")?)?,
        decision_revision: positive_u64(get(row, "decision_revision")?)?,
        decided_by_owner_device_id: fixed(get(row, "decided_by_owner_device_id")?)?,
        candidate_content,
        materialization,
        cleanup_completed_at_unix_millis: get(row, "cleanup_completed_at_unix_millis")?,
        completed: get(row, "completed")?,
        rejected: get(row, "rejected")?,
        note_id: optional_fixed(get(row, "note_id")?)?,
        note_creation_fingerprint: optional_fixed(get(row, "note_creation_fingerprint")?)?,
        received_at_unix_millis: get(row, "received_at_unix_millis")?,
    })
}

pub(crate) fn decode_outbox(
    row: &PgRow,
) -> Result<KnowledgeOutboxRecordV1, KnowledgePersistenceErrorV1> {
    Ok(KnowledgeOutboxRecordV1 {
        message_id: fixed(get(row, "message_id")?)?,
        envelope_sha256: fixed(get(row, "envelope_sha256")?)?,
        envelope_bytes: get(row, "envelope_bytes")?,
    })
}

fn get<T>(row: &PgRow, column: &str) -> Result<T, KnowledgePersistenceErrorV1>
where
    for<'r> T: sqlx::Decode<'r, sqlx::Postgres> + sqlx::Type<sqlx::Postgres>,
{
    row.try_get(column)
        .map_err(|_| KnowledgePersistenceErrorV1::InvalidRow)
}

fn fixed<const N: usize>(value: Vec<u8>) -> Result<[u8; N], KnowledgePersistenceErrorV1> {
    value
        .try_into()
        .map_err(|_| KnowledgePersistenceErrorV1::InvalidRow)
}

fn optional_fixed<const N: usize>(
    value: Option<Vec<u8>>,
) -> Result<Option<[u8; N]>, KnowledgePersistenceErrorV1> {
    value.map(fixed).transpose()
}

fn positive_u64(value: i64) -> Result<u64, KnowledgePersistenceErrorV1> {
    u64::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .ok_or(KnowledgePersistenceErrorV1::InvalidRow)
}
