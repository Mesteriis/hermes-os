use crate::{MailContactsSyncPersistenceErrorV1, OutboxEnvelopeV1};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailContactsSyncReverseOperationSeedV1 {
    pub operation_id: [u8; 16],
    pub configuration_instance_id: String,
    pub account_id: String,
    pub contact_id: [u8; 16],
    pub contact_revision: u64,
    pub source_prepare_command: OutboxEnvelopeV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AcceptContactChangedForMailSyncV1 {
    pub logical_owner_id: String,
    pub event_message_id: [u8; 16],
    pub event_envelope_sha256: [u8; 32],
    pub operations: Vec<MailContactsSyncReverseOperationSeedV1>,
    pub occurred_at_unix_millis: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AcceptContactChangedForMailSyncOutcomeV1 {
    Applied { operations: u16 },
    Duplicate,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailContactsSyncReverseOperationV1 {
    pub operation_id: [u8; 16],
    pub configuration_instance_id: String,
    pub account_id: String,
    pub contact_id: [u8; 16],
    pub contact_revision: u64,
    pub state: u8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompleteContactMailSyncSourceV1 {
    pub logical_owner_id: String,
    pub result_message_id: [u8; 16],
    pub result_envelope_sha256: [u8; 32],
    pub operation_id: [u8; 16],
    pub mail_command: Option<OutboxEnvelopeV1>,
    pub rejected: bool,
    pub occurred_at_unix_millis: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompleteContactMailSyncSourceOutcomeV1 {
    Applied,
    Duplicate,
}

pub(crate) fn validate_changed_input(
    input: &AcceptContactChangedForMailSyncV1,
) -> Result<(), MailContactsSyncPersistenceErrorV1> {
    if !crate::model::valid_identity(&input.logical_owner_id)
        || input.event_message_id.iter().all(|byte| *byte == 0)
        || input.event_envelope_sha256.iter().all(|byte| *byte == 0)
        || input.operations.len() > 32
        || input.occurred_at_unix_millis <= 0
    {
        return Err(MailContactsSyncPersistenceErrorV1::InvalidInput);
    }
    let mut operation_ids = std::collections::BTreeSet::new();
    let mut configurations = std::collections::BTreeSet::new();
    for operation in &input.operations {
        if operation.operation_id.iter().all(|byte| *byte == 0)
            || !crate::model::valid_identity(&operation.configuration_instance_id)
            || operation.account_id.trim().is_empty()
            || operation.account_id.len() > 256
            || operation.account_id.chars().any(char::is_control)
            || operation.contact_id.iter().all(|byte| *byte == 0)
            || operation.contact_revision == 0
            || !crate::model::valid_envelope(&operation.source_prepare_command)
            || operation.source_prepare_command.message_id != operation.operation_id
            || !operation_ids.insert(operation.operation_id)
            || !configurations.insert(&operation.configuration_instance_id)
        {
            return Err(MailContactsSyncPersistenceErrorV1::InvalidInput);
        }
    }
    Ok(())
}

pub(crate) fn validate_source_completion(
    input: &CompleteContactMailSyncSourceV1,
) -> Result<(), MailContactsSyncPersistenceErrorV1> {
    if !crate::model::valid_identity(&input.logical_owner_id)
        || input.result_message_id.iter().all(|byte| *byte == 0)
        || input.result_envelope_sha256.iter().all(|byte| *byte == 0)
        || input.operation_id.iter().all(|byte| *byte == 0)
        || input.occurred_at_unix_millis <= 0
        || input.rejected == input.mail_command.is_some()
        || input.mail_command.as_ref().is_some_and(|command| {
            !crate::model::valid_envelope(command) || command.message_id == input.result_message_id
        })
    {
        return Err(MailContactsSyncPersistenceErrorV1::InvalidInput);
    }
    Ok(())
}
