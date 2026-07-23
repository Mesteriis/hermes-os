//! Owner-local persistence for opaque Communications search projection digests.

use hermes_communications_api::{
    CommunicationConversationIdV1, CommunicationMessageIdV1, CommunicationObservationIdV1,
    CommunicationSearchHitV1,
};
use sqlx::Row;

use crate::{CommunicationsDurablePersistence, CommunicationsPersistenceError};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationsSearchProjectionWriteV1 {
    pub evidence_id: CommunicationObservationIdV1,
    pub message_id: CommunicationMessageIdV1,
    pub conversation_id: CommunicationConversationIdV1,
    pub observed_at_unix_seconds: i64,
    pub projection_revision: u32,
    pub indexed_at_unix_seconds: i64,
    pub token_digests: Vec<[u8; 32]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsSearchProjectionWriteErrorV1 {
    Empty,
    DuplicateDigest,
}

impl CommunicationsDurablePersistence {
    /// Replaces one message's derived digest set when the revision is current.
    /// The stored rows intentionally never contain query or source-content text.
    pub async fn replace_search_projection(
        &self,
        projection: &CommunicationsSearchProjectionWriteV1,
    ) -> Result<bool, CommunicationsPersistenceError> {
        validate_projection(projection).map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
        let mut transaction = self.pool.begin().await
            .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
        let applied = sqlx::query(
            "INSERT INTO hermes_data.communications_derived_index_projections (message_id, evidence_id, conversation_id, observed_at_unix_seconds, projection_revision, indexed_at_unix_seconds) SELECT $1, $2, $3, $4, $5, $6 WHERE NOT EXISTS (SELECT 1 FROM hermes_data.communications_derived_index_tombstones WHERE message_id = $1 AND (projection_revision > $5 OR (projection_revision = $5 AND observed_at_unix_seconds >= $4))) ON CONFLICT (message_id) DO UPDATE SET evidence_id = EXCLUDED.evidence_id, conversation_id = EXCLUDED.conversation_id, observed_at_unix_seconds = EXCLUDED.observed_at_unix_seconds, projection_revision = EXCLUDED.projection_revision, indexed_at_unix_seconds = EXCLUDED.indexed_at_unix_seconds WHERE communications_derived_index_projections.projection_revision < EXCLUDED.projection_revision OR (communications_derived_index_projections.projection_revision = EXCLUDED.projection_revision AND communications_derived_index_projections.observed_at_unix_seconds <= EXCLUDED.observed_at_unix_seconds) RETURNING message_id",
        )
        .bind(projection.message_id.bytes().as_slice())
        .bind(projection.evidence_id.bytes().as_slice())
        .bind(projection.conversation_id.bytes().as_slice())
        .bind(projection.observed_at_unix_seconds)
        .bind(i32::try_from(projection.projection_revision).map_err(|_| CommunicationsPersistenceError::InvalidRow)?)
        .bind(projection.indexed_at_unix_seconds)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?
        .is_some();
        if !applied {
            transaction.rollback().await.map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
            return Ok(false);
        }
        sqlx::query("DELETE FROM hermes_data.communications_derived_index_tombstones WHERE message_id = $1 AND (projection_revision < $2 OR (projection_revision = $2 AND observed_at_unix_seconds <= $3))")
            .bind(projection.message_id.bytes().as_slice())
            .bind(i32::try_from(projection.projection_revision).map_err(|_| CommunicationsPersistenceError::InvalidRow)?)
            .bind(projection.observed_at_unix_seconds)
            .execute(&mut *transaction)
            .await
            .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
        sqlx::query("DELETE FROM hermes_data.communications_derived_index_token_digests WHERE message_id = $1")
            .bind(projection.message_id.bytes().as_slice())
            .execute(&mut *transaction)
            .await
            .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
        for digest in &projection.token_digests {
            sqlx::query("INSERT INTO hermes_data.communications_derived_index_token_digests (message_id, token_digest) VALUES ($1, $2)")
                .bind(projection.message_id.bytes().as_slice())
                .bind(digest.as_slice())
                .execute(&mut *transaction)
                .await
                .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
        }
        transaction.commit().await.map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
        Ok(true)
    }

    pub async fn remove_search_projection(
        &self,
        evidence_id: CommunicationObservationIdV1,
        message_id: CommunicationMessageIdV1,
        projection_revision: u32,
        observed_at_unix_seconds: i64,
    ) -> Result<bool, CommunicationsPersistenceError> {
        if projection_revision == 0 {
            return Err(CommunicationsPersistenceError::InvalidRow);
        }
        let revision = i32::try_from(projection_revision).map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
        let mut transaction = self.pool.begin().await
            .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
        let applied = sqlx::query("INSERT INTO hermes_data.communications_derived_index_tombstones (message_id, evidence_id, observed_at_unix_seconds, projection_revision, removed_at_unix_seconds) VALUES ($1, $2, $3, $4, $3) ON CONFLICT (message_id) DO UPDATE SET evidence_id = EXCLUDED.evidence_id, observed_at_unix_seconds = EXCLUDED.observed_at_unix_seconds, projection_revision = EXCLUDED.projection_revision, removed_at_unix_seconds = EXCLUDED.removed_at_unix_seconds WHERE hermes_data.communications_derived_index_tombstones.projection_revision < EXCLUDED.projection_revision OR (hermes_data.communications_derived_index_tombstones.projection_revision = EXCLUDED.projection_revision AND hermes_data.communications_derived_index_tombstones.observed_at_unix_seconds <= EXCLUDED.observed_at_unix_seconds) RETURNING message_id")
            .bind(message_id.bytes().as_slice())
            .bind(evidence_id.bytes().as_slice())
            .bind(observed_at_unix_seconds)
            .bind(revision)
            .fetch_optional(&mut *transaction)
            .await
            .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?
            .is_some();
        if !applied {
            transaction.rollback().await.map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
            return Ok(false);
        }
        sqlx::query("DELETE FROM hermes_data.communications_derived_index_projections WHERE message_id = $1 AND (projection_revision < $2 OR (projection_revision = $2 AND observed_at_unix_seconds <= $3))")
            .bind(message_id.bytes().as_slice())
            .bind(revision)
            .bind(observed_at_unix_seconds)
            .execute(&mut *transaction)
            .await
            .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
        transaction.commit().await.map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
        Ok(true)
    }

    pub async fn search_by_token_digests(
        &self,
        token_digests: &[[u8; 32]],
        limit: u16,
    ) -> Result<Vec<CommunicationSearchHitV1>, CommunicationsPersistenceError> {
        if token_digests.is_empty() || token_digests.len() > 16 || limit == 0 || limit > 100 {
            return Err(CommunicationsPersistenceError::InvalidRow);
        }
        let digests = token_digests.iter().map(|digest| digest.to_vec()).collect::<Vec<_>>();
        let rows = sqlx::query(
            "SELECT projection.evidence_id, projection.message_id, projection.conversation_id, projection.observed_at_unix_seconds, COUNT(DISTINCT digest.token_digest) AS matched_token_count FROM hermes_data.communications_derived_index_projections projection JOIN hermes_data.communications_derived_index_token_digests digest ON digest.message_id = projection.message_id WHERE digest.token_digest = ANY($1::bytea[]) GROUP BY projection.evidence_id, projection.message_id, projection.conversation_id, projection.observed_at_unix_seconds HAVING COUNT(DISTINCT digest.token_digest) = $2 ORDER BY projection.observed_at_unix_seconds DESC, projection.message_id ASC LIMIT $3",
        )
        .bind(digests)
        .bind(i64::try_from(token_digests.len()).map_err(|_| CommunicationsPersistenceError::InvalidRow)?)
        .bind(i64::from(limit))
        .fetch_all(&self.pool)
        .await
        .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
        rows.into_iter().map(|row| {
            let evidence_id: Vec<u8> = row.try_get("evidence_id").map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
            let message_id: Vec<u8> = row.try_get("message_id").map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
            let conversation_id: Vec<u8> = row.try_get("conversation_id").map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
            let observed_at_unix_seconds: i64 = row.try_get("observed_at_unix_seconds").map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
            let matched_token_count: i64 = row.try_get("matched_token_count").map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
            Ok(CommunicationSearchHitV1 {
                evidence_id: CommunicationObservationIdV1::new(id16(&evidence_id)?),
                message_id: CommunicationMessageIdV1::new(id16(&message_id)?),
                conversation_id: CommunicationConversationIdV1::new(id16(&conversation_id)?),
                observed_at_unix_seconds,
                matched_token_count: u16::try_from(matched_token_count).map_err(|_| CommunicationsPersistenceError::InvalidRow)?,
            })
        }).collect()
    }
}

fn validate_projection(
    projection: &CommunicationsSearchProjectionWriteV1,
) -> Result<(), CommunicationsSearchProjectionWriteErrorV1> {
    if projection.projection_revision == 0 || projection.token_digests.is_empty() {
        return Err(CommunicationsSearchProjectionWriteErrorV1::Empty);
    }
    if projection.token_digests.len() > 2_048
        || projection.token_digests.iter().enumerate().any(|(index, digest)| {
            projection.token_digests[..index].iter().any(|prior| prior == digest)
        })
    {
        return Err(CommunicationsSearchProjectionWriteErrorV1::DuplicateDigest);
    }
    Ok(())
}

fn id16(value: &[u8]) -> Result<[u8; 16], CommunicationsPersistenceError> {
    value.try_into().map_err(|_| CommunicationsPersistenceError::InvalidRow)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projection_validation_rejects_duplicate_digests_without_sql() {
        let projection = CommunicationsSearchProjectionWriteV1 {
            evidence_id: CommunicationObservationIdV1::new([1; 16]),
            message_id: CommunicationMessageIdV1::new([2; 16]),
            conversation_id: CommunicationConversationIdV1::new([3; 16]),
            observed_at_unix_seconds: 1,
            projection_revision: 1,
            indexed_at_unix_seconds: 1,
            token_digests: vec![[4; 32], [4; 32]],
        };
        assert_eq!(validate_projection(&projection), Err(CommunicationsSearchProjectionWriteErrorV1::DuplicateDigest));
    }
}
