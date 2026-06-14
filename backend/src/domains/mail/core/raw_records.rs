use super::errors::CommunicationIngestionError;
use super::models::{NewRawCommunicationRecord, StoredRawCommunicationRecord};
use super::rows::row_to_raw_record;
use super::store::CommunicationIngestionStore;
use super::validation::validate_non_empty;

impl CommunicationIngestionStore {
    pub async fn record_raw_source(
        &self,
        record: &NewRawCommunicationRecord,
    ) -> Result<StoredRawCommunicationRecord, CommunicationIngestionError> {
        record.validate()?;

        let inserted = sqlx::query(
            r#"
            INSERT INTO communication_raw_records (
                raw_record_id,
                account_id,
                record_kind,
                provider_record_id,
                source_fingerprint,
                import_batch_id,
                occurred_at,
                payload,
                provenance
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (account_id, record_kind, provider_record_id)
            DO NOTHING
            RETURNING
                raw_record_id,
                account_id,
                record_kind,
                provider_record_id,
                source_fingerprint,
                import_batch_id,
                occurred_at,
                captured_at,
                payload,
                provenance
            "#,
        )
        .bind(record.raw_record_id.trim())
        .bind(record.account_id.trim())
        .bind(record.record_kind.trim())
        .bind(record.provider_record_id.trim())
        .bind(record.source_fingerprint.trim())
        .bind(record.import_batch_id.trim())
        .bind(record.occurred_at)
        .bind(&record.payload)
        .bind(&record.provenance)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = inserted {
            return row_to_raw_record(row);
        }

        let row = sqlx::query(
            r#"
            SELECT
                raw_record_id,
                account_id,
                record_kind,
                provider_record_id,
                source_fingerprint,
                import_batch_id,
                occurred_at,
                captured_at,
                payload,
                provenance
            FROM communication_raw_records
            WHERE account_id = $1
              AND record_kind = $2
              AND provider_record_id = $3
            "#,
        )
        .bind(record.account_id.trim())
        .bind(record.record_kind.trim())
        .bind(record.provider_record_id.trim())
        .fetch_one(&self.pool)
        .await?;

        row_to_raw_record(row)
    }

    pub async fn raw_record(
        &self,
        raw_record_id: &str,
    ) -> Result<Option<StoredRawCommunicationRecord>, CommunicationIngestionError> {
        validate_non_empty("raw_record_id", raw_record_id)?;

        let row = sqlx::query(
            r#"
            SELECT
                raw_record_id,
                account_id,
                record_kind,
                provider_record_id,
                source_fingerprint,
                import_batch_id,
                occurred_at,
                captured_at,
                payload,
                provenance
            FROM communication_raw_records
            WHERE raw_record_id = $1
            "#,
        )
        .bind(raw_record_id.trim())
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_raw_record).transpose()
    }
}
