//! Test-only PostgreSQL diagnostics for the disposable Text Extraction contour.

use super::*;

use sqlx::{
    Row,
    postgres::{PgConnectOptions, PgPoolOptions, PgSslMode},
};
use zeroize::Zeroizing;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct AttachmentTextExtractionDiagnosticsV1 {
    pub(super) candidates: i64,
    pub(super) safety_facts: i64,
    pub(super) custody_requests: i64,
    pub(super) pending_custody_outbox: i64,
    pub(super) custody_results: i64,
    pub(super) jobs: i64,
    pub(super) attempts: i64,
    pub(super) artifacts: i64,
    pub(super) security_delegation_commands: i64,
    pub(super) security_delegation_attempts: i64,
    pub(super) security_delegation_results: i64,
}

pub(super) fn attachment_text_extraction_diagnostics_v1() -> AttachmentTextExtractionDiagnosticsV1 {
    tokio::runtime::Runtime::new()
        .expect("Attachment Text Extraction diagnostics runtime")
        .block_on(async {
            let password = Zeroizing::new(
                std::fs::read_to_string(required(
                    "HERMES_STORAGE_AUTHENTICATED_POSTGRES_PASSWORD_FILE",
                ))
                .expect("read disposable PostgreSQL credential")
                .trim()
                .to_owned(),
            );
            let options = PgConnectOptions::new()
                .host(&required("HERMES_STORAGE_AUTHENTICATED_POSTGRES_HOST"))
                .port(
                    required("HERMES_STORAGE_AUTHENTICATED_POSTGRES_PORT")
                        .parse()
                        .expect("valid PostgreSQL port"),
                )
                .username("hermes_postgres_admin")
                .password(password.as_str())
                .database("hermes_storage_authenticated")
                .ssl_mode(PgSslMode::Disable);
            let pool = PgPoolOptions::new()
                .max_connections(1)
                .connect_with(options)
                .await
                .expect("connect Attachment Text Extraction diagnostics");
            let row = sqlx::query(
                "SELECT \
                 (SELECT count(*) FROM hermes_data.attachment_text_extraction_scan_candidates) AS candidates, \
                 (SELECT count(*) FROM hermes_data.attachment_text_extraction_safety_facts) AS safety_facts, \
                 (SELECT count(*) FROM hermes_data.attachment_text_extraction_custody_outbox) AS custody_requests, \
                 (SELECT count(*) FROM hermes_data.attachment_text_extraction_custody_outbox WHERE published_at_unix_millis IS NULL) AS pending_custody_outbox, \
                 (SELECT count(*) FROM hermes_data.attachment_text_extraction_custody_result_inbox) AS custody_results, \
                 (SELECT count(*) FROM hermes_data.attachment_text_extraction_jobs) AS jobs, \
                 (SELECT coalesce(sum(attempt_count), 0) FROM hermes_data.attachment_text_extraction_jobs) AS attempts, \
                 (SELECT count(*) FROM hermes_data.attachment_text_extraction_artifacts) AS artifacts, \
                 (SELECT count(*) FROM hermes_data.attachment_security_text_extraction_delegation_inbox) AS security_delegation_commands, \
                 (SELECT coalesce(sum(attempt_count), 0) FROM hermes_data.attachment_security_text_extraction_delegation_jobs) AS security_delegation_attempts, \
                 (SELECT count(*) FROM hermes_data.attachment_security_text_extraction_delegation_outbox) AS security_delegation_results",
            )
            .fetch_one(&pool)
            .await
            .expect("read Attachment Text Extraction diagnostics");
            AttachmentTextExtractionDiagnosticsV1 {
                candidates: row.try_get("candidates").expect("candidate count"),
                safety_facts: row.try_get("safety_facts").expect("safety count"),
                custody_requests: row.try_get("custody_requests").expect("custody count"),
                pending_custody_outbox: row
                    .try_get("pending_custody_outbox")
                    .expect("pending custody count"),
                custody_results: row.try_get("custody_results").expect("result count"),
                jobs: row.try_get("jobs").expect("job count"),
                attempts: row.try_get("attempts").expect("attempt count"),
                artifacts: row.try_get("artifacts").expect("artifact count"),
                security_delegation_commands: row
                    .try_get("security_delegation_commands")
                    .expect("security command count"),
                security_delegation_attempts: row
                    .try_get("security_delegation_attempts")
                    .expect("security attempt count"),
                security_delegation_results: row
                    .try_get("security_delegation_results")
                    .expect("security result count"),
            }
        })
}
