use super::*;

#[derive(Serialize)]
pub(crate) struct PersonIdentityCandidateListResponse {
    pub(crate) items: Vec<PersonIdentityCandidate>,
}

#[derive(Deserialize)]
pub(crate) struct PersonIdentityReviewApiRequest {
    pub(crate) command_id: String,
    pub(crate) review_state: String,
}

impl PersonIdentityReviewApiRequest {
    pub(crate) fn into_command(
        self,
        identity_candidate_id: String,
        actor_id: String,
    ) -> Result<PersonIdentityReviewCommand, ApiError> {
        let command_id = validate_non_empty_person_identity_field("command_id", &self.command_id)?;
        let identity_candidate_id = validate_non_empty_person_identity_field(
            "identity_candidate_id",
            &identity_candidate_id,
        )?;
        let review_state = parse_person_identity_review_state(&self.review_state)?;

        Ok(PersonIdentityReviewCommand {
            command_id,
            identity_candidate_id,
            review_state,
            actor_id,
        })
    }
}

#[derive(Serialize)]
pub(crate) struct PersonIdentityReviewApiResponse {
    pub(crate) identity_candidate_id: String,
    pub(crate) review_state: String,
    pub(crate) event_id: String,
}

impl From<crate::domains::persons::identity::PersonIdentityReviewCommandResult>
    for PersonIdentityReviewApiResponse
{
    fn from(result: crate::domains::persons::identity::PersonIdentityReviewCommandResult) -> Self {
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
