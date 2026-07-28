pub const MAX_MESSAGE_LOCATION_ID_BYTES: usize = 512;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailMessageLocationKindV1 {
    Archive,
    Trash,
    Restore,
    Move,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailMessageLocationCommandV1 {
    pub operation_id: String,
    pub connection_id: String,
    pub message_id: String,
    pub kind: MailMessageLocationKindV1,
    pub target_folder_id: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailMessageLocationAcceptedV1 {
    pub operation_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailMessageLocationStatusRequestV1 {
    pub operation_id: String,
    pub connection_id: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailMessageLocationOperationOutcomeV1 {
    Pending,
    Succeeded,
    Rejected,
    Unsupported,
    OutcomeUnknown,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailMessageLocationOperationStatusV1 {
    pub operation_id: String,
    pub connection_id: String,
    pub message_id: String,
    pub kind: MailMessageLocationKindV1,
    pub target_folder_id: Option<String>,
    pub outcome: MailMessageLocationOperationOutcomeV1,
    pub requested_at_unix_seconds: i64,
    pub completed_at_unix_seconds: Option<i64>,
    pub projection_revision: Option<u64>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailMessageLocationContractErrorV1 {
    InvalidId,
    InvalidTarget,
    InvalidTimestamp,
    InvalidStatus,
}

pub fn validate_message_location_command(
    command: &MailMessageLocationCommandV1,
) -> Result<(), MailMessageLocationContractErrorV1> {
    validate_id(&command.operation_id)?;
    validate_id(&command.connection_id)?;
    validate_id(&command.message_id)?;
    match (command.kind, command.target_folder_id.as_deref()) {
        (MailMessageLocationKindV1::Move, Some(target)) => validate_id(target),
        (MailMessageLocationKindV1::Move, None) => {
            Err(MailMessageLocationContractErrorV1::InvalidTarget)
        }
        (_, None) => Ok(()),
        (_, Some(_)) => Err(MailMessageLocationContractErrorV1::InvalidTarget),
    }
}

pub fn validate_message_location_accepted(
    accepted: &MailMessageLocationAcceptedV1,
) -> Result<(), MailMessageLocationContractErrorV1> {
    validate_id(&accepted.operation_id)
}

pub fn validate_message_location_status_request(
    request: &MailMessageLocationStatusRequestV1,
) -> Result<(), MailMessageLocationContractErrorV1> {
    validate_id(&request.operation_id)?;
    validate_id(&request.connection_id)
}

pub fn validate_message_location_status(
    status: &MailMessageLocationOperationStatusV1,
) -> Result<(), MailMessageLocationContractErrorV1> {
    validate_message_location_command(&MailMessageLocationCommandV1 {
        operation_id: status.operation_id.clone(),
        connection_id: status.connection_id.clone(),
        message_id: status.message_id.clone(),
        kind: status.kind,
        target_folder_id: status.target_folder_id.clone(),
    })?;
    if status.requested_at_unix_seconds <= 0 {
        return Err(MailMessageLocationContractErrorV1::InvalidTimestamp);
    }
    match status.outcome {
        MailMessageLocationOperationOutcomeV1::Pending => {
            if status.completed_at_unix_seconds.is_some() || status.projection_revision.is_some() {
                return Err(MailMessageLocationContractErrorV1::InvalidStatus);
            }
        }
        MailMessageLocationOperationOutcomeV1::Succeeded => {
            if status.completed_at_unix_seconds.is_none()
                || status
                    .projection_revision
                    .is_none_or(|revision| revision == 0)
            {
                return Err(MailMessageLocationContractErrorV1::InvalidStatus);
            }
        }
        MailMessageLocationOperationOutcomeV1::Rejected
        | MailMessageLocationOperationOutcomeV1::Unsupported
        | MailMessageLocationOperationOutcomeV1::OutcomeUnknown => {
            if status.completed_at_unix_seconds.is_none() || status.projection_revision.is_some() {
                return Err(MailMessageLocationContractErrorV1::InvalidStatus);
            }
        }
    }
    if status
        .completed_at_unix_seconds
        .is_some_and(|completed| completed < status.requested_at_unix_seconds)
    {
        return Err(MailMessageLocationContractErrorV1::InvalidTimestamp);
    }
    Ok(())
}

fn validate_id(value: &str) -> Result<(), MailMessageLocationContractErrorV1> {
    if value.is_empty()
        || value.trim() != value
        || value.len() > MAX_MESSAGE_LOCATION_ID_BYTES
        || value.chars().any(char::is_control)
    {
        return Err(MailMessageLocationContractErrorV1::InvalidId);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn command(kind: MailMessageLocationKindV1) -> MailMessageLocationCommandV1 {
        MailMessageLocationCommandV1 {
            operation_id: "location-operation-1".to_owned(),
            connection_id: "mail-account-1".to_owned(),
            message_id: "message-1".to_owned(),
            kind,
            target_folder_id: None,
        }
    }

    #[test]
    fn move_alone_requires_an_exact_target_folder() {
        assert_eq!(
            validate_message_location_command(&command(MailMessageLocationKindV1::Archive)),
            Ok(())
        );
        assert_eq!(
            validate_message_location_command(&command(MailMessageLocationKindV1::Move)),
            Err(MailMessageLocationContractErrorV1::InvalidTarget)
        );
        let mut move_command = command(MailMessageLocationKindV1::Move);
        move_command.target_folder_id = Some("Archive/2026".to_owned());
        assert_eq!(validate_message_location_command(&move_command), Ok(()));
        let mut trash = command(MailMessageLocationKindV1::Trash);
        trash.target_folder_id = Some("Trash".to_owned());
        assert_eq!(
            validate_message_location_command(&trash),
            Err(MailMessageLocationContractErrorV1::InvalidTarget)
        );
    }

    #[test]
    fn unsupported_is_a_terminal_status_without_projection_evidence() {
        let status = MailMessageLocationOperationStatusV1 {
            operation_id: "location-operation-1".to_owned(),
            connection_id: "mail-account-1".to_owned(),
            message_id: "message-1".to_owned(),
            kind: MailMessageLocationKindV1::Archive,
            target_folder_id: None,
            outcome: MailMessageLocationOperationOutcomeV1::Unsupported,
            requested_at_unix_seconds: 10,
            completed_at_unix_seconds: Some(11),
            projection_revision: None,
        };
        assert_eq!(validate_message_location_status(&status), Ok(()));
    }
}
