use std::io;

use crate::ai::control_center::AiControlCenterError;
use crate::ai::core::AiError;
use crate::application::provider_runtime_contracts::{TelegramError, WhatsappWebError, ZoomError};
use crate::application::review_promotion::ReviewPromotionError;
use crate::domains::calendar::events::CalendarError;
use crate::domains::communications::core::CommunicationIngestionError;
use crate::domains::communications::messages::MessageProjectionError;
use crate::domains::communications::storage::CommunicationStorageError;
use crate::domains::decisions::DecisionStoreError;
use crate::domains::documents::processing::DocumentProcessingError;
use crate::domains::obligations::ObligationStoreError;
use crate::domains::organizations::api::OrganizationError;
use crate::domains::persons::api::PersonProjectionError;
use crate::domains::persons::identity::PersonIdentityError;
use crate::domains::projects::core::ProjectStoreError;
use crate::domains::projects::link_reviews::ProjectLinkReviewError;
use crate::domains::relationships::RelationshipStoreError;
use crate::domains::review::ReviewInboxError;
use crate::domains::signal_hub::SignalHubError;
use crate::domains::tasks::candidates::TaskCandidateError;
use crate::engines::automation::AutomationError;
use crate::engines::consistency::ConsistencyError;
use crate::integrations::mail::accounts::EmailAccountSetupError;
use crate::platform::audit::ApiAuditError;
use crate::platform::calls::CallError;
use crate::platform::events::{EventEnvelopeError, EventStoreError};
use crate::platform::settings::SettingsError;
use crate::platform::storage::StorageError;
use crate::vault::HostVaultError;

pub enum ApiError {
    DatabaseNotConfigured,
    InvalidEnvelope(EventEnvelopeError),
    Audit(ApiAuditError),
    Store(EventStoreError),
    Graph(crate::domains::graph::core::GraphStoreError),
    InvalidGraphQuery(&'static str),
    InvalidPersonaQuery(&'static str),
    Projects(ProjectStoreError),
    InvalidProjectQuery(&'static str),
    InvalidProjectLinkReview(&'static str),
    InvalidTaskCandidateQuery(&'static str),
    InvalidTaskCandidateReview(&'static str),
    InvalidTaskQuery(&'static str),
    InvalidObligationQuery(&'static str),
    InvalidObligationReview(&'static str),
    InvalidDecisionQuery(&'static str),
    InvalidDecisionReview(&'static str),
    InvalidRelationshipQuery(&'static str),
    InvalidRelationshipReview(&'static str),
    InvalidContradictionQuery(&'static str),
    InvalidContradictionReview(&'static str),
    InvalidReviewQuery(&'static str),
    InvalidReviewItem(&'static str),
    FailedPrecondition(String),
    InvalidPersonIdentityReview(&'static str),
    InvalidDocumentProcessingQuery(&'static str),
    Settings(SettingsError),
    SignalHub(SignalHubError),
    SettingNotFound,
    DocumentProcessing(DocumentProcessingError),
    TaskCandidateNotFound,
    TaskCandidate(TaskCandidateError),
    ObligationNotFound,
    Obligation(ObligationStoreError),
    DecisionNotFound,
    Decision(DecisionStoreError),
    RelationshipNotFound,
    Relationship(RelationshipStoreError),
    ContradictionObservationNotFound,
    ReviewItemNotFound,
    ReviewInbox(ReviewInboxError),
    ReviewPromotion(ReviewPromotionError),
    Consistency(ConsistencyError),
    AiRunNotFound,
    Ai(AiError),
    AiControlCenter(AiControlCenterError),
    Telegram(TelegramError),
    WhatsappWeb(WhatsappWebError),
    Zoom(ZoomError),
    Automation(AutomationError),
    Call(CallError),
    ProjectLinkTargetNotFound,
    ProjectLinkReview(ProjectLinkReviewError),
    PersonIdentityNotFound,
    PersonProjection(PersonProjectionError),
    PersonIdentity(PersonIdentityError),
    Messages(MessageProjectionError),
    CommunicationIngestion(CommunicationIngestionError),
    CommunicationStorage(CommunicationStorageError),
    InvalidCommunicationQuery(&'static str),
    EmailAccountDeleteConflict,
    ProviderWriteConfirmationRequired,
    CommunicationMessageNotFound,
    SecretVaultNotConfigured,
    HostVault(HostVaultError),
    AccountSetup(EmailAccountSetupError),
    AccountSetupState,
    AccountSetupPendingGrantNotFound,
    AccountSetupStateMismatch,
    GraphNotFound,
    ProjectNotFound,
    NotFound,
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    Storage(#[from] StorageError),

    #[error(transparent)]
    Io(#[from] io::Error),
}
