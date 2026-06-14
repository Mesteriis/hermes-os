use sqlx::postgres::PgPool;

use crate::domains::decisions::DecisionStore;
use crate::domains::mail::messages::ProjectedMessage;
use crate::domains::tasks::candidates::TaskCandidateStore;

use super::errors::EmailSyncPipelineError;

#[derive(Default)]
pub(crate) struct MessageCandidateRefreshReport {
    pub(crate) refreshed_decision_candidates: usize,
    pub(crate) refreshed_task_candidates: usize,
}

pub(crate) async fn refresh_message_context_candidates(
    pool: &PgPool,
    messages: &[ProjectedMessage],
) -> Result<MessageCandidateRefreshReport, EmailSyncPipelineError> {
    let message_ids = messages
        .iter()
        .map(|message| message.message_id.clone())
        .collect::<Vec<_>>();
    if message_ids.is_empty() {
        return Ok(MessageCandidateRefreshReport::default());
    }

    let decision_store = DecisionStore::new(pool.clone());
    let task_candidate_store = TaskCandidateStore::new(pool.clone());

    Ok(MessageCandidateRefreshReport {
        refreshed_decision_candidates: decision_store
            .refresh_message_candidates_for_ids(&message_ids)
            .await?,
        refreshed_task_candidates: task_candidate_store
            .refresh_message_candidates_for_ids(&message_ids)
            .await?,
    })
}
