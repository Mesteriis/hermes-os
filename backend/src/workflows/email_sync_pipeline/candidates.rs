use sqlx::postgres::PgPool;

use crate::domains::communications::messages::ProjectedMessage;
use crate::workflows::review_inbox::{
    refresh_message_decisions_into_review, refresh_message_knowledge_candidates_into_review,
    refresh_message_task_candidates_into_review,
};

use super::errors::EmailSyncPipelineError;

#[derive(Default)]
pub(crate) struct MessageCandidateRefreshReport {
    pub(crate) refreshed_decision_candidates: usize,
    pub(crate) refreshed_knowledge_candidates: usize,
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

    Ok(MessageCandidateRefreshReport {
        refreshed_decision_candidates: refresh_message_decisions_into_review(pool, &message_ids)
            .await?,
        refreshed_knowledge_candidates: refresh_message_knowledge_candidates_into_review(
            pool, messages,
        )
        .await?,
        refreshed_task_candidates: refresh_message_task_candidates_into_review(pool, &message_ids)
            .await?,
    })
}
