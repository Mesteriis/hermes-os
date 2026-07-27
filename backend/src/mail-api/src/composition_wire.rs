//! Canonical Protobuf mapping for Mail-owned composition contracts.

use prost::Message;

use crate::{
    client_wire::MailClientWireErrorV1,
    composition::{
        MailCompositionCommandV1, MailCompositionEntityKindV1, MailCompositionModeV1,
        MailCompositionMutationReceiptV1, MailCompositionPageV1, MailCompositionQueryResponseV1,
        MailCompositionQueryV1, MailDraftInputV1, MailDraftV1, MailSignatureInputV1,
        MailSignatureV1, MailTemplateInputV1, MailTemplatePreviewV1, MailTemplateV1,
        MailTemplateVariableValueV1, validate_composition_command, validate_composition_query,
        validate_composition_receipt, validate_composition_response,
    },
    composition_wire_generated as wire,
};

pub fn encode_composition_command(
    command: &MailCompositionCommandV1,
) -> Result<Vec<u8>, MailClientWireErrorV1> {
    validate_composition_command(command).map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    use wire::mail_composition_command_v1::Command;
    let command = match command {
        MailCompositionCommandV1::UpsertDraft {
            operation_id,
            draft,
            expected_revision,
        } => Command::UpsertDraft(wire::UpsertMailDraftCommandV1 {
            operation_id: operation_id.clone(),
            draft: Some(draft_input_to_wire(draft)),
            expected_revision: *expected_revision,
        }),
        MailCompositionCommandV1::DeleteDraft {
            operation_id,
            connection_id,
            draft_id,
            expected_revision,
        } => Command::DeleteDraft(wire::DeleteMailDraftCommandV1 {
            operation_id: operation_id.clone(),
            connection_id: connection_id.clone(),
            draft_id: draft_id.clone(),
            expected_revision: *expected_revision,
        }),
        MailCompositionCommandV1::UpsertTemplate {
            operation_id,
            template,
            expected_revision,
        } => Command::UpsertTemplate(wire::UpsertMailTemplateCommandV1 {
            operation_id: operation_id.clone(),
            template: Some(template_input_to_wire(template)),
            expected_revision: *expected_revision,
        }),
        MailCompositionCommandV1::DeleteTemplate {
            operation_id,
            connection_id,
            template_id,
            expected_revision,
        } => Command::DeleteTemplate(wire::DeleteMailTemplateCommandV1 {
            operation_id: operation_id.clone(),
            connection_id: connection_id.clone(),
            template_id: template_id.clone(),
            expected_revision: *expected_revision,
        }),
        MailCompositionCommandV1::UpsertSignature {
            operation_id,
            signature,
            expected_revision,
        } => Command::UpsertSignature(wire::UpsertMailSignatureCommandV1 {
            operation_id: operation_id.clone(),
            signature: Some(signature_input_to_wire(signature)),
            expected_revision: *expected_revision,
        }),
        MailCompositionCommandV1::DeleteSignature {
            operation_id,
            connection_id,
            signature_id,
            expected_revision,
        } => Command::DeleteSignature(wire::DeleteMailSignatureCommandV1 {
            operation_id: operation_id.clone(),
            connection_id: connection_id.clone(),
            signature_id: signature_id.clone(),
            expected_revision: *expected_revision,
        }),
    };
    Ok(wire::MailCompositionCommandV1 {
        command: Some(command),
    }
    .encode_to_vec())
}

pub fn decode_composition_command(
    bytes: &[u8],
) -> Result<MailCompositionCommandV1, MailClientWireErrorV1> {
    use wire::mail_composition_command_v1::Command;
    let command = wire::MailCompositionCommandV1::decode(bytes)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?
        .command
        .ok_or(MailClientWireErrorV1::InvalidPayload)?;
    let command = match command {
        Command::UpsertDraft(value) => MailCompositionCommandV1::UpsertDraft {
            operation_id: value.operation_id,
            draft: draft_input_from_wire(
                value.draft.ok_or(MailClientWireErrorV1::InvalidPayload)?,
            )?,
            expected_revision: value.expected_revision,
        },
        Command::DeleteDraft(value) => MailCompositionCommandV1::DeleteDraft {
            operation_id: value.operation_id,
            connection_id: value.connection_id,
            draft_id: value.draft_id,
            expected_revision: value.expected_revision,
        },
        Command::UpsertTemplate(value) => MailCompositionCommandV1::UpsertTemplate {
            operation_id: value.operation_id,
            template: template_input_from_wire(
                value
                    .template
                    .ok_or(MailClientWireErrorV1::InvalidPayload)?,
            ),
            expected_revision: value.expected_revision,
        },
        Command::DeleteTemplate(value) => MailCompositionCommandV1::DeleteTemplate {
            operation_id: value.operation_id,
            connection_id: value.connection_id,
            template_id: value.template_id,
            expected_revision: value.expected_revision,
        },
        Command::UpsertSignature(value) => MailCompositionCommandV1::UpsertSignature {
            operation_id: value.operation_id,
            signature: signature_input_from_wire(
                value
                    .signature
                    .ok_or(MailClientWireErrorV1::InvalidPayload)?,
            ),
            expected_revision: value.expected_revision,
        },
        Command::DeleteSignature(value) => MailCompositionCommandV1::DeleteSignature {
            operation_id: value.operation_id,
            connection_id: value.connection_id,
            signature_id: value.signature_id,
            expected_revision: value.expected_revision,
        },
    };
    validate_composition_command(&command).map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    if encode_composition_command(&command)? != bytes {
        return Err(MailClientWireErrorV1::InvalidPayload);
    }
    Ok(command)
}

pub fn encode_composition_receipt(
    receipt: &MailCompositionMutationReceiptV1,
) -> Result<Vec<u8>, MailClientWireErrorV1> {
    validate_composition_receipt(receipt).map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    Ok(wire::MailCompositionMutationReceiptV1 {
        operation_id: receipt.operation_id.clone(),
        connection_id: receipt.connection_id.clone(),
        entity_kind: entity_kind_to_wire(receipt.entity_kind) as i32,
        entity_id: receipt.entity_id.clone(),
        revision: receipt.revision,
        deleted: receipt.deleted,
    }
    .encode_to_vec())
}

pub fn decode_composition_receipt(
    bytes: &[u8],
) -> Result<MailCompositionMutationReceiptV1, MailClientWireErrorV1> {
    let value = wire::MailCompositionMutationReceiptV1::decode(bytes)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    let receipt = MailCompositionMutationReceiptV1 {
        operation_id: value.operation_id,
        connection_id: value.connection_id,
        entity_kind: entity_kind_from_wire(value.entity_kind)?,
        entity_id: value.entity_id,
        revision: value.revision,
        deleted: value.deleted,
    };
    validate_composition_receipt(&receipt).map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    if encode_composition_receipt(&receipt)? != bytes {
        return Err(MailClientWireErrorV1::InvalidPayload);
    }
    Ok(receipt)
}

pub fn encode_composition_query(
    query: &MailCompositionQueryV1,
) -> Result<Vec<u8>, MailClientWireErrorV1> {
    validate_composition_query(query).map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    use wire::mail_composition_query_v1::Query;
    let query = match query {
        MailCompositionQueryV1::ListDrafts {
            connection_id,
            cursor,
            limit,
        } => Query::ListDrafts(wire::ListMailDraftsQueryV1 {
            connection_id: connection_id.clone(),
            cursor: cursor.clone(),
            limit: *limit,
        }),
        MailCompositionQueryV1::GetDraft {
            connection_id,
            draft_id,
        } => Query::GetDraft(wire::GetMailDraftQueryV1 {
            connection_id: connection_id.clone(),
            draft_id: draft_id.clone(),
        }),
        MailCompositionQueryV1::ListTemplates {
            connection_id,
            cursor,
            limit,
        } => Query::ListTemplates(wire::ListMailTemplatesQueryV1 {
            connection_id: connection_id.clone(),
            cursor: cursor.clone(),
            limit: *limit,
        }),
        MailCompositionQueryV1::GetTemplate {
            connection_id,
            template_id,
        } => Query::GetTemplate(wire::GetMailTemplateQueryV1 {
            connection_id: connection_id.clone(),
            template_id: template_id.clone(),
        }),
        MailCompositionQueryV1::PreviewTemplate {
            connection_id,
            template_id,
            values,
        } => Query::PreviewTemplate(wire::PreviewMailTemplateQueryV1 {
            connection_id: connection_id.clone(),
            template_id: template_id.clone(),
            value: values
                .iter()
                .map(|value| wire::MailTemplateVariableValueV1 {
                    name: value.name.clone(),
                    value: value.value.clone(),
                })
                .collect(),
        }),
        MailCompositionQueryV1::ListSignatures {
            connection_id,
            cursor,
            limit,
        } => Query::ListSignatures(wire::ListMailSignaturesQueryV1 {
            connection_id: connection_id.clone(),
            cursor: cursor.clone(),
            limit: *limit,
        }),
        MailCompositionQueryV1::GetSignature {
            connection_id,
            signature_id,
        } => Query::GetSignature(wire::GetMailSignatureQueryV1 {
            connection_id: connection_id.clone(),
            signature_id: signature_id.clone(),
        }),
    };
    Ok(wire::MailCompositionQueryV1 { query: Some(query) }.encode_to_vec())
}

pub fn decode_composition_query(
    bytes: &[u8],
) -> Result<MailCompositionQueryV1, MailClientWireErrorV1> {
    use wire::mail_composition_query_v1::Query;
    let query = wire::MailCompositionQueryV1::decode(bytes)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?
        .query
        .ok_or(MailClientWireErrorV1::InvalidPayload)?;
    let query = match query {
        Query::ListDrafts(value) => MailCompositionQueryV1::ListDrafts {
            connection_id: value.connection_id,
            cursor: value.cursor,
            limit: value.limit,
        },
        Query::GetDraft(value) => MailCompositionQueryV1::GetDraft {
            connection_id: value.connection_id,
            draft_id: value.draft_id,
        },
        Query::ListTemplates(value) => MailCompositionQueryV1::ListTemplates {
            connection_id: value.connection_id,
            cursor: value.cursor,
            limit: value.limit,
        },
        Query::GetTemplate(value) => MailCompositionQueryV1::GetTemplate {
            connection_id: value.connection_id,
            template_id: value.template_id,
        },
        Query::PreviewTemplate(value) => MailCompositionQueryV1::PreviewTemplate {
            connection_id: value.connection_id,
            template_id: value.template_id,
            values: value
                .value
                .into_iter()
                .map(|value| MailTemplateVariableValueV1 {
                    name: value.name,
                    value: value.value,
                })
                .collect(),
        },
        Query::ListSignatures(value) => MailCompositionQueryV1::ListSignatures {
            connection_id: value.connection_id,
            cursor: value.cursor,
            limit: value.limit,
        },
        Query::GetSignature(value) => MailCompositionQueryV1::GetSignature {
            connection_id: value.connection_id,
            signature_id: value.signature_id,
        },
    };
    validate_composition_query(&query).map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    if encode_composition_query(&query)? != bytes {
        return Err(MailClientWireErrorV1::InvalidPayload);
    }
    Ok(query)
}

pub fn encode_composition_query_response(
    response: &MailCompositionQueryResponseV1,
) -> Result<Vec<u8>, MailClientWireErrorV1> {
    validate_composition_response(response).map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    use wire::mail_composition_query_response_v1::Response;
    let response = match response {
        MailCompositionQueryResponseV1::Drafts(page) => Response::Drafts(wire::MailDraftPageV1 {
            item: page.items.iter().map(draft_to_wire).collect(),
            next_cursor: page.next_cursor.clone(),
        }),
        MailCompositionQueryResponseV1::Draft(value) => Response::Draft(draft_to_wire(value)),
        MailCompositionQueryResponseV1::Templates(page) => {
            Response::Templates(wire::MailTemplatePageV1 {
                item: page.items.iter().map(template_to_wire).collect(),
                next_cursor: page.next_cursor.clone(),
            })
        }
        MailCompositionQueryResponseV1::Template(value) => {
            Response::Template(template_to_wire(value))
        }
        MailCompositionQueryResponseV1::TemplatePreview(value) => {
            Response::TemplatePreview(preview_to_wire(value))
        }
        MailCompositionQueryResponseV1::Signatures(page) => {
            Response::Signatures(wire::MailSignaturePageV1 {
                item: page.items.iter().map(signature_to_wire).collect(),
                next_cursor: page.next_cursor.clone(),
            })
        }
        MailCompositionQueryResponseV1::Signature(value) => {
            Response::Signature(signature_to_wire(value))
        }
        MailCompositionQueryResponseV1::NotFound => Response::NotFound(true),
    };
    Ok(wire::MailCompositionQueryResponseV1 {
        response: Some(response),
    }
    .encode_to_vec())
}

pub fn decode_composition_query_response(
    bytes: &[u8],
) -> Result<MailCompositionQueryResponseV1, MailClientWireErrorV1> {
    use wire::mail_composition_query_response_v1::Response;
    let response = wire::MailCompositionQueryResponseV1::decode(bytes)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?
        .response
        .ok_or(MailClientWireErrorV1::InvalidPayload)?;
    let response = match response {
        Response::Drafts(page) => MailCompositionQueryResponseV1::Drafts(MailCompositionPageV1 {
            items: page
                .item
                .into_iter()
                .map(draft_from_wire)
                .collect::<Result<Vec<_>, _>>()?,
            next_cursor: page.next_cursor,
        }),
        Response::Draft(value) => MailCompositionQueryResponseV1::Draft(draft_from_wire(value)?),
        Response::Templates(page) => {
            MailCompositionQueryResponseV1::Templates(MailCompositionPageV1 {
                items: page.item.into_iter().map(template_from_wire).collect(),
                next_cursor: page.next_cursor,
            })
        }
        Response::Template(value) => {
            MailCompositionQueryResponseV1::Template(template_from_wire(value))
        }
        Response::TemplatePreview(value) => {
            MailCompositionQueryResponseV1::TemplatePreview(preview_from_wire(value))
        }
        Response::Signatures(page) => {
            MailCompositionQueryResponseV1::Signatures(MailCompositionPageV1 {
                items: page.item.into_iter().map(signature_from_wire).collect(),
                next_cursor: page.next_cursor,
            })
        }
        Response::Signature(value) => {
            MailCompositionQueryResponseV1::Signature(signature_from_wire(value))
        }
        Response::NotFound(value) if value => MailCompositionQueryResponseV1::NotFound,
        Response::NotFound(_) => return Err(MailClientWireErrorV1::InvalidPayload),
    };
    validate_composition_response(&response).map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    if encode_composition_query_response(&response)? != bytes {
        return Err(MailClientWireErrorV1::InvalidPayload);
    }
    Ok(response)
}

fn draft_input_to_wire(value: &MailDraftInputV1) -> wire::MailDraftInputV1 {
    wire::MailDraftInputV1 {
        connection_id: value.connection_id.clone(),
        draft_id: value.draft_id.clone(),
        mode: mode_to_wire(value.mode) as i32,
        provider_conversation_id: value.provider_conversation_id.clone(),
        in_reply_to_provider_message_id: value.in_reply_to_provider_message_id.clone(),
        to_recipient: value.to_recipients.clone(),
        cc_recipient: value.cc_recipients.clone(),
        bcc_recipient: value.bcc_recipients.clone(),
        subject: value.subject.clone(),
        text_body: value.text_body.clone(),
        template_id: value.template_id.clone(),
        signature_id: value.signature_id.clone(),
    }
}

fn draft_input_from_wire(
    value: wire::MailDraftInputV1,
) -> Result<MailDraftInputV1, MailClientWireErrorV1> {
    Ok(MailDraftInputV1 {
        connection_id: value.connection_id,
        draft_id: value.draft_id,
        mode: mode_from_wire(value.mode)?,
        provider_conversation_id: value.provider_conversation_id,
        in_reply_to_provider_message_id: value.in_reply_to_provider_message_id,
        to_recipients: value.to_recipient,
        cc_recipients: value.cc_recipient,
        bcc_recipients: value.bcc_recipient,
        subject: value.subject,
        text_body: value.text_body,
        template_id: value.template_id,
        signature_id: value.signature_id,
    })
}

fn template_input_to_wire(value: &MailTemplateInputV1) -> wire::MailTemplateInputV1 {
    wire::MailTemplateInputV1 {
        connection_id: value.connection_id.clone(),
        template_id: value.template_id.clone(),
        name: value.name.clone(),
        subject_template: value.subject_template.clone(),
        text_body_template: value.text_body_template.clone(),
        variable: value.variables.clone(),
        locale: value.locale.clone(),
    }
}

fn template_input_from_wire(value: wire::MailTemplateInputV1) -> MailTemplateInputV1 {
    MailTemplateInputV1 {
        connection_id: value.connection_id,
        template_id: value.template_id,
        name: value.name,
        subject_template: value.subject_template,
        text_body_template: value.text_body_template,
        variables: value.variable,
        locale: value.locale,
    }
}

fn signature_input_to_wire(value: &MailSignatureInputV1) -> wire::MailSignatureInputV1 {
    wire::MailSignatureInputV1 {
        connection_id: value.connection_id.clone(),
        signature_id: value.signature_id.clone(),
        name: value.name.clone(),
        text_body: value.text_body.clone(),
        is_default: value.is_default,
    }
}

fn signature_input_from_wire(value: wire::MailSignatureInputV1) -> MailSignatureInputV1 {
    MailSignatureInputV1 {
        connection_id: value.connection_id,
        signature_id: value.signature_id,
        name: value.name,
        text_body: value.text_body,
        is_default: value.is_default,
    }
}

fn draft_to_wire(value: &MailDraftV1) -> wire::MailDraftV1 {
    let input = draft_input_to_wire(&MailDraftInputV1 {
        connection_id: value.connection_id.clone(),
        draft_id: value.draft_id.clone(),
        mode: value.mode,
        provider_conversation_id: value.provider_conversation_id.clone(),
        in_reply_to_provider_message_id: value.in_reply_to_provider_message_id.clone(),
        to_recipients: value.to_recipients.clone(),
        cc_recipients: value.cc_recipients.clone(),
        bcc_recipients: value.bcc_recipients.clone(),
        subject: value.subject.clone(),
        text_body: value.text_body.clone(),
        template_id: value.template_id.clone(),
        signature_id: value.signature_id.clone(),
    });
    wire::MailDraftV1 {
        connection_id: input.connection_id,
        draft_id: input.draft_id,
        revision: value.revision,
        mode: input.mode,
        provider_conversation_id: input.provider_conversation_id,
        in_reply_to_provider_message_id: input.in_reply_to_provider_message_id,
        to_recipient: input.to_recipient,
        cc_recipient: input.cc_recipient,
        bcc_recipient: input.bcc_recipient,
        subject: input.subject,
        text_body: input.text_body,
        template_id: input.template_id,
        signature_id: input.signature_id,
        created_at_unix_seconds: value.created_at_unix_seconds,
        updated_at_unix_seconds: value.updated_at_unix_seconds,
    }
}

fn draft_from_wire(value: wire::MailDraftV1) -> Result<MailDraftV1, MailClientWireErrorV1> {
    Ok(MailDraftV1 {
        connection_id: value.connection_id,
        draft_id: value.draft_id,
        revision: value.revision,
        mode: mode_from_wire(value.mode)?,
        provider_conversation_id: value.provider_conversation_id,
        in_reply_to_provider_message_id: value.in_reply_to_provider_message_id,
        to_recipients: value.to_recipient,
        cc_recipients: value.cc_recipient,
        bcc_recipients: value.bcc_recipient,
        subject: value.subject,
        text_body: value.text_body,
        template_id: value.template_id,
        signature_id: value.signature_id,
        created_at_unix_seconds: value.created_at_unix_seconds,
        updated_at_unix_seconds: value.updated_at_unix_seconds,
    })
}

fn template_to_wire(value: &MailTemplateV1) -> wire::MailTemplateV1 {
    wire::MailTemplateV1 {
        connection_id: value.connection_id.clone(),
        template_id: value.template_id.clone(),
        revision: value.revision,
        name: value.name.clone(),
        subject_template: value.subject_template.clone(),
        text_body_template: value.text_body_template.clone(),
        variable: value.variables.clone(),
        locale: value.locale.clone(),
        created_at_unix_seconds: value.created_at_unix_seconds,
        updated_at_unix_seconds: value.updated_at_unix_seconds,
    }
}

fn template_from_wire(value: wire::MailTemplateV1) -> MailTemplateV1 {
    MailTemplateV1 {
        connection_id: value.connection_id,
        template_id: value.template_id,
        revision: value.revision,
        name: value.name,
        subject_template: value.subject_template,
        text_body_template: value.text_body_template,
        variables: value.variable,
        locale: value.locale,
        created_at_unix_seconds: value.created_at_unix_seconds,
        updated_at_unix_seconds: value.updated_at_unix_seconds,
    }
}

fn signature_to_wire(value: &MailSignatureV1) -> wire::MailSignatureV1 {
    wire::MailSignatureV1 {
        connection_id: value.connection_id.clone(),
        signature_id: value.signature_id.clone(),
        revision: value.revision,
        name: value.name.clone(),
        text_body: value.text_body.clone(),
        is_default: value.is_default,
        created_at_unix_seconds: value.created_at_unix_seconds,
        updated_at_unix_seconds: value.updated_at_unix_seconds,
    }
}

fn signature_from_wire(value: wire::MailSignatureV1) -> MailSignatureV1 {
    MailSignatureV1 {
        connection_id: value.connection_id,
        signature_id: value.signature_id,
        revision: value.revision,
        name: value.name,
        text_body: value.text_body,
        is_default: value.is_default,
        created_at_unix_seconds: value.created_at_unix_seconds,
        updated_at_unix_seconds: value.updated_at_unix_seconds,
    }
}

fn preview_to_wire(value: &MailTemplatePreviewV1) -> wire::MailTemplatePreviewV1 {
    wire::MailTemplatePreviewV1 {
        template_id: value.template_id.clone(),
        subject: value.subject.clone(),
        text_body: value.text_body.clone(),
        missing_variable: value.missing_variables.clone(),
        unresolved_variable: value.unresolved_variables.clone(),
        malformed_placeholder: value.malformed_placeholders.clone(),
        ready: value.ready,
    }
}

fn preview_from_wire(value: wire::MailTemplatePreviewV1) -> MailTemplatePreviewV1 {
    MailTemplatePreviewV1 {
        template_id: value.template_id,
        subject: value.subject,
        text_body: value.text_body,
        missing_variables: value.missing_variable,
        unresolved_variables: value.unresolved_variable,
        malformed_placeholders: value.malformed_placeholder,
        ready: value.ready,
    }
}

const fn mode_to_wire(value: MailCompositionModeV1) -> wire::MailCompositionModeV1 {
    match value {
        MailCompositionModeV1::New => wire::MailCompositionModeV1::MailCompositionModeNew,
        MailCompositionModeV1::Reply => wire::MailCompositionModeV1::MailCompositionModeReply,
        MailCompositionModeV1::ReplyAll => wire::MailCompositionModeV1::MailCompositionModeReplyAll,
        MailCompositionModeV1::Forward => wire::MailCompositionModeV1::MailCompositionModeForward,
        MailCompositionModeV1::Redirect => wire::MailCompositionModeV1::MailCompositionModeRedirect,
    }
}

fn mode_from_wire(value: i32) -> Result<MailCompositionModeV1, MailClientWireErrorV1> {
    match wire::MailCompositionModeV1::try_from(value) {
        Ok(wire::MailCompositionModeV1::MailCompositionModeNew) => Ok(MailCompositionModeV1::New),
        Ok(wire::MailCompositionModeV1::MailCompositionModeReply) => {
            Ok(MailCompositionModeV1::Reply)
        }
        Ok(wire::MailCompositionModeV1::MailCompositionModeReplyAll) => {
            Ok(MailCompositionModeV1::ReplyAll)
        }
        Ok(wire::MailCompositionModeV1::MailCompositionModeForward) => {
            Ok(MailCompositionModeV1::Forward)
        }
        Ok(wire::MailCompositionModeV1::MailCompositionModeRedirect) => {
            Ok(MailCompositionModeV1::Redirect)
        }
        _ => Err(MailClientWireErrorV1::InvalidPayload),
    }
}

const fn entity_kind_to_wire(
    value: MailCompositionEntityKindV1,
) -> wire::MailCompositionEntityKindV1 {
    match value {
        MailCompositionEntityKindV1::Draft => {
            wire::MailCompositionEntityKindV1::MailCompositionEntityKindDraft
        }
        MailCompositionEntityKindV1::Template => {
            wire::MailCompositionEntityKindV1::MailCompositionEntityKindTemplate
        }
        MailCompositionEntityKindV1::Signature => {
            wire::MailCompositionEntityKindV1::MailCompositionEntityKindSignature
        }
    }
}

fn entity_kind_from_wire(value: i32) -> Result<MailCompositionEntityKindV1, MailClientWireErrorV1> {
    match wire::MailCompositionEntityKindV1::try_from(value) {
        Ok(wire::MailCompositionEntityKindV1::MailCompositionEntityKindDraft) => {
            Ok(MailCompositionEntityKindV1::Draft)
        }
        Ok(wire::MailCompositionEntityKindV1::MailCompositionEntityKindTemplate) => {
            Ok(MailCompositionEntityKindV1::Template)
        }
        Ok(wire::MailCompositionEntityKindV1::MailCompositionEntityKindSignature) => {
            Ok(MailCompositionEntityKindV1::Signature)
        }
        _ => Err(MailClientWireErrorV1::InvalidPayload),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_and_query_round_trip_canonically() {
        let command = MailCompositionCommandV1::UpsertSignature {
            operation_id: "operation-1".to_owned(),
            signature: MailSignatureInputV1 {
                connection_id: "account-1".to_owned(),
                signature_id: "signature-1".to_owned(),
                name: "Default".to_owned(),
                text_body: "Regards".to_owned(),
                is_default: true,
            },
            expected_revision: None,
        };
        let bytes = encode_composition_command(&command).expect("encode");
        assert_eq!(decode_composition_command(&bytes), Ok(command));

        let query = MailCompositionQueryV1::PreviewTemplate {
            connection_id: "account-1".to_owned(),
            template_id: "template-1".to_owned(),
            values: vec![MailTemplateVariableValueV1 {
                name: "name".to_owned(),
                value: "Alice".to_owned(),
            }],
        };
        let bytes = encode_composition_query(&query).expect("encode");
        assert_eq!(decode_composition_query(&bytes), Ok(query));
    }

    #[test]
    fn response_and_receipt_round_trip_canonically() {
        let receipt = MailCompositionMutationReceiptV1 {
            operation_id: "operation-1".to_owned(),
            connection_id: "account-1".to_owned(),
            entity_kind: MailCompositionEntityKindV1::Draft,
            entity_id: "draft-1".to_owned(),
            revision: 1,
            deleted: false,
        };
        let bytes = encode_composition_receipt(&receipt).expect("encode");
        assert_eq!(decode_composition_receipt(&bytes), Ok(receipt));

        let response = MailCompositionQueryResponseV1::NotFound;
        let bytes = encode_composition_query_response(&response).expect("encode");
        assert_eq!(decode_composition_query_response(&bytes), Ok(response));
    }
}
