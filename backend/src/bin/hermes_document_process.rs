use std::env;

use hermes_hub_backend::domains::documents::processing::{
    DocumentProcessingRunReport, DocumentProcessingStore,
};
use hermes_hub_backend::platform::config::AppConfig;
use hermes_hub_backend::platform::storage::Database;
use serde::Serialize;
use thiserror::Error;

const DEFAULT_LIMIT: i64 = 25;

#[derive(Debug, Serialize)]
struct DocumentProcessCommandReport {
    runner: DocumentProcessingRunReport,
    requested_limit: i64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    hermes_hub_backend::app::init_tracing();

    let config = AppConfig::from_env()?;
    let database_url = config
        .database_url()
        .ok_or(DocumentProcessCommandError::MissingDatabaseUrl)?;
    let database = Database::connect(Some(database_url)).await?;
    let pool = database
        .pool()
        .ok_or(DocumentProcessCommandError::MissingDatabaseUrl)?
        .clone();
    let store = DocumentProcessingStore::new(pool);

    let requested_limit = env::args()
        .nth(1)
        .as_deref()
        .map_or(Ok(DEFAULT_LIMIT), |value| {
            value.parse::<i64>().map_err(|_| {
                DocumentProcessCommandError::InvalidLimit(format!(
                    "limit argument must be integer: {value}"
                ))
            })
        })?;

    let runner = store.run_queued_jobs(requested_limit).await?;

    println!(
        "{}",
        serde_json::to_string_pretty(&DocumentProcessCommandReport {
            runner,
            requested_limit,
        })?
    );

    Ok(())
}

#[derive(Debug, Error)]
enum DocumentProcessCommandError {
    #[error("DATABASE_URL is required for document processing")]
    MissingDatabaseUrl,

    #[error("{0}")]
    InvalidLimit(String),
}
