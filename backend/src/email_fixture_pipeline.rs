use serde::Serialize;
use serde_json::json;
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::communications::{
    CommunicationIngestionError, CommunicationIngestionStore, EmailProviderKind, NewProviderAccount,
};
use crate::email_import::{
    FixtureEmailImportError, FixtureEmailImportRequest, import_fixture_email_messages_with_records,
};
use crate::graph::{GraphStore, GraphStoreError, GraphSummary};
use crate::graph_projection::{
    GraphProjectionError, GraphProjectionReport, GraphProjectionService,
};
use crate::messages::{MessageProjectionError, MessageProjectionStore, project_raw_email_message};
use crate::persons::{
    PersonProjectionError, PersonProjectionStore, upsert_persons_from_message_participants,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmailFixturePipelineRequest {
    pub account_id: String,
    pub display_name: String,
    pub external_account_id: String,
    pub provider_kind: EmailProviderKind,
    pub import_batch_id: String,
    pub fixture_json: String,
}

impl EmailFixturePipelineRequest {
    pub fn new(
        account_id: impl Into<String>,
        display_name: impl Into<String>,
        external_account_id: impl Into<String>,
        provider_kind: EmailProviderKind,
        import_batch_id: impl Into<String>,
        fixture_json: impl Into<String>,
    ) -> Self {
        Self {
            account_id: account_id.into(),
            display_name: display_name.into(),
            external_account_id: external_account_id.into(),
            provider_kind,
            import_batch_id: import_batch_id.into(),
            fixture_json: fixture_json.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct EmailFixtureImportPipelineReport {
    pub account_id: String,
    pub import_batch_id: String,
    pub provider_kind: EmailProviderKind,
    pub imported_records: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct EmailFixtureProjectionPipelineReport {
    pub account_id: String,
    pub import_batch_id: String,
    pub provider_kind: EmailProviderKind,
    pub imported_records: usize,
    pub projected_messages: usize,
    pub upserted_persons: usize,
    pub graph_projection: GraphProjectionReport,
    pub graph_summary: GraphSummary,
    pub total_graph_nodes: i64,
    pub total_graph_edges: i64,
}

pub async fn import_fixture_email_messages_for_dev(
    pool: PgPool,
    request: &EmailFixturePipelineRequest,
) -> Result<EmailFixtureImportPipelineReport, EmailFixturePipelineError> {
    let communication_store = CommunicationIngestionStore::new(pool);
    upsert_fixture_provider_account(&communication_store, request).await?;
    let import_report = import_fixture_email_messages_with_records(
        &communication_store,
        &FixtureEmailImportRequest::new(
            &request.account_id,
            &request.import_batch_id,
            &request.fixture_json,
        ),
    )
    .await?;

    Ok(EmailFixtureImportPipelineReport {
        account_id: request.account_id.clone(),
        import_batch_id: request.import_batch_id.clone(),
        provider_kind: request.provider_kind,
        imported_records: import_report.inserted_or_existing_records,
    })
}

pub async fn project_fixture_email_messages(
    pool: PgPool,
    request: &EmailFixturePipelineRequest,
) -> Result<EmailFixtureProjectionPipelineReport, EmailFixturePipelineError> {
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    upsert_fixture_provider_account(&communication_store, request).await?;
    let import_report = import_fixture_email_messages_with_records(
        &communication_store,
        &FixtureEmailImportRequest::new(
            &request.account_id,
            &request.import_batch_id,
            &request.fixture_json,
        ),
    )
    .await?;

    let message_store = MessageProjectionStore::new(pool.clone());
    let person_store = PersonProjectionStore::new(pool.clone());
    let mut projected_messages = 0;
    let mut participants = Vec::new();
    for raw_record in &import_report.raw_records {
        let message = project_raw_email_message(&message_store, raw_record).await?;
        participants.push(message.sender.clone());
        participants.extend(message.recipients.clone());
        projected_messages += 1;
    }
    let persons = upsert_persons_from_message_participants(&person_store, &participants).await?;

    let graph_projection = GraphProjectionService::new(pool.clone())
        .project_from_v1()
        .await?;
    let graph_summary = GraphStore::new(pool).summary().await?;
    let total_graph_nodes = graph_summary
        .node_counts
        .iter()
        .map(|count| count.count)
        .sum();
    let total_graph_edges = graph_summary
        .edge_counts
        .iter()
        .map(|count| count.count)
        .sum();

    Ok(EmailFixtureProjectionPipelineReport {
        account_id: request.account_id.clone(),
        import_batch_id: request.import_batch_id.clone(),
        provider_kind: request.provider_kind,
        imported_records: import_report.inserted_or_existing_records,
        projected_messages,
        upserted_persons: persons.len(),
        graph_projection,
        graph_summary,
        total_graph_nodes,
        total_graph_edges,
    })
}

async fn upsert_fixture_provider_account(
    communication_store: &CommunicationIngestionStore,
    request: &EmailFixturePipelineRequest,
) -> Result<(), CommunicationIngestionError> {
    let account = NewProviderAccount::new(
        &request.account_id,
        request.provider_kind,
        &request.display_name,
        &request.external_account_id,
    )
    .config(provider_config(request.provider_kind));
    communication_store
        .upsert_provider_account(&account)
        .await?;
    Ok(())
}

fn provider_config(provider_kind: EmailProviderKind) -> serde_json::Value {
    match provider_kind {
        EmailProviderKind::Gmail => json!({"history_stream_id": "gmail:fixture"}),
        EmailProviderKind::Icloud => {
            json!({"host": "imap.mail.me.com", "port": 993, "tls": true, "mailbox": "INBOX"})
        }
        EmailProviderKind::Imap => {
            json!({"host": "localhost", "port": 993, "tls": true, "mailbox": "INBOX"})
        }
        EmailProviderKind::TelegramUser
        | EmailProviderKind::TelegramBot
        | EmailProviderKind::WhatsappWeb => json!({}),
    }
}

#[derive(Debug, Error)]
pub enum EmailFixturePipelineError {
    #[error(transparent)]
    Communication(#[from] CommunicationIngestionError),

    #[error(transparent)]
    Import(#[from] FixtureEmailImportError),

    #[error(transparent)]
    Message(#[from] MessageProjectionError),

    #[error(transparent)]
    Contact(#[from] PersonProjectionError),

    #[error(transparent)]
    GraphProjection(#[from] GraphProjectionError),

    #[error(transparent)]
    GraphStore(#[from] GraphStoreError),
}
