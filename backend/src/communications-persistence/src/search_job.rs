//! Typed owner-local derived-index job records; no content or key material crosses this seam.

use hermes_communications_api::{
    CommunicationBodyBlobReferenceV1, CommunicationConversationIdV1, CommunicationMessageIdV1,
    CommunicationObservationIdV1,
};
use sqlx::Row;

use crate::{CommunicationsDurablePersistence, CommunicationsPersistenceError};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsDerivedIndexJobOperationV1 { Index, Remove }

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationsDerivedIndexJobV1 {
    pub job_id: [u8; 16],
    pub operation: CommunicationsDerivedIndexJobOperationV1,
    pub evidence_id: CommunicationObservationIdV1,
    pub message_id: CommunicationMessageIdV1,
    pub conversation_id: Option<CommunicationConversationIdV1>,
    pub blob: Option<CommunicationBodyBlobReferenceV1>,
    pub projection_revision: u32,
    pub observed_at_unix_seconds: i64,
    pub created_at_unix_seconds: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsDerivedIndexJobErrorV1 { InvalidShape }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsDerivedIndexFailureV1 {
    BlobDenied,
    BlobUnavailable,
    InvalidContent,
    VaultDenied,
    VaultUnavailable,
    DocumentLimit,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationsDerivedIndexFailureRecordV1 {
    pub evidence_id: CommunicationObservationIdV1,
    pub message_id: CommunicationMessageIdV1,
    pub projection_revision: u32,
    pub observed_at_unix_seconds: i64,
    pub failure: CommunicationsDerivedIndexFailureV1,
    pub recorded_at_unix_seconds: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClaimedCommunicationsDerivedIndexJobV1 {
    pub job: CommunicationsDerivedIndexJobV1,
    pub worker_id: String,
}

impl CommunicationsDerivedIndexJobV1 {
    pub fn validate(&self) -> Result<(), CommunicationsDerivedIndexJobErrorV1> {
        if self.job_id.iter().all(|byte| *byte == 0) || self.projection_revision == 0 {
            return Err(CommunicationsDerivedIndexJobErrorV1::InvalidShape);
        }
        match self.operation {
            CommunicationsDerivedIndexJobOperationV1::Index
                if self.conversation_id.is_some()
                    && self.blob.as_ref().is_some_and(|blob| (1..=256 * 1024).contains(&blob.declared_bytes)) =>
            {
                Ok(())
            }
            CommunicationsDerivedIndexJobOperationV1::Remove if self.conversation_id.is_none() && self.blob.is_none() => Ok(()),
            _ => Err(CommunicationsDerivedIndexJobErrorV1::InvalidShape),
        }
    }
}

impl CommunicationsDurablePersistence {
    pub async fn claim_next_derived_index_job(
        &self,
        worker_id: &str,
        claimed_at_unix_seconds: i64,
        lease_expires_at_unix_seconds: i64,
    ) -> Result<Option<ClaimedCommunicationsDerivedIndexJobV1>, CommunicationsPersistenceError> {
        if worker_id.is_empty()
            || worker_id.len() > 256
            || !worker_id.is_ascii()
            || lease_expires_at_unix_seconds <= claimed_at_unix_seconds
        {
            return Err(CommunicationsPersistenceError::InvalidRow);
        }
        let row = sqlx::query(
            "WITH candidate AS (SELECT job_id FROM hermes_data.communications_derived_index_jobs WHERE completed_at_unix_seconds IS NULL AND (lease_expires_at_unix_seconds IS NULL OR lease_expires_at_unix_seconds <= $2) ORDER BY created_at_unix_seconds ASC, job_id ASC LIMIT 1 FOR UPDATE SKIP LOCKED) UPDATE hermes_data.communications_derived_index_jobs AS job SET claimed_by = $1, lease_expires_at_unix_seconds = $3, attempt_count = job.attempt_count + 1 FROM candidate WHERE job.job_id = candidate.job_id RETURNING job.job_id, job.operation, job.evidence_id, job.message_id, job.conversation_id, job.blob_ref, job.blob_reference_id, job.blob_declared_bytes, job.blob_sha256, job.projection_revision, job.observed_at_unix_seconds, job.created_at_unix_seconds",
        )
        .bind(worker_id)
        .bind(claimed_at_unix_seconds)
        .bind(lease_expires_at_unix_seconds)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
        row.map(|row| claimed_job_from_row(row, worker_id)).transpose()
    }

    pub async fn complete_derived_index_job(
        &self,
        job_id: [u8; 16],
        worker_id: &str,
        completed_at_unix_seconds: i64,
    ) -> Result<bool, CommunicationsPersistenceError> {
        settle_derived_index_job(&self.pool, job_id, worker_id, 1, None, completed_at_unix_seconds).await
    }

    pub async fn fail_derived_index_job(
        &self,
        job_id: [u8; 16],
        worker_id: &str,
        failure: CommunicationsDerivedIndexFailureV1,
        completed_at_unix_seconds: i64,
    ) -> Result<bool, CommunicationsPersistenceError> {
        settle_derived_index_job(&self.pool, job_id, worker_id, 2, Some(failure), completed_at_unix_seconds).await
    }
}

async fn settle_derived_index_job(
    pool: &sqlx::PgPool,
    job_id: [u8; 16],
    worker_id: &str,
    outcome: i16,
    failure: Option<CommunicationsDerivedIndexFailureV1>,
    completed_at_unix_seconds: i64,
) -> Result<bool, CommunicationsPersistenceError> {
    let result = sqlx::query(
        "UPDATE hermes_data.communications_derived_index_jobs SET completed_at_unix_seconds = $3, outcome = $4, failure_code = $5, claimed_by = NULL, lease_expires_at_unix_seconds = NULL WHERE job_id = $1 AND claimed_by = $2 AND completed_at_unix_seconds IS NULL",
    )
    .bind(job_id.as_slice())
    .bind(worker_id)
    .bind(completed_at_unix_seconds)
    .bind(outcome)
    .bind(failure.map(failure_value))
    .execute(pool)
    .await
    .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
    Ok(result.rows_affected() == 1)
}

fn claimed_job_from_row(
    row: sqlx::postgres::PgRow,
    worker_id: &str,
) -> Result<ClaimedCommunicationsDerivedIndexJobV1, CommunicationsPersistenceError> {
    let operation = match row.try_get::<i16, _>("operation").map_err(|_| CommunicationsPersistenceError::InvalidRow)? {
        1 => CommunicationsDerivedIndexJobOperationV1::Index,
        2 => CommunicationsDerivedIndexJobOperationV1::Remove,
        _ => return Err(CommunicationsPersistenceError::InvalidRow),
    };
    let job = CommunicationsDerivedIndexJobV1 {
        job_id: id16(&row.try_get::<Vec<u8>, _>("job_id").map_err(|_| CommunicationsPersistenceError::InvalidRow)?)?,
        operation,
        evidence_id: CommunicationObservationIdV1::new(id16(&row.try_get::<Vec<u8>, _>("evidence_id").map_err(|_| CommunicationsPersistenceError::InvalidRow)?)?),
        message_id: CommunicationMessageIdV1::new(id16(&row.try_get::<Vec<u8>, _>("message_id").map_err(|_| CommunicationsPersistenceError::InvalidRow)?)?),
        conversation_id: row.try_get::<Option<Vec<u8>>, _>("conversation_id").map_err(|_| CommunicationsPersistenceError::InvalidRow)?.map(|value| id16(&value).map(CommunicationConversationIdV1::new)).transpose()?,
        blob: blob_from_row(&row)?,
        projection_revision: u32::try_from(row.try_get::<i32, _>("projection_revision").map_err(|_| CommunicationsPersistenceError::InvalidRow)?).map_err(|_| CommunicationsPersistenceError::InvalidRow)?,
        observed_at_unix_seconds: row.try_get("observed_at_unix_seconds").map_err(|_| CommunicationsPersistenceError::InvalidRow)?,
        created_at_unix_seconds: row.try_get("created_at_unix_seconds").map_err(|_| CommunicationsPersistenceError::InvalidRow)?,
    };
    job.validate().map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
    Ok(ClaimedCommunicationsDerivedIndexJobV1 { job, worker_id: worker_id.to_owned() })
}

fn blob_from_row(row: &sqlx::postgres::PgRow) -> Result<Option<CommunicationBodyBlobReferenceV1>, CommunicationsPersistenceError> {
    let blob_ref: Option<String> = row.try_get("blob_ref").map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
    let reference_id: Option<Vec<u8>> = row.try_get("blob_reference_id").map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
    let declared_bytes: Option<i64> = row.try_get("blob_declared_bytes").map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
    let sha256: Option<Vec<u8>> = row.try_get("blob_sha256").map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
    match (blob_ref, reference_id, declared_bytes, sha256) {
        (None, None, None, None) => Ok(None),
        (Some(blob_ref), Some(reference_id), Some(declared_bytes), Some(sha256)) => Ok(Some(CommunicationBodyBlobReferenceV1 {
            blob_ref,
            reference_id: id16(&reference_id)?,
            declared_bytes: u64::try_from(declared_bytes).map_err(|_| CommunicationsPersistenceError::InvalidRow)?,
            sha256: id32(&sha256)?,
        })),
        _ => Err(CommunicationsPersistenceError::InvalidRow),
    }
}

fn failure_value(value: CommunicationsDerivedIndexFailureV1) -> i16 {
    match value {
        CommunicationsDerivedIndexFailureV1::BlobDenied => 1,
        CommunicationsDerivedIndexFailureV1::BlobUnavailable => 2,
        CommunicationsDerivedIndexFailureV1::InvalidContent => 3,
        CommunicationsDerivedIndexFailureV1::VaultDenied => 4,
        CommunicationsDerivedIndexFailureV1::VaultUnavailable => 5,
        CommunicationsDerivedIndexFailureV1::DocumentLimit => 6,
    }
}

fn id16(value: &[u8]) -> Result<[u8; 16], CommunicationsPersistenceError> {
    value.try_into().map_err(|_| CommunicationsPersistenceError::InvalidRow)
}

fn id32(value: &[u8]) -> Result<[u8; 32], CommunicationsPersistenceError> {
    value.try_into().map_err(|_| CommunicationsPersistenceError::InvalidRow)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn index_job(declared_bytes: u64) -> CommunicationsDerivedIndexJobV1 {
        CommunicationsDerivedIndexJobV1 {
            job_id: [1; 16],
            operation: CommunicationsDerivedIndexJobOperationV1::Index,
            evidence_id: CommunicationObservationIdV1::new([2; 16]),
            message_id: CommunicationMessageIdV1::new([3; 16]),
            conversation_id: Some(CommunicationConversationIdV1::new([4; 16])),
            blob: Some(CommunicationBodyBlobReferenceV1 {
                blob_ref: "owner-local-blob".to_owned(),
                reference_id: [5; 16],
                declared_bytes,
                sha256: [6; 32],
            }),
            projection_revision: 1,
            observed_at_unix_seconds: 1,
            created_at_unix_seconds: 1,
        }
    }

    #[test]
    fn index_job_accepts_the_full_admitted_blob_range_only() {
        assert_eq!(index_job(256 * 1024).validate(), Ok(()));
        assert_eq!(
            index_job(256 * 1024 + 1).validate(),
            Err(CommunicationsDerivedIndexJobErrorV1::InvalidShape),
        );
    }
}
