use crate::domains::mail::messages::{MessageProjectionStore, ProjectedMessage};
use crate::engines::search::{SearchDocument, SearchIndex, SearchResult};

pub fn project_message_to_search_document(message: &ProjectedMessage) -> SearchDocument {
    SearchDocument {
        object_id: message.message_id.clone(),
        object_kind: "communication_message".to_owned(),
        title: format!("[{}] {}", message.sender, message.subject),
        body: message.body_text.clone(),
    }
}

pub async fn index_messages(
    index: &SearchIndex,
    store: &MessageProjectionStore,
    limit: i64,
) -> Result<usize, IndexEmailError> {
    let messages = store.recent_messages(limit).await?;
    let count = messages.len();
    for summary in &messages {
        let doc = project_message_to_search_document(&summary.message);
        index.upsert_document(&doc)?;
    }
    Ok(count)
}

pub fn search_emails(
    index: &SearchIndex,
    query: &str,
    limit: usize,
) -> Result<Vec<SearchResult>, IndexEmailError> {
    let results = index.search(query, limit)?;
    Ok(results
        .into_iter()
        .filter(|r| r.object_kind == "communication_message")
        .collect())
}

#[derive(Debug, thiserror::Error)]
pub enum IndexEmailError {
    #[error(transparent)]
    Search(#[from] crate::engines::search::SearchError),
    #[error(transparent)]
    Messages(#[from] crate::domains::mail::messages::MessageProjectionError),
}
