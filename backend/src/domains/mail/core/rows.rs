use sqlx::Row;
use sqlx::postgres::PgRow;

use super::errors::CommunicationIngestionError;
use super::models::{
    CommunicationProviderKind, IngestionCheckpoint, ProviderAccount, ProviderAccountSecretBinding,
    ProviderAccountSecretPurpose, StoredRawCommunicationRecord,
};

pub(super) fn row_to_provider_account(
    row: PgRow,
) -> Result<ProviderAccount, CommunicationIngestionError> {
    let provider_kind =
        CommunicationProviderKind::try_from(row.try_get::<String, _>("provider_kind")?.as_str())?;

    Ok(ProviderAccount {
        account_id: row.try_get("account_id")?,
        provider_kind,
        display_name: row.try_get("display_name")?,
        external_account_id: row.try_get("external_account_id")?,
        config: row.try_get("config")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

pub(super) fn row_to_raw_record(
    row: PgRow,
) -> Result<StoredRawCommunicationRecord, CommunicationIngestionError> {
    Ok(StoredRawCommunicationRecord {
        raw_record_id: row.try_get("raw_record_id")?,
        account_id: row.try_get("account_id")?,
        record_kind: row.try_get("record_kind")?,
        provider_record_id: row.try_get("provider_record_id")?,
        source_fingerprint: row.try_get("source_fingerprint")?,
        import_batch_id: row.try_get("import_batch_id")?,
        occurred_at: row.try_get("occurred_at")?,
        captured_at: row.try_get("captured_at")?,
        payload: row.try_get("payload")?,
        provenance: row.try_get("provenance")?,
    })
}

pub(super) fn row_to_checkpoint(
    row: PgRow,
) -> Result<IngestionCheckpoint, CommunicationIngestionError> {
    Ok(IngestionCheckpoint {
        account_id: row.try_get("account_id")?,
        stream_id: row.try_get("stream_id")?,
        checkpoint: row.try_get("checkpoint")?,
        updated_at: row.try_get("updated_at")?,
    })
}

pub(super) fn row_to_secret_binding(
    row: PgRow,
) -> Result<ProviderAccountSecretBinding, CommunicationIngestionError> {
    let secret_purpose = ProviderAccountSecretPurpose::try_from(
        row.try_get::<String, _>("secret_purpose")?.as_str(),
    )?;

    Ok(ProviderAccountSecretBinding {
        account_id: row.try_get("account_id")?,
        secret_purpose,
        secret_ref: row.try_get("secret_ref")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}
