use super::*;
use crate::app::api_support::query_parsing::documents::validate_non_empty_document_processing_field;
use crate::app::api_support::query_parsing::personas::{
    parse_persona_identity_review_state, validate_non_empty_persona_identity_field,
};
use crate::app::api_support::query_parsing::projects::{
    parse_project_link_review_state, parse_project_link_target_kind,
    validate_non_empty_project_link_field,
};
use crate::app::api_support::query_parsing::tasks::{
    parse_task_candidate_review_state, validate_non_empty_task_candidate_field,
};

#[derive(Serialize)]
pub(crate) struct PersonaIdentityCandidateListResponse {
    pub(crate) items: Vec<PersonaIdentityCandidateApiResponse>,
}

impl From<Vec<PersonaIdentityCandidate>> for PersonaIdentityCandidateListResponse {
    fn from(items: Vec<PersonaIdentityCandidate>) -> Self {
        Self {
            items: items.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Serialize)]
pub(crate) struct PersonaIdentityDetailResponse {
    pub(crate) items: Vec<PersonaIdentityCandidateApiResponse>,
}

impl From<PersonaIdentityDetail> for PersonaIdentityDetailResponse {
    fn from(detail: PersonaIdentityDetail) -> Self {
        Self {
            items: detail.items.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Serialize)]
pub(crate) struct PersonaIdentityCandidateApiResponse {
    pub(crate) identity_candidate_id: String,
    pub(crate) candidate_kind: String,
    pub(crate) left_persona_id: String,
    pub(crate) right_persona_id: Option<String>,
    pub(crate) email_address: Option<String>,
    pub(crate) evidence_summary: String,
    pub(crate) confidence: f64,
    pub(crate) review_state: String,
    pub(crate) generated_at: DateTime<Utc>,
    pub(crate) reviewed_at: Option<DateTime<Utc>>,
    pub(crate) updated_at: DateTime<Utc>,
}

impl From<PersonaIdentityCandidate> for PersonaIdentityCandidateApiResponse {
    fn from(candidate: PersonaIdentityCandidate) -> Self {
        Self {
            identity_candidate_id: candidate.identity_candidate_id,
            candidate_kind: candidate.candidate_kind,
            left_persona_id: candidate.left_persona_id,
            right_persona_id: candidate.right_persona_id,
            email_address: candidate.email_address,
            evidence_summary: candidate.evidence_summary,
            confidence: candidate.confidence,
            review_state: candidate.review_state,
            generated_at: candidate.generated_at,
            reviewed_at: candidate.reviewed_at,
            updated_at: candidate.updated_at,
        }
    }
}

#[derive(Deserialize)]
pub(crate) struct PersonaIdentityReviewApiRequest {
    pub(crate) command_id: String,
    pub(crate) review_state: String,
}

impl PersonaIdentityReviewApiRequest {
    pub(crate) fn into_command(
        self,
        identity_candidate_id: String,
        actor_id: String,
    ) -> Result<PersonaIdentityReviewCommand, ApiError> {
        let command_id = validate_non_empty_persona_identity_field("command_id", &self.command_id)?;
        let identity_candidate_id = validate_non_empty_persona_identity_field(
            "identity_candidate_id",
            &identity_candidate_id,
        )?;
        let review_state = parse_persona_identity_review_state(&self.review_state)?;

        Ok(PersonaIdentityReviewCommand {
            command_id,
            identity_candidate_id,
            review_state,
            actor_id,
        })
    }
}

#[derive(Serialize)]
pub(crate) struct PersonaIdentityReviewApiResponse {
    pub(crate) identity_candidate_id: String,
    pub(crate) review_state: String,
    pub(crate) event_id: String,
}

impl From<crate::domains::personas::identity::PersonaIdentityReviewCommandResult>
    for PersonaIdentityReviewApiResponse
{
    fn from(
        result: crate::domains::personas::identity::PersonaIdentityReviewCommandResult,
    ) -> Self {
        Self {
            identity_candidate_id: result.identity_candidate_id,
            review_state: result.review_state.as_str().to_owned(),
            event_id: result.event_id,
        }
    }
}

#[derive(Serialize)]
pub(crate) struct DocumentProcessingJobsResponse {
    pub(crate) items: Vec<DocumentProcessingJob>,
}

#[derive(Deserialize)]
pub(crate) struct DocumentProcessingRetryApiRequest {
    pub(crate) command_id: String,
}

impl DocumentProcessingRetryApiRequest {
    pub(crate) fn into_command(
        self,
        job_id: String,
        actor_id: String,
    ) -> Result<DocumentProcessingRetryCommand, ApiError> {
        let command_id =
            validate_non_empty_document_processing_field("command_id", &self.command_id)?;
        let job_id = validate_non_empty_document_processing_field("job_id", &job_id)?;

        Ok(DocumentProcessingRetryCommand {
            command_id,
            job_id,
            actor_id,
        })
    }
}

#[derive(Serialize)]
pub(crate) struct DocumentProcessingRetryApiResponse {
    pub(crate) job_id: String,
    pub(crate) status: DocumentProcessingStatus,
    pub(crate) event_id: String,
}

impl From<DocumentProcessingRetryCommandResult> for DocumentProcessingRetryApiResponse {
    fn from(result: DocumentProcessingRetryCommandResult) -> Self {
        Self {
            job_id: result.job_id,
            status: result.status,
            event_id: result.event_id,
        }
    }
}

#[derive(Deserialize)]
pub(crate) struct TaskCandidateReviewApiRequest {
    pub(crate) command_id: String,
    pub(crate) review_state: String,
}

impl TaskCandidateReviewApiRequest {
    pub(crate) fn into_command(
        self,
        task_candidate_id: String,
        actor_id: String,
    ) -> Result<TaskCandidateReviewCommand, ApiError> {
        let command_id = validate_non_empty_task_candidate_field("command_id", &self.command_id)?;
        let task_candidate_id =
            validate_non_empty_task_candidate_field("task_candidate_id", &task_candidate_id)?;
        let review_state = parse_task_candidate_review_state(&self.review_state)?;

        Ok(TaskCandidateReviewCommand {
            command_id,
            task_candidate_id,
            review_state,
            actor_id,
        })
    }
}

#[derive(Serialize)]
pub(crate) struct TaskCandidateReviewApiResponse {
    pub(crate) task_candidate_id: String,
    pub(crate) review_state: String,
    pub(crate) event_id: String,
}

impl From<crate::domains::tasks::candidates::TaskCandidateReviewCommandResult>
    for TaskCandidateReviewApiResponse
{
    fn from(result: crate::domains::tasks::candidates::TaskCandidateReviewCommandResult) -> Self {
        Self {
            task_candidate_id: result.task_candidate_id,
            review_state: result.review_state.as_str().to_owned(),
            event_id: result.event_id,
        }
    }
}

#[derive(Deserialize)]
pub(crate) struct ProjectLinkReviewApiRequest {
    pub(crate) command_id: String,
    pub(crate) target_kind: String,
    pub(crate) target_id: String,
    pub(crate) review_state: String,
}

impl ProjectLinkReviewApiRequest {
    pub(crate) fn into_command(
        self,
        project_id: String,
        actor_id: String,
    ) -> Result<ProjectLinkReviewCommand, ApiError> {
        let command_id = validate_non_empty_project_link_field("command_id", &self.command_id)?;
        let project_id = validate_non_empty_project_link_field("project_id", &project_id)?;
        let target_id = validate_non_empty_project_link_field("target_id", &self.target_id)?;
        let target_kind = parse_project_link_target_kind(&self.target_kind)?;
        let review_state = parse_project_link_review_state(&self.review_state)?;

        Ok(ProjectLinkReviewCommand {
            command_id,
            project_id,
            target_kind,
            target_id,
            review_state,
            actor_id,
        })
    }
}

#[derive(Serialize)]
pub(crate) struct ProjectLinkReviewApiResponse {
    pub(crate) project_id: String,
    pub(crate) target_kind: String,
    pub(crate) target_id: String,
    pub(crate) review_state: String,
    pub(crate) event_id: String,
}

impl From<crate::domains::projects::link_reviews::ProjectLinkReviewCommandResult>
    for ProjectLinkReviewApiResponse
{
    fn from(
        result: crate::domains::projects::link_reviews::ProjectLinkReviewCommandResult,
    ) -> Self {
        Self {
            project_id: result.project_id,
            target_kind: result.target_kind.as_str().to_owned(),
            target_id: result.target_id,
            review_state: result.review_state.as_str().to_owned(),
            event_id: result.event_id,
        }
    }
}
