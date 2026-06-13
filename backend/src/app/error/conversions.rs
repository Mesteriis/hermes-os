use super::types::ApiError;
use crate::ai::control_center::AiControlCenterError;
use crate::ai::core::AiError;
use crate::domains::calendar::brain::CalendarBrainError;
use crate::domains::calendar::core::CalendarCoreError;
use crate::domains::calendar::events::CalendarError;
use crate::domains::calendar::health::CalendarHealthError;
use crate::domains::calendar::meetings::MeetingsError;
use crate::domains::calendar::reminders::ReminderError;
use crate::domains::calendar::rules::CalendarRuleError;
use crate::domains::calendar::scheduling::SchedulingError;
use crate::domains::decisions::DecisionStoreError;
use crate::domains::documents::processing::DocumentProcessingError;
use crate::domains::mail::accounts::EmailAccountSetupError;
use crate::domains::mail::core::CommunicationIngestionError;
use crate::domains::mail::messages::MessageProjectionError;
use crate::domains::mail::storage::MailStorageError;
use crate::domains::obligations::ObligationStoreError;
use crate::domains::organizations::api::OrganizationError;
use crate::domains::persons::api::PersonProjectionError;
use crate::domains::persons::core::PersonCoreError;
use crate::domains::persons::identity::PersonIdentityError;
use crate::domains::persons::memory::PersonMemoryError;
use crate::domains::projects::core::ProjectStoreError;
use crate::domains::projects::link_reviews::ProjectLinkReviewError;
use crate::domains::relationships::RelationshipStoreError;
use crate::domains::tasks::api::TaskError;
use crate::domains::tasks::brain::TaskBrainError;
use crate::domains::tasks::candidates::TaskCandidateError;
use crate::domains::tasks::core::TaskCoreError;
use crate::domains::tasks::health::TaskHealthError;
use crate::domains::tasks::rules::TaskRuleError;
use crate::engines::automation::AutomationError;
use crate::engines::consistency::ConsistencyError;
use crate::integrations::telegram::client::TelegramError;
use crate::integrations::whatsapp::client::WhatsappWebError;
use crate::platform::audit::ApiAuditError;
use crate::platform::calls::CallError;
use crate::platform::events::{EventEnvelopeError, EventStoreError};
use crate::platform::settings::SettingsError;
use crate::vault::HostVaultError;
use crate::workflows::email_intelligence::EmailIntelligenceError;

impl From<EventEnvelopeError> for ApiError {
    fn from(error: EventEnvelopeError) -> Self {
        Self::InvalidEnvelope(error)
    }
}

impl From<EventStoreError> for ApiError {
    fn from(error: EventStoreError) -> Self {
        Self::Store(error)
    }
}

impl From<crate::domains::graph::core::GraphStoreError> for ApiError {
    fn from(error: crate::domains::graph::core::GraphStoreError) -> Self {
        Self::Graph(error)
    }
}

impl From<ProjectLinkReviewError> for ApiError {
    fn from(error: ProjectLinkReviewError) -> Self {
        match error {
            ProjectLinkReviewError::ProjectNotFound | ProjectLinkReviewError::TargetNotFound => {
                Self::ProjectLinkTargetNotFound
            }
            _ => Self::ProjectLinkReview(error),
        }
    }
}

impl From<TaskCandidateError> for ApiError {
    fn from(error: TaskCandidateError) -> Self {
        match error {
            TaskCandidateError::TaskCandidateNotFound => Self::TaskCandidateNotFound,
            _ => Self::TaskCandidate(error),
        }
    }
}

impl From<ObligationStoreError> for ApiError {
    fn from(error: ObligationStoreError) -> Self {
        match error {
            ObligationStoreError::ObligationNotFound => Self::ObligationNotFound,
            ObligationStoreError::UnknownEntityKind(_) => Self::InvalidObligationQuery(
                "entity_kind must be persona, organization, project, communication, document, task, event, decision, obligation, or knowledge",
            ),
            ObligationStoreError::UnknownReviewState(_) => Self::InvalidObligationReview(
                "review_state must be suggested, user_confirmed, or user_rejected",
            ),
            _ => Self::Obligation(error),
        }
    }
}

impl From<DecisionStoreError> for ApiError {
    fn from(error: DecisionStoreError) -> Self {
        match error {
            DecisionStoreError::DecisionNotFound => Self::DecisionNotFound,
            DecisionStoreError::UnknownEntityKind(_) => Self::InvalidDecisionQuery(
                "entity_kind must be persona, organization, project, communication, document, task, event, decision, obligation, or knowledge",
            ),
            DecisionStoreError::UnknownReviewState(_) => Self::InvalidDecisionReview(
                "review_state must be suggested, user_confirmed, or user_rejected",
            ),
            _ => Self::Decision(error),
        }
    }
}

impl From<RelationshipStoreError> for ApiError {
    fn from(error: RelationshipStoreError) -> Self {
        match error {
            RelationshipStoreError::RelationshipNotFound => Self::RelationshipNotFound,
            RelationshipStoreError::UnknownEntityKind(_) => Self::InvalidRelationshipQuery(
                "entity_kind must be persona, organization, project, communication, document, task, event, decision, obligation, or knowledge",
            ),
            RelationshipStoreError::UnknownReviewState(_) => Self::InvalidRelationshipReview(
                "review_state must be suggested, system_accepted, user_confirmed, or user_rejected",
            ),
            _ => Self::Relationship(error),
        }
    }
}

impl From<ConsistencyError> for ApiError {
    fn from(error: ConsistencyError) -> Self {
        match error {
            ConsistencyError::ObservationNotFound(_) => Self::ContradictionObservationNotFound,
            ConsistencyError::UnknownReviewState(_) => Self::InvalidContradictionReview(
                "review_state must be suggested, user_confirmed, or user_rejected",
            ),
            _ => Self::Consistency(error),
        }
    }
}

impl From<AiError> for ApiError {
    fn from(error: AiError) -> Self {
        match error {
            AiError::RunNotFound => Self::AiRunNotFound,
            _ => Self::Ai(error),
        }
    }
}

impl From<AiControlCenterError> for ApiError {
    fn from(error: AiControlCenterError) -> Self {
        Self::AiControlCenter(error)
    }
}

impl From<TelegramError> for ApiError {
    fn from(error: TelegramError) -> Self {
        Self::Telegram(error)
    }
}

impl From<WhatsappWebError> for ApiError {
    fn from(error: WhatsappWebError) -> Self {
        Self::WhatsappWeb(error)
    }
}

impl From<AutomationError> for ApiError {
    fn from(error: AutomationError) -> Self {
        Self::Automation(error)
    }
}

impl From<CallError> for ApiError {
    fn from(error: CallError) -> Self {
        Self::Call(error)
    }
}

impl From<crate::integrations::ollama::client::OllamaError> for ApiError {
    fn from(error: crate::integrations::ollama::client::OllamaError) -> Self {
        Self::Ai(AiError::Runtime(
            crate::integrations::ai_runtime::AiRuntimeError::Ollama(error),
        ))
    }
}

impl From<crate::integrations::omniroute::client::OmniRouteError> for ApiError {
    fn from(error: crate::integrations::omniroute::client::OmniRouteError) -> Self {
        Self::Ai(AiError::Runtime(
            crate::integrations::ai_runtime::AiRuntimeError::OmniRoute(error),
        ))
    }
}

impl From<crate::integrations::ai_runtime::AiRuntimeError> for ApiError {
    fn from(error: crate::integrations::ai_runtime::AiRuntimeError) -> Self {
        Self::Ai(AiError::Runtime(error))
    }
}

impl From<PersonIdentityError> for ApiError {
    fn from(error: PersonIdentityError) -> Self {
        match error {
            PersonIdentityError::IdentityCandidateNotFound => Self::PersonIdentityNotFound,
            PersonIdentityError::InvalidLimit | PersonIdentityError::InvalidReviewState(_) => {
                Self::InvalidPersonIdentityReview(
                    "review_state or limit must be valid for person identity candidates",
                )
            }
            PersonIdentityError::InvalidPayload(_)
            | PersonIdentityError::MissingPayloadField(_)
            | PersonIdentityError::MissingActorId => {
                Self::InvalidPersonIdentityReview("invalid identity candidate review payload")
            }
            _ => Self::PersonIdentity(error),
        }
    }
}

impl From<PersonProjectionError> for ApiError {
    fn from(error: PersonProjectionError) -> Self {
        Self::PersonProjection(error)
    }
}

impl From<DocumentProcessingError> for ApiError {
    fn from(error: DocumentProcessingError) -> Self {
        Self::DocumentProcessing(error)
    }
}

impl From<SettingsError> for ApiError {
    fn from(error: SettingsError) -> Self {
        match error {
            SettingsError::SettingNotFound { .. } => Self::SettingNotFound,
            _ => Self::Settings(error),
        }
    }
}

impl From<CommunicationIngestionError> for ApiError {
    fn from(error: CommunicationIngestionError) -> Self {
        Self::CommunicationIngestion(error)
    }
}

impl From<ProjectStoreError> for ApiError {
    fn from(error: ProjectStoreError) -> Self {
        Self::Projects(error)
    }
}

impl From<MessageProjectionError> for ApiError {
    fn from(error: MessageProjectionError) -> Self {
        Self::Messages(error)
    }
}

impl From<MailStorageError> for ApiError {
    fn from(error: MailStorageError) -> Self {
        Self::MailStorage(error)
    }
}

impl From<ApiAuditError> for ApiError {
    fn from(error: ApiAuditError) -> Self {
        Self::Audit(error)
    }
}

impl From<crate::domains::mail::threads::EmailThreadError> for ApiError {
    fn from(error: crate::domains::mail::threads::EmailThreadError) -> Self {
        tracing::error!(error = %error, "email thread operation failed");
        ApiError::InvalidCommunicationQuery("email thread operation failed")
    }
}

impl From<EmailIntelligenceError> for ApiError {
    fn from(error: EmailIntelligenceError) -> Self {
        match error {
            EmailIntelligenceError::ParseError(_msg) => {
                ApiError::InvalidCommunicationQuery("failed to parse AI analysis result")
            }
            _ => {
                tracing::error!(error = %error, "email intelligence operation failed");
                ApiError::InvalidCommunicationQuery("email intelligence operation failed")
            }
        }
    }
}

impl From<crate::engines::search::SearchError> for ApiError {
    fn from(error: crate::engines::search::SearchError) -> Self {
        tracing::error!(error = %error, "search operation failed");
        ApiError::InvalidCommunicationQuery("search operation failed")
    }
}

impl From<crate::domains::mail::drafts::EmailDraftError> for ApiError {
    fn from(error: crate::domains::mail::drafts::EmailDraftError) -> Self {
        tracing::error!(error = %error, "email draft operation failed");
        ApiError::InvalidCommunicationQuery("email draft operation failed")
    }
}

impl From<crate::domains::mail::finance::EmailFinanceError> for ApiError {
    fn from(error: crate::domains::mail::finance::EmailFinanceError) -> Self {
        tracing::error!(error = %error, "email finance operation failed");
        ApiError::InvalidCommunicationQuery("email finance operation failed")
    }
}

impl From<crate::domains::mail::analytics::EmailAnalyticsError> for ApiError {
    fn from(error: crate::domains::mail::analytics::EmailAnalyticsError) -> Self {
        tracing::error!(error = %error, "email analytics operation failed");
        ApiError::InvalidCommunicationQuery("email analytics operation failed")
    }
}

impl From<crate::domains::mail::personas::EmailPersonaError> for ApiError {
    fn from(error: crate::domains::mail::personas::EmailPersonaError) -> Self {
        tracing::error!(error = %error, "email persona operation failed");
        ApiError::InvalidCommunicationQuery("email persona operation failed")
    }
}

impl From<crate::domains::mail::search::IndexEmailError> for ApiError {
    fn from(error: crate::domains::mail::search::IndexEmailError) -> Self {
        tracing::error!(error = %error, "email search operation failed");
        ApiError::InvalidCommunicationQuery("email search operation failed")
    }
}

impl From<crate::domains::mail::flags::MessageFlagsError> for ApiError {
    fn from(error: crate::domains::mail::flags::MessageFlagsError) -> Self {
        tracing::error!(error = %error, "message flags operation failed");
        ApiError::InvalidCommunicationQuery("message flags operation failed")
    }
}

impl From<crate::domains::mail::subscriptions::SubscriptionError> for ApiError {
    fn from(error: crate::domains::mail::subscriptions::SubscriptionError) -> Self {
        tracing::error!(error = %error, "subscriptions operation failed");
        ApiError::InvalidCommunicationQuery("subscriptions operation failed")
    }
}

impl From<crate::domains::mail::attachment_dedup::AttachmentDedupError> for ApiError {
    fn from(error: crate::domains::mail::attachment_dedup::AttachmentDedupError) -> Self {
        tracing::error!(error = %error, "attachment dedup operation failed");
        ApiError::InvalidCommunicationQuery("attachment dedup operation failed")
    }
}

impl From<crate::domains::mail::legal::LegalDocumentError> for ApiError {
    fn from(error: crate::domains::mail::legal::LegalDocumentError) -> Self {
        tracing::error!(error = %error, "legal document operation failed");
        ApiError::InvalidCommunicationQuery("legal document operation failed")
    }
}

impl From<crate::domains::mail::export::EmailExportError> for ApiError {
    fn from(error: crate::domains::mail::export::EmailExportError) -> Self {
        match error {
            crate::domains::mail::export::EmailExportError::NotFound => {
                ApiError::CommunicationMessageNotFound
            }
            _ => {
                tracing::error!(error = %error, "email export failed");
                ApiError::InvalidCommunicationQuery("email export failed")
            }
        }
    }
}

impl From<crate::domains::mail::send::EmailSendError> for ApiError {
    fn from(error: crate::domains::mail::send::EmailSendError) -> Self {
        tracing::error!(error = %error, "email send failed");
        ApiError::InvalidCommunicationQuery("email send failed")
    }
}
impl From<crate::domains::mail::imap_write::ImapWriteError> for ApiError {
    fn from(error: crate::domains::mail::imap_write::ImapWriteError) -> Self {
        tracing::error!(error = %error, "IMAP write operation failed");
        ApiError::InvalidCommunicationQuery("IMAP write operation failed")
    }
}
impl From<crate::domains::mail::signatures::CertificateError> for ApiError {
    fn from(error: crate::domains::mail::signatures::CertificateError) -> Self {
        tracing::error!(error = %error, "certificate operation failed");
        ApiError::InvalidCommunicationQuery("certificate operation failed")
    }
}
impl From<crate::domains::mail::multilingual::MultilingualError> for ApiError {
    fn from(error: crate::domains::mail::multilingual::MultilingualError) -> Self {
        tracing::error!(error = %error, "multilingual operation failed");
        ApiError::InvalidCommunicationQuery("multilingual operation failed")
    }
}
impl From<crate::domains::mail::ai_reply::AiReplyError> for ApiError {
    fn from(error: crate::domains::mail::ai_reply::AiReplyError) -> Self {
        tracing::error!(error = %error, "AI reply generation failed");
        ApiError::InvalidCommunicationQuery("AI reply generation failed")
    }
}
impl From<crate::domains::mail::extract::ExtractError> for ApiError {
    fn from(error: crate::domains::mail::extract::ExtractError) -> Self {
        tracing::error!(error = %error, "extract failed");
        ApiError::InvalidCommunicationQuery("extract failed")
    }
}
impl From<crate::domains::persons::enrichment::PersonEnrichmentError> for ApiError {
    fn from(error: crate::domains::persons::enrichment::PersonEnrichmentError) -> Self {
        match error {
            crate::domains::persons::enrichment::PersonEnrichmentError::NotFound => {
                ApiError::PersonIdentityNotFound
            }
            _ => {
                tracing::error!(error = %error, "person enrichment failed");
                ApiError::InvalidCommunicationQuery("person enrichment failed")
            }
        }
    }
}
impl From<PersonMemoryError> for ApiError {
    fn from(error: PersonMemoryError) -> Self {
        match error {
            PersonMemoryError::NotFound => ApiError::PersonIdentityNotFound,
            _ => {
                tracing::error!(error = %error, "person memory operation failed");
                ApiError::InvalidCommunicationQuery("person memory operation failed")
            }
        }
    }
}

impl From<PersonCoreError> for ApiError {
    fn from(error: PersonCoreError) -> Self {
        match error {
            PersonCoreError::IdentityNotFound | PersonCoreError::PersonaNotFound => {
                ApiError::PersonIdentityNotFound
            }
            _ => {
                tracing::error!(error = %error, "person core operation failed");
                ApiError::InvalidCommunicationQuery("person core operation failed")
            }
        }
    }
}

impl From<crate::domains::organizations::core::OrgCoreError> for ApiError {
    fn from(error: crate::domains::organizations::core::OrgCoreError) -> Self {
        tracing::error!(error = %error, "org core operation failed");
        ApiError::InvalidCommunicationQuery("org core operation failed")
    }
}
impl From<crate::domains::organizations::memory::OrgMemoryError> for ApiError {
    fn from(error: crate::domains::organizations::memory::OrgMemoryError) -> Self {
        tracing::error!(error = %error, "org memory operation failed");
        ApiError::InvalidCommunicationQuery("org memory operation failed")
    }
}
impl From<crate::domains::organizations::workflows::OrgWorkflowError> for ApiError {
    fn from(error: crate::domains::organizations::workflows::OrgWorkflowError) -> Self {
        tracing::error!(error = %error, "org workflow operation failed");
        ApiError::InvalidCommunicationQuery("org workflow operation failed")
    }
}
impl From<crate::domains::organizations::finance::OrgFinanceError> for ApiError {
    fn from(error: crate::domains::organizations::finance::OrgFinanceError) -> Self {
        tracing::error!(error = %error, "org finance operation failed");
        ApiError::InvalidCommunicationQuery("org finance operation failed")
    }
}
impl From<crate::domains::organizations::enrichment::OrgEnrichmentError> for ApiError {
    fn from(error: crate::domains::organizations::enrichment::OrgEnrichmentError) -> Self {
        tracing::error!(error = %error, "org enrichment operation failed");
        ApiError::InvalidCommunicationQuery("org enrichment operation failed")
    }
}
impl From<crate::domains::organizations::health::OrgHealthError> for ApiError {
    fn from(error: crate::domains::organizations::health::OrgHealthError) -> Self {
        tracing::error!(error = %error, "org health operation failed");
        ApiError::InvalidCommunicationQuery("org health operation failed")
    }
}
impl From<crate::domains::organizations::investigator::InvestigatorError> for ApiError {
    fn from(error: crate::domains::organizations::investigator::InvestigatorError) -> Self {
        match error {
            crate::domains::organizations::investigator::InvestigatorError::NotFound => {
                ApiError::NotFound
            }
            _ => {
                tracing::error!(error = %error, "investigator operation failed");
                ApiError::InvalidCommunicationQuery("investigator operation failed")
            }
        }
    }
}

impl From<CalendarCoreError> for ApiError {
    fn from(error: CalendarCoreError) -> Self {
        match error {
            CalendarCoreError::NotFound => ApiError::NotFound,
            _ => {
                tracing::error!(error = %error, "calendar core operation failed");
                ApiError::InvalidCommunicationQuery("calendar core operation failed")
            }
        }
    }
}
impl From<MeetingsError> for ApiError {
    fn from(error: MeetingsError) -> Self {
        match error {
            MeetingsError::NotFound => ApiError::NotFound,
            _ => {
                tracing::error!(error = %error, "meetings operation failed");
                ApiError::InvalidCommunicationQuery("meetings operation failed")
            }
        }
    }
}
impl From<SchedulingError> for ApiError {
    fn from(error: SchedulingError) -> Self {
        match error {
            SchedulingError::NotFound => ApiError::NotFound,
            _ => {
                tracing::error!(error = %error, "scheduling operation failed");
                ApiError::InvalidCommunicationQuery("scheduling operation failed")
            }
        }
    }
}
impl From<CalendarHealthError> for ApiError {
    fn from(error: CalendarHealthError) -> Self {
        tracing::error!(error = %error, "calendar health operation failed");
        ApiError::InvalidCommunicationQuery("calendar health operation failed")
    }
}
impl From<CalendarBrainError> for ApiError {
    fn from(error: CalendarBrainError) -> Self {
        match error {
            CalendarBrainError::NotFound => ApiError::NotFound,
            _ => {
                tracing::error!(error = %error, "calendar brain operation failed");
                ApiError::InvalidCommunicationQuery("calendar brain operation failed")
            }
        }
    }
}
impl From<ReminderError> for ApiError {
    fn from(error: ReminderError) -> Self {
        tracing::error!(error = %error, "reminder operation failed");
        ApiError::InvalidCommunicationQuery("reminder operation failed")
    }
}

impl From<CalendarRuleError> for ApiError {
    fn from(error: CalendarRuleError) -> Self {
        match error {
            CalendarRuleError::NotFound => ApiError::NotFound,
            _ => {
                tracing::error!(error = %error, "calendar rule operation failed");
                ApiError::InvalidCommunicationQuery("calendar rule operation failed")
            }
        }
    }
}

impl From<TaskError> for ApiError {
    fn from(error: TaskError) -> Self {
        match error {
            TaskError::NotFound => ApiError::NotFound,
            _ => {
                tracing::error!(error = %error, "task operation failed");
                ApiError::InvalidCommunicationQuery("task operation failed")
            }
        }
    }
}
impl From<TaskCoreError> for ApiError {
    fn from(error: TaskCoreError) -> Self {
        match error {
            TaskCoreError::NotFound => ApiError::NotFound,
            _ => {
                tracing::error!(error = %error, "task core operation failed");
                ApiError::InvalidCommunicationQuery("task core operation failed")
            }
        }
    }
}
impl From<TaskHealthError> for ApiError {
    fn from(error: TaskHealthError) -> Self {
        tracing::error!(error = %error, "task health failed");
        ApiError::InvalidCommunicationQuery("task health failed")
    }
}
impl From<TaskRuleError> for ApiError {
    fn from(error: TaskRuleError) -> Self {
        match error {
            TaskRuleError::NotFound => ApiError::NotFound,
            _ => {
                tracing::error!(error = %error, "task rule failed");
                ApiError::InvalidCommunicationQuery("task rule failed")
            }
        }
    }
}

impl From<TaskBrainError> for ApiError {
    fn from(error: TaskBrainError) -> Self {
        match error {
            TaskBrainError::NotFound => ApiError::NotFound,
            _ => {
                tracing::error!(error = %error, "task brain failed");
                ApiError::InvalidCommunicationQuery("task brain failed")
            }
        }
    }
}
impl From<CalendarError> for ApiError {
    fn from(error: CalendarError) -> Self {
        match error {
            CalendarError::NotFound => ApiError::NotFound,
            _ => {
                tracing::error!(error = %error, "calendar operation failed");
                ApiError::InvalidCommunicationQuery("calendar operation failed")
            }
        }
    }
}

impl From<OrganizationError> for ApiError {
    fn from(error: OrganizationError) -> Self {
        match error {
            OrganizationError::NotFound => ApiError::NotFound,
            _ => {
                tracing::error!(error = %error, "organization operation failed");
                ApiError::InvalidCommunicationQuery("organization operation failed")
            }
        }
    }
}

impl From<EmailAccountSetupError> for ApiError {
    fn from(error: EmailAccountSetupError) -> Self {
        match error {
            EmailAccountSetupError::HostVault(error) => Self::HostVault(error),
            error => Self::AccountSetup(error),
        }
    }
}

impl From<HostVaultError> for ApiError {
    fn from(error: HostVaultError) -> Self {
        Self::HostVault(error)
    }
}
