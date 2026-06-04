use serde_json::json;
use thiserror::Error;

use crate::communications::{
    CommunicationIngestionError, CommunicationIngestionStore, NewRawCommunicationRecord,
};
use crate::email_sources::{FixtureEmailSourceError, parse_fixture_email_messages};

pub struct FixtureEmailImportRequest {
    pub account_id: String,
    pub import_batch_id: String,
    pub fixture_json: String,
}

impl FixtureEmailImportRequest {
    pub fn new(
        account_id: impl Into<String>,
        import_batch_id: impl Into<String>,
        fixture_json: impl Into<String>,
    ) -> Self {
        Self {
            account_id: account_id.into(),
            import_batch_id: import_batch_id.into(),
            fixture_json: fixture_json.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FixtureEmailImportReport {
    pub inserted_or_existing_records: usize,
}

pub async fn import_fixture_email_messages(
    store: &CommunicationIngestionStore,
    request: &FixtureEmailImportRequest,
) -> Result<FixtureEmailImportReport, FixtureEmailImportError> {
    let messages = parse_fixture_email_messages(&request.fixture_json)?;
    let mut inserted_or_existing_records = 0;

    for message in messages {
        store
            .record_raw_source(
                &NewRawCommunicationRecord::new(
                    format!("raw:{}:{}", request.account_id, message.provider_record_id),
                    &request.account_id,
                    "email_message",
                    &message.provider_record_id,
                    &message.source_fingerprint,
                    &request.import_batch_id,
                    json!({
                        "subject": message.subject,
                        "from": message.from,
                        "to": message.to,
                        "body_text": message.body_text
                    }),
                )
                .occurred_at(message.sent_at.unwrap_or_else(chrono::Utc::now))
                .provenance(json!({"source": "fixture_email"})),
            )
            .await?;
        inserted_or_existing_records += 1;
    }

    Ok(FixtureEmailImportReport {
        inserted_or_existing_records,
    })
}

#[derive(Debug, Error)]
pub enum FixtureEmailImportError {
    #[error(transparent)]
    Source(#[from] FixtureEmailSourceError),

    #[error(transparent)]
    Communication(#[from] CommunicationIngestionError),
}
