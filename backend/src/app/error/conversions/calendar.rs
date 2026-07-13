use super::super::types::ApiError;
use crate::application::calendar_meeting_outcomes::CalendarMeetingOutcomeApplicationError;
use crate::domains::calendar::brain::CalendarBrainError;
use crate::domains::calendar::command_service::CalendarCommandServiceError;
use crate::domains::calendar::core::CalendarCoreError;
use crate::domains::calendar::events::CalendarError;
use crate::domains::calendar::health::CalendarHealthError;
use crate::domains::calendar::meetings::MeetingsError;
use crate::domains::calendar::reminders::ReminderError;
use crate::domains::calendar::rules::CalendarRuleError;
use crate::domains::calendar::scheduling::SchedulingError;

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

impl From<CalendarCommandServiceError> for ApiError {
    fn from(error: CalendarCommandServiceError) -> Self {
        match error {
            CalendarCommandServiceError::ObservationCapture(source) => {
                tracing::error!(error = %source, "calendar command observation capture failed");
                ApiError::InvalidCommunicationQuery("calendar command observation capture failed")
            }
            CalendarCommandServiceError::Calendar(source) => ApiError::from(source),
            CalendarCommandServiceError::CalendarCore(source) => ApiError::from(source),
            CalendarCommandServiceError::Meetings(source) => ApiError::from(source),
            CalendarCommandServiceError::Reminder(source) => ApiError::from(source),
            CalendarCommandServiceError::CalendarRule(source) => ApiError::from(source),
            CalendarCommandServiceError::Scheduling(source) => ApiError::from(source),
        }
    }
}

impl From<CalendarMeetingOutcomeApplicationError> for ApiError {
    fn from(error: CalendarMeetingOutcomeApplicationError) -> Self {
        match error {
            CalendarMeetingOutcomeApplicationError::Meetings(source) => ApiError::from(source),
            CalendarMeetingOutcomeApplicationError::Decision(source) => ApiError::from(source),
            CalendarMeetingOutcomeApplicationError::Obligation(source) => ApiError::from(source),
            CalendarMeetingOutcomeApplicationError::Observation(source) => {
                tracing::error!(error = %source, "calendar meeting outcome observation failed");
                ApiError::InvalidCommunicationQuery("calendar meeting outcome observation failed")
            }
            CalendarMeetingOutcomeApplicationError::ReviewMirror(source) => {
                tracing::error!(error = %source, "calendar meeting outcome review mirror failed");
                ApiError::InvalidCommunicationQuery("calendar meeting outcome review mirror failed")
            }
            CalendarMeetingOutcomeApplicationError::Sqlx(source) => {
                tracing::error!(error = %source, "calendar meeting outcome operation failed");
                ApiError::InvalidCommunicationQuery("calendar meeting outcome operation failed")
            }
        }
    }
}
