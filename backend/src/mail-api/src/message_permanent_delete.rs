pub const MAX_MESSAGE_PERMANENT_DELETE_ID_BYTES: usize = 512;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailMessagePermanentDeleteConfirmationV1 {
    Confirmed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailMessagePermanentDeleteCommandV1 {
    pub operation_id: String,
    pub connection_id: String,
    pub message_id: String,
    pub expected_projection_revision: u64,
    pub confirmation: MailMessagePermanentDeleteConfirmationV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailMessagePermanentDeleteAcceptedV1 {
    pub operation_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailMessagePermanentDeleteStatusRequestV1 {
    pub operation_id: String,
    pub connection_id: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailMessagePermanentDeleteOperationOutcomeV1 {
    Pending,
    Succeeded,
    Rejected,
    Unsupported,
    ReauthorizationRequired,
    OutcomeUnknown,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailMessagePermanentDeleteOperationStatusV1 {
    pub operation_id: String,
    pub connection_id: String,
    pub message_id: String,
    pub expected_projection_revision: u64,
    pub confirmation: MailMessagePermanentDeleteConfirmationV1,
    pub outcome: MailMessagePermanentDeleteOperationOutcomeV1,
    pub requested_at_unix_seconds: i64,
    pub completed_at_unix_seconds: Option<i64>,
    pub deletion_projection_revision: Option<u64>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailMessagePermanentDeleteContractErrorV1 {
    InvalidId,
    InvalidRevision,
    InvalidTimestamp,
    InvalidStatus,
}

pub fn validate_message_permanent_delete_command(
    command: &MailMessagePermanentDeleteCommandV1,
) -> Result<(), MailMessagePermanentDeleteContractErrorV1> {
    validate_id(&command.operation_id)?;
    validate_id(&command.connection_id)?;
    validate_id(&command.message_id)?;
    if command.expected_projection_revision == 0 {
        return Err(MailMessagePermanentDeleteContractErrorV1::InvalidRevision);
    }
    Ok(())
}

pub fn validate_message_permanent_delete_accepted(
    accepted: &MailMessagePermanentDeleteAcceptedV1,
) -> Result<(), MailMessagePermanentDeleteContractErrorV1> {
    validate_id(&accepted.operation_id)
}

pub fn validate_message_permanent_delete_status_request(
    request: &MailMessagePermanentDeleteStatusRequestV1,
) -> Result<(), MailMessagePermanentDeleteContractErrorV1> {
    validate_id(&request.operation_id)?;
    validate_id(&request.connection_id)
}

pub fn validate_message_permanent_delete_status(
    status: &MailMessagePermanentDeleteOperationStatusV1,
) -> Result<(), MailMessagePermanentDeleteContractErrorV1> {
    validate_message_permanent_delete_command(&MailMessagePermanentDeleteCommandV1 {
        operation_id: status.operation_id.clone(),
        connection_id: status.connection_id.clone(),
        message_id: status.message_id.clone(),
        expected_projection_revision: status.expected_projection_revision,
        confirmation: status.confirmation,
    })?;
    if status.requested_at_unix_seconds <= 0 {
        return Err(MailMessagePermanentDeleteContractErrorV1::InvalidTimestamp);
    }
    match status.outcome {
        MailMessagePermanentDeleteOperationOutcomeV1::Pending => {
            if status.completed_at_unix_seconds.is_some()
                || status.deletion_projection_revision.is_some()
            {
                return Err(MailMessagePermanentDeleteContractErrorV1::InvalidStatus);
            }
        }
        MailMessagePermanentDeleteOperationOutcomeV1::Succeeded => {
            if status.completed_at_unix_seconds.is_none()
                || status
                    .deletion_projection_revision
                    .is_none_or(|revision| revision <= status.expected_projection_revision)
            {
                return Err(MailMessagePermanentDeleteContractErrorV1::InvalidStatus);
            }
        }
        MailMessagePermanentDeleteOperationOutcomeV1::Rejected
        | MailMessagePermanentDeleteOperationOutcomeV1::Unsupported
        | MailMessagePermanentDeleteOperationOutcomeV1::ReauthorizationRequired
        | MailMessagePermanentDeleteOperationOutcomeV1::OutcomeUnknown => {
            if status.completed_at_unix_seconds.is_none()
                || status.deletion_projection_revision.is_some()
            {
                return Err(MailMessagePermanentDeleteContractErrorV1::InvalidStatus);
            }
        }
    }
    if status
        .completed_at_unix_seconds
        .is_some_and(|completed| completed < status.requested_at_unix_seconds)
    {
        return Err(MailMessagePermanentDeleteContractErrorV1::InvalidTimestamp);
    }
    Ok(())
}

fn validate_id(value: &str) -> Result<(), MailMessagePermanentDeleteContractErrorV1> {
    if value.is_empty()
        || value.trim() != value
        || value.len() > MAX_MESSAGE_PERMANENT_DELETE_ID_BYTES
        || value.chars().any(char::is_control)
    {
        return Err(MailMessagePermanentDeleteContractErrorV1::InvalidId);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn command() -> MailMessagePermanentDeleteCommandV1 {
        MailMessagePermanentDeleteCommandV1 {
            operation_id: "permanent-delete-1".to_owned(),
            connection_id: "mail-account-1".to_owned(),
            message_id: "message-1".to_owned(),
            expected_projection_revision: 7,
            confirmation: MailMessagePermanentDeleteConfirmationV1::Confirmed,
        }
    }

    #[test]
    fn command_requires_a_current_nonzero_projection_revision() {
        assert_eq!(
            validate_message_permanent_delete_command(&command()),
            Ok(())
        );
        let mut invalid = command();
        invalid.expected_projection_revision = 0;
        assert_eq!(
            validate_message_permanent_delete_command(&invalid),
            Err(MailMessagePermanentDeleteContractErrorV1::InvalidRevision)
        );
    }

    #[test]
    fn success_requires_a_newer_deletion_revision() {
        let mut status = MailMessagePermanentDeleteOperationStatusV1 {
            operation_id: "permanent-delete-1".to_owned(),
            connection_id: "mail-account-1".to_owned(),
            message_id: "message-1".to_owned(),
            expected_projection_revision: 7,
            confirmation: MailMessagePermanentDeleteConfirmationV1::Confirmed,
            outcome: MailMessagePermanentDeleteOperationOutcomeV1::Succeeded,
            requested_at_unix_seconds: 10,
            completed_at_unix_seconds: Some(11),
            deletion_projection_revision: Some(8),
        };
        assert_eq!(validate_message_permanent_delete_status(&status), Ok(()));
        status.deletion_projection_revision = Some(7);
        assert_eq!(
            validate_message_permanent_delete_status(&status),
            Err(MailMessagePermanentDeleteContractErrorV1::InvalidStatus)
        );
    }
}
