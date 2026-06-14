use sqlx::postgres::PgPool;

use super::rows::{CERTIFICATE_COLUMNS, row_to_cert};
use super::{CertificateError, CertificateRecord, NewCertificate};

#[derive(Clone)]
pub struct CertificateStore {
    pool: PgPool,
}

impl CertificateStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert(
        &self,
        cert: &NewCertificate,
    ) -> Result<CertificateRecord, CertificateError> {
        cert.validate()?;
        let row = sqlx::query(
            r#"INSERT INTO email_certificates (cert_id, owner_name, issuer, serial_number, fingerprint_sha256, valid_from, valid_until, cert_type, provider, storage_kind, storage_ref, trust_status, is_revoked, usage, linked_message_id, metadata)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16)
            ON CONFLICT (cert_id) DO UPDATE SET owner_name=EXCLUDED.owner_name, issuer=EXCLUDED.issuer, valid_from=EXCLUDED.valid_from, valid_until=EXCLUDED.valid_until, trust_status=EXCLUDED.trust_status, is_revoked=EXCLUDED.is_revoked, usage=EXCLUDED.usage, metadata=EXCLUDED.metadata, updated_at=now()
            RETURNING cert_id,owner_name,issuer,serial_number,fingerprint_sha256,valid_from,valid_until,cert_type,provider,storage_kind,storage_ref,trust_status,is_revoked,usage,linked_message_id,metadata,created_at,updated_at"#,
        )
        .bind(&cert.cert_id)
        .bind(&cert.owner_name)
        .bind(&cert.issuer)
        .bind(cert.serial_number.as_deref())
        .bind(cert.fingerprint_sha256.as_deref())
        .bind(cert.valid_from)
        .bind(cert.valid_until)
        .bind(cert.cert_type.as_str())
        .bind(cert.provider.as_str())
        .bind(cert.storage_kind.as_str())
        .bind(cert.storage_ref.as_deref())
        .bind(cert.trust_status.as_str())
        .bind(cert.is_revoked)
        .bind(serde_json::to_value(&cert.usage).unwrap_or_default())
        .bind(cert.linked_message_id.as_deref())
        .bind(&cert.metadata)
        .fetch_one(&self.pool)
        .await?;
        row_to_cert(row)
    }

    pub async fn list(&self) -> Result<Vec<CertificateRecord>, CertificateError> {
        let query = format!(
            "SELECT {CERTIFICATE_COLUMNS} FROM email_certificates ORDER BY COALESCE(valid_until, created_at) DESC"
        );
        let rows = sqlx::query(&query).fetch_all(&self.pool).await?;
        rows.into_iter().map(row_to_cert).collect()
    }

    pub async fn expiring_soon(
        &self,
        days: i64,
    ) -> Result<Vec<CertificateRecord>, CertificateError> {
        let query = format!(
            "SELECT {CERTIFICATE_COLUMNS} FROM email_certificates WHERE valid_until IS NOT NULL AND valid_until BETWEEN now() AND now() + ($1 || ' days')::interval AND is_revoked = false ORDER BY valid_until ASC"
        );
        let rows = sqlx::query(&query).bind(days).fetch_all(&self.pool).await?;
        rows.into_iter().map(row_to_cert).collect()
    }
}
