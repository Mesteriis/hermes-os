//! Test-only diagnostics for the disposable retained Preview evidence replay contour.

use std::time::{Duration, Instant};

use super::*;

use sqlx::{
    Row,
    postgres::{PgConnectOptions, PgPoolOptions, PgSslMode},
};
use zeroize::Zeroizing;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct RetainedPreviewEvidenceMessageIdsV1 {
    pub(super) communications: [u8; 16],
    pub(super) mail: [u8; 16],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct RetainedPreviewReplayDiagnosticsV1 {
    pub(super) state: i16,
    pub(super) error: i16,
    pub(super) producer_results: i64,
    pub(super) communications_published_audits: i64,
    pub(super) mail_published_audits: i64,
}

pub(super) fn wait_for_retained_preview_evidence_message_ids_v1(
    attachment_anchor_id: [u8; 16],
) -> RetainedPreviewEvidenceMessageIdsV1 {
    let runtime = tokio::runtime::Runtime::new().expect("retained Preview diagnostics runtime");
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let result = runtime.block_on(async {
            let pool = retained_preview_diagnostics_pool_v1().await;
            let communications = sqlx::query(
                "SELECT message_id FROM hermes_data.communications_retained_evidence_replay_index WHERE attachment_anchor_id=$1",
            )
            .bind(attachment_anchor_id.as_slice())
            .fetch_optional(&pool)
            .await
            .expect("read retained Communications evidence index");
            let mail = sqlx::query(
                "SELECT message_id FROM hermes_data.mail_retained_evidence_replay_index WHERE attachment_anchor_id=$1",
            )
            .bind(attachment_anchor_id.as_slice())
            .fetch_optional(&pool)
            .await
            .expect("read retained Mail evidence index");
            match (communications, mail) {
                (Some(communications), Some(mail)) => Some(RetainedPreviewEvidenceMessageIdsV1 {
                    communications: id16(&communications, "message_id"),
                    mail: id16(&mail, "message_id"),
                }),
                _ => None,
            }
        });
        if let Some(result) = result {
            return result;
        }
        assert!(
            Instant::now() < deadline,
            "retained Preview producer indexes were not populated"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

pub(super) fn wait_for_retained_preview_replay_terminal_v1(
    operation_id: [u8; 16],
) -> RetainedPreviewReplayDiagnosticsV1 {
    let runtime = tokio::runtime::Runtime::new().expect("retained Preview diagnostics runtime");
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let diagnostics = runtime.block_on(async {
            let pool = retained_preview_diagnostics_pool_v1().await;
            let operation = sqlx::query(
                "SELECT state,error FROM hermes_data.attachment_preview_evidence_replay_operations WHERE operation_id=$1",
            )
            .bind(operation_id.as_slice())
            .fetch_one(&pool)
            .await
            .expect("read retained Preview replay operation");
            let producer_results: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM hermes_data.attachment_preview_evidence_replay_result_inbox WHERE operation_id=$1",
            )
            .bind(operation_id.as_slice())
            .fetch_one(&pool)
            .await
            .expect("count retained Preview replay producer results");
            let communications_published_audits: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM hermes_data.communications_retained_evidence_replay_audit WHERE operation_id=$1 AND phase=2",
            )
            .bind(operation_id.as_slice())
            .fetch_one(&pool)
            .await
            .expect("count retained Communications replay audits");
            let mail_published_audits: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM hermes_data.mail_retained_evidence_replay_audit WHERE operation_id=$1 AND phase=2",
            )
            .bind(operation_id.as_slice())
            .fetch_one(&pool)
            .await
            .expect("count retained Mail replay audits");
            RetainedPreviewReplayDiagnosticsV1 {
                state: operation.try_get("state").expect("replay operation state"),
                error: operation.try_get("error").expect("replay operation error"),
                producer_results,
                communications_published_audits,
                mail_published_audits,
            }
        });
        if diagnostics.state == 3 || diagnostics.state == 4 || diagnostics.state == 5 {
            return diagnostics;
        }
        assert!(
            Instant::now() < deadline,
            "retained Preview replay operation did not become terminal"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn id16(row: &sqlx::postgres::PgRow, column: &str) -> [u8; 16] {
    row.try_get::<Vec<u8>, _>(column)
        .expect("retained evidence message identifier")
        .try_into()
        .expect("retained evidence message identifier length")
}

async fn retained_preview_diagnostics_pool_v1() -> sqlx::PgPool {
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
    PgPoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .expect("connect retained Preview replay diagnostics")
}
