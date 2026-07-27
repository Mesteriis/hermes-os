pub const MAX_MESSAGE_FLAG_ID_BYTES: usize = 512;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailMessageFlagKindV1 {
    Read,
    Starred,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailMessageFlagCommandV1 {
    pub operation_id: String,
    pub connection_id: String,
    pub provider_message_id: String,
    pub kind: MailMessageFlagKindV1,
    pub target_value: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailMessageFlagAcceptedV1 {
    pub operation_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailMessageFlagStatusRequestV1 {
    pub operation_id: String,
    pub connection_id: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailMessageFlagOperationOutcomeV1 {
    Pending,
    Succeeded,
    Rejected,
    OutcomeUnknown,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailMessageFlagOperationStatusV1 {
    pub operation_id: String,
    pub connection_id: String,
    pub provider_message_id: String,
    pub kind: MailMessageFlagKindV1,
    pub target_value: bool,
    pub outcome: MailMessageFlagOperationOutcomeV1,
    pub requested_at_unix_seconds: i64,
    pub completed_at_unix_seconds: Option<i64>,
    pub projection_revision: Option<u64>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailMessageFlagContractErrorV1 {
    InvalidId,
    InvalidTimestamp,
    InvalidStatus,
}

pub fn validate_message_flag_command(
    command: &MailMessageFlagCommandV1,
) -> Result<(), MailMessageFlagContractErrorV1> {
    validate_id(&command.operation_id)?;
    validate_id(&command.connection_id)?;
    validate_id(&command.provider_message_id)
}

pub fn validate_message_flag_accepted(
    accepted: &MailMessageFlagAcceptedV1,
) -> Result<(), MailMessageFlagContractErrorV1> {
    validate_id(&accepted.operation_id)
}

pub fn validate_message_flag_status_request(
    request: &MailMessageFlagStatusRequestV1,
) -> Result<(), MailMessageFlagContractErrorV1> {
    validate_id(&request.operation_id)?;
    validate_id(&request.connection_id)
}

pub fn validate_message_flag_status(
    status: &MailMessageFlagOperationStatusV1,
) -> Result<(), MailMessageFlagContractErrorV1> {
    validate_id(&status.operation_id)?;
    validate_id(&status.connection_id)?;
    validate_id(&status.provider_message_id)?;
    if status.requested_at_unix_seconds <= 0 {
        return Err(MailMessageFlagContractErrorV1::InvalidTimestamp);
    }
    match status.outcome {
        MailMessageFlagOperationOutcomeV1::Pending => {
            if status.completed_at_unix_seconds.is_some() || status.projection_revision.is_some() {
                return Err(MailMessageFlagContractErrorV1::InvalidStatus);
            }
        }
        MailMessageFlagOperationOutcomeV1::Succeeded => {
            if status.completed_at_unix_seconds.is_none()
                || status
                    .projection_revision
                    .is_none_or(|revision| revision == 0)
            {
                return Err(MailMessageFlagContractErrorV1::InvalidStatus);
            }
        }
        MailMessageFlagOperationOutcomeV1::Rejected
        | MailMessageFlagOperationOutcomeV1::OutcomeUnknown => {
            if status.completed_at_unix_seconds.is_none() || status.projection_revision.is_some() {
                return Err(MailMessageFlagContractErrorV1::InvalidStatus);
            }
        }
    }
    if status
        .completed_at_unix_seconds
        .is_some_and(|completed| completed < status.requested_at_unix_seconds)
    {
        return Err(MailMessageFlagContractErrorV1::InvalidTimestamp);
    }
    Ok(())
}

fn validate_id(value: &str) -> Result<(), MailMessageFlagContractErrorV1> {
    if value.is_empty()
        || value.trim() != value
        || value.len() > MAX_MESSAGE_FLAG_ID_BYTES
        || value.chars().any(char::is_control)
    {
        return Err(MailMessageFlagContractErrorV1::InvalidId);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_requires_exact_bounded_ids() {
        let command = MailMessageFlagCommandV1 {
            operation_id: "flag-operation-1".to_owned(),
            connection_id: "mail-account-1".to_owned(),
            provider_message_id: "provider-message-1".to_owned(),
            kind: MailMessageFlagKindV1::Read,
            target_value: true,
        };
        assert_eq!(validate_message_flag_command(&command), Ok(()));
        assert_eq!(
            validate_message_flag_command(&MailMessageFlagCommandV1 {
                provider_message_id: " provider-message-1".to_owned(),
                ..command
            }),
            Err(MailMessageFlagContractErrorV1::InvalidId)
        );
    }

    #[test]
    fn terminal_status_requires_consistent_completion_evidence() {
        let status = MailMessageFlagOperationStatusV1 {
            operation_id: "flag-operation-1".to_owned(),
            connection_id: "mail-account-1".to_owned(),
            provider_message_id: "provider-message-1".to_owned(),
            kind: MailMessageFlagKindV1::Starred,
            target_value: true,
            outcome: MailMessageFlagOperationOutcomeV1::Succeeded,
            requested_at_unix_seconds: 100,
            completed_at_unix_seconds: Some(101),
            projection_revision: Some(2),
        };
        assert_eq!(validate_message_flag_status(&status), Ok(()));
        assert_eq!(
            validate_message_flag_status(&MailMessageFlagOperationStatusV1 {
                projection_revision: None,
                ..status
            }),
            Err(MailMessageFlagContractErrorV1::InvalidStatus)
        );
    }
}
