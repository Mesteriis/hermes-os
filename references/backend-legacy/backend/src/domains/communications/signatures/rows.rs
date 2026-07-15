use sqlx::Row;
use sqlx::postgres::PgRow;

use super::certificate_type::CertificateType;
use super::errors::CertificateError;
use super::models::CertificateRecord;
use super::provider::CertificateProvider;
use super::storage_kind::CertificateStorageKind;
use super::trust::TrustStatus;

pub(super) const CERTIFICATE_COLUMNS: &str = "cert_id,owner_name,issuer,serial_number,fingerprint_sha256,valid_from,valid_until,cert_type,provider,storage_kind,storage_ref,trust_status,is_revoked,usage,linked_message_id,metadata,created_at,updated_at";

pub(super) fn row_to_cert(row: PgRow) -> Result<CertificateRecord, CertificateError> {
    Ok(CertificateRecord {
        cert_id: row.try_get("cert_id")?,
        owner_name: row.try_get("owner_name")?,
        issuer: row.try_get("issuer")?,
        serial_number: row.try_get("serial_number")?,
        fingerprint_sha256: row.try_get("fingerprint_sha256")?,
        valid_from: row.try_get("valid_from")?,
        valid_until: row.try_get("valid_until")?,
        cert_type: CertificateType::parse(&row.try_get::<String, _>("cert_type")?)
            .unwrap_or(CertificateType::Unknown),
        provider: CertificateProvider::parse(&row.try_get::<String, _>("provider")?)
            .unwrap_or(CertificateProvider::Other),
        storage_kind: CertificateStorageKind::parse(&row.try_get::<String, _>("storage_kind")?)
            .unwrap_or(CertificateStorageKind::EncryptedVault),
        storage_ref: row.try_get("storage_ref")?,
        trust_status: TrustStatus::parse(&row.try_get::<String, _>("trust_status")?)
            .unwrap_or(TrustStatus::Untrusted),
        is_revoked: row.try_get("is_revoked")?,
        usage: serde_json::from_value(row.try_get("usage")?).unwrap_or_default(),
        linked_message_id: row.try_get("linked_message_id")?,
        metadata: row.try_get("metadata")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}
