//! Explicit disposable-database access for live conformance only.

use sqlx::{
    Row,
    postgres::{PgConnectOptions, PgPoolOptions},
};

use crate::{AttachmentSecurityPersistenceErrorV1, AttachmentSecurityPersistenceV1};

pub struct AttachmentSecurityPersistenceConformanceV1;

pub struct AttachmentSecurityPersistenceDiagnosticsV1 {
    pub candidates: i64,
    pub canonical_states: i64,
    pub jobs: i64,
    pub attempts: i64,
    pub target_blob_receipts: i64,
    pub outbox: i64,
}

impl AttachmentSecurityPersistenceConformanceV1 {
    pub async fn connect(
        host: &str,
        port: u16,
        username: &str,
        password: &str,
        database_id: &str,
    ) -> Result<AttachmentSecurityPersistenceV1, AttachmentSecurityPersistenceErrorV1> {
        if host.trim().is_empty()
            || port == 0
            || username.trim().is_empty()
            || password.is_empty()
            || database_id.trim().is_empty()
        {
            return Err(AttachmentSecurityPersistenceErrorV1::InvalidInput);
        }
        let options = PgConnectOptions::new()
            .host(host)
            .port(port)
            .username(username)
            .password(password)
            .database(database_id);
        let pool = PgPoolOptions::new()
            .max_connections(2)
            .connect_with(options)
            .await
            .map_err(|_| AttachmentSecurityPersistenceErrorV1::StorageUnavailable)?;
        Ok(AttachmentSecurityPersistenceV1 { pool })
    }

    pub async fn diagnostics(
        persistence: &AttachmentSecurityPersistenceV1,
    ) -> Result<AttachmentSecurityPersistenceDiagnosticsV1, AttachmentSecurityPersistenceErrorV1>
    {
        let row = sqlx::query(
            "SELECT \
             (SELECT count(*) FROM hermes_data.attachment_security_scan_candidates) AS candidates, \
             (SELECT count(*) FROM hermes_data.attachment_security_canonical_states) AS canonical_states, \
             (SELECT count(*) FROM hermes_data.attachment_security_scan_jobs) AS jobs, \
             (SELECT coalesce(sum(attempt_count), 0) FROM hermes_data.attachment_security_scan_jobs) AS attempts, \
             (SELECT count(*) FROM hermes_data.attachment_security_scan_jobs WHERE target_blob_reference_id IS NOT NULL) AS target_blob_receipts, \
             (SELECT count(*) FROM hermes_data.attachment_security_verdict_outbox) AS outbox",
        )
        .fetch_one(&persistence.pool)
        .await
        .map_err(|_| AttachmentSecurityPersistenceErrorV1::StorageUnavailable)?;
        Ok(AttachmentSecurityPersistenceDiagnosticsV1 {
            candidates: row
                .try_get("candidates")
                .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?,
            canonical_states: row
                .try_get("canonical_states")
                .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?,
            jobs: row
                .try_get("jobs")
                .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?,
            attempts: row
                .try_get("attempts")
                .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?,
            target_blob_receipts: row
                .try_get("target_blob_receipts")
                .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?,
            outbox: row
                .try_get("outbox")
                .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?,
        })
    }
}
