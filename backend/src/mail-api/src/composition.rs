use std::collections::{BTreeMap, BTreeSet};

use crate::{MAX_PLAIN_TEXT_BYTES, valid_mailbox};

pub const MAX_COMPOSITION_ID_BYTES: usize = 512;
pub const MAX_COMPOSITION_CURSOR_BYTES: usize = 512;
pub const MAX_COMPOSITION_PAGE_SIZE: u32 = 100;
pub const MAX_COMPOSITION_RECIPIENTS_PER_KIND: usize = 100;
pub const MAX_COMPOSITION_SUBJECT_BYTES: usize = 998;
pub const MAX_COMPOSITION_NAME_BYTES: usize = 256;
pub const MAX_COMPOSITION_LOCALE_BYTES: usize = 64;
pub const MAX_COMPOSITION_TEMPLATE_VARIABLES: usize = 64;
pub const MAX_COMPOSITION_VARIABLE_NAME_BYTES: usize = 128;
pub const MAX_COMPOSITION_VARIABLE_VALUE_BYTES: usize = 16 * 1024;
pub const MAX_COMPOSITION_MALFORMED_PLACEHOLDERS: usize = 64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailCompositionModeV1 {
    New,
    Reply,
    ReplyAll,
    Forward,
    Redirect,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailCompositionEntityKindV1 {
    Draft,
    Template,
    Signature,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailDraftInputV1 {
    pub connection_id: String,
    pub draft_id: String,
    pub mode: MailCompositionModeV1,
    pub provider_conversation_id: Option<String>,
    pub in_reply_to_provider_message_id: Option<String>,
    pub to_recipients: Vec<String>,
    pub cc_recipients: Vec<String>,
    pub bcc_recipients: Vec<String>,
    pub subject: String,
    pub text_body: String,
    pub template_id: Option<String>,
    pub signature_id: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailTemplateInputV1 {
    pub connection_id: String,
    pub template_id: String,
    pub name: String,
    pub subject_template: String,
    pub text_body_template: String,
    pub variables: Vec<String>,
    pub locale: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailSignatureInputV1 {
    pub connection_id: String,
    pub signature_id: String,
    pub name: String,
    pub text_body: String,
    pub is_default: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MailCompositionCommandV1 {
    UpsertDraft {
        operation_id: String,
        draft: MailDraftInputV1,
        expected_revision: Option<u64>,
    },
    DeleteDraft {
        operation_id: String,
        connection_id: String,
        draft_id: String,
        expected_revision: u64,
    },
    UpsertTemplate {
        operation_id: String,
        template: MailTemplateInputV1,
        expected_revision: Option<u64>,
    },
    DeleteTemplate {
        operation_id: String,
        connection_id: String,
        template_id: String,
        expected_revision: u64,
    },
    UpsertSignature {
        operation_id: String,
        signature: MailSignatureInputV1,
        expected_revision: Option<u64>,
    },
    DeleteSignature {
        operation_id: String,
        connection_id: String,
        signature_id: String,
        expected_revision: u64,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailCompositionMutationReceiptV1 {
    pub operation_id: String,
    pub connection_id: String,
    pub entity_kind: MailCompositionEntityKindV1,
    pub entity_id: String,
    pub revision: u64,
    pub deleted: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MailCompositionQueryV1 {
    ListDrafts {
        connection_id: String,
        cursor: Option<String>,
        limit: u32,
    },
    GetDraft {
        connection_id: String,
        draft_id: String,
    },
    ListTemplates {
        connection_id: String,
        cursor: Option<String>,
        limit: u32,
    },
    GetTemplate {
        connection_id: String,
        template_id: String,
    },
    PreviewTemplate {
        connection_id: String,
        template_id: String,
        values: Vec<MailTemplateVariableValueV1>,
    },
    ListSignatures {
        connection_id: String,
        cursor: Option<String>,
        limit: u32,
    },
    GetSignature {
        connection_id: String,
        signature_id: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailTemplateVariableValueV1 {
    pub name: String,
    pub value: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailDraftV1 {
    pub connection_id: String,
    pub draft_id: String,
    pub revision: u64,
    pub mode: MailCompositionModeV1,
    pub provider_conversation_id: Option<String>,
    pub in_reply_to_provider_message_id: Option<String>,
    pub to_recipients: Vec<String>,
    pub cc_recipients: Vec<String>,
    pub bcc_recipients: Vec<String>,
    pub subject: String,
    pub text_body: String,
    pub template_id: Option<String>,
    pub signature_id: Option<String>,
    pub created_at_unix_seconds: i64,
    pub updated_at_unix_seconds: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailTemplateV1 {
    pub connection_id: String,
    pub template_id: String,
    pub revision: u64,
    pub name: String,
    pub subject_template: String,
    pub text_body_template: String,
    pub variables: Vec<String>,
    pub locale: Option<String>,
    pub created_at_unix_seconds: i64,
    pub updated_at_unix_seconds: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailSignatureV1 {
    pub connection_id: String,
    pub signature_id: String,
    pub revision: u64,
    pub name: String,
    pub text_body: String,
    pub is_default: bool,
    pub created_at_unix_seconds: i64,
    pub updated_at_unix_seconds: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailCompositionPageV1<T> {
    pub items: Vec<T>,
    pub next_cursor: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailTemplatePreviewV1 {
    pub template_id: String,
    pub subject: String,
    pub text_body: String,
    pub missing_variables: Vec<String>,
    pub unresolved_variables: Vec<String>,
    pub malformed_placeholders: Vec<String>,
    pub ready: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MailCompositionQueryResponseV1 {
    Drafts(MailCompositionPageV1<MailDraftV1>),
    Draft(MailDraftV1),
    Templates(MailCompositionPageV1<MailTemplateV1>),
    Template(MailTemplateV1),
    TemplatePreview(MailTemplatePreviewV1),
    Signatures(MailCompositionPageV1<MailSignatureV1>),
    Signature(MailSignatureV1),
    NotFound,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailCompositionContractErrorV1 {
    InvalidId,
    InvalidCursor,
    InvalidLimit,
    InvalidRevision,
    InvalidRecipient,
    InvalidText,
    InvalidVariable,
    InvalidEntity,
    InvalidResponse,
}

#[must_use]
pub fn composition_command_connection_id(command: &MailCompositionCommandV1) -> &str {
    match command {
        MailCompositionCommandV1::UpsertDraft { draft, .. } => &draft.connection_id,
        MailCompositionCommandV1::DeleteDraft { connection_id, .. }
        | MailCompositionCommandV1::DeleteTemplate { connection_id, .. }
        | MailCompositionCommandV1::DeleteSignature { connection_id, .. } => connection_id,
        MailCompositionCommandV1::UpsertTemplate { template, .. } => &template.connection_id,
        MailCompositionCommandV1::UpsertSignature { signature, .. } => &signature.connection_id,
    }
}

#[must_use]
pub fn composition_command_operation_id(command: &MailCompositionCommandV1) -> &str {
    match command {
        MailCompositionCommandV1::UpsertDraft { operation_id, .. }
        | MailCompositionCommandV1::DeleteDraft { operation_id, .. }
        | MailCompositionCommandV1::UpsertTemplate { operation_id, .. }
        | MailCompositionCommandV1::DeleteTemplate { operation_id, .. }
        | MailCompositionCommandV1::UpsertSignature { operation_id, .. }
        | MailCompositionCommandV1::DeleteSignature { operation_id, .. } => operation_id,
    }
}

#[must_use]
pub fn composition_query_connection_id(query: &MailCompositionQueryV1) -> &str {
    match query {
        MailCompositionQueryV1::ListDrafts { connection_id, .. }
        | MailCompositionQueryV1::GetDraft { connection_id, .. }
        | MailCompositionQueryV1::ListTemplates { connection_id, .. }
        | MailCompositionQueryV1::GetTemplate { connection_id, .. }
        | MailCompositionQueryV1::PreviewTemplate { connection_id, .. }
        | MailCompositionQueryV1::ListSignatures { connection_id, .. }
        | MailCompositionQueryV1::GetSignature { connection_id, .. } => connection_id,
    }
}

pub fn validate_composition_command(
    command: &MailCompositionCommandV1,
) -> Result<(), MailCompositionContractErrorV1> {
    validate_id(composition_command_operation_id(command))?;
    match command {
        MailCompositionCommandV1::UpsertDraft {
            draft,
            expected_revision,
            ..
        } => {
            validate_draft_input(draft)?;
            validate_optional_revision(*expected_revision)
        }
        MailCompositionCommandV1::DeleteDraft {
            connection_id,
            draft_id,
            expected_revision,
            ..
        } => validate_delete(connection_id, draft_id, *expected_revision),
        MailCompositionCommandV1::UpsertTemplate {
            template,
            expected_revision,
            ..
        } => {
            validate_template_input(template)?;
            validate_optional_revision(*expected_revision)
        }
        MailCompositionCommandV1::DeleteTemplate {
            connection_id,
            template_id,
            expected_revision,
            ..
        } => validate_delete(connection_id, template_id, *expected_revision),
        MailCompositionCommandV1::UpsertSignature {
            signature,
            expected_revision,
            ..
        } => {
            validate_signature_input(signature)?;
            validate_optional_revision(*expected_revision)
        }
        MailCompositionCommandV1::DeleteSignature {
            connection_id,
            signature_id,
            expected_revision,
            ..
        } => validate_delete(connection_id, signature_id, *expected_revision),
    }
}

pub fn validate_composition_query(
    query: &MailCompositionQueryV1,
) -> Result<(), MailCompositionContractErrorV1> {
    validate_id(composition_query_connection_id(query))?;
    match query {
        MailCompositionQueryV1::ListDrafts { cursor, limit, .. }
        | MailCompositionQueryV1::ListTemplates { cursor, limit, .. }
        | MailCompositionQueryV1::ListSignatures { cursor, limit, .. } => {
            validate_cursor(cursor.as_deref())?;
            if !(1..=MAX_COMPOSITION_PAGE_SIZE).contains(limit) {
                return Err(MailCompositionContractErrorV1::InvalidLimit);
            }
            Ok(())
        }
        MailCompositionQueryV1::GetDraft { draft_id, .. } => validate_id(draft_id),
        MailCompositionQueryV1::GetTemplate { template_id, .. }
        | MailCompositionQueryV1::GetSignature {
            signature_id: template_id,
            ..
        } => validate_id(template_id),
        MailCompositionQueryV1::PreviewTemplate {
            template_id,
            values,
            ..
        } => {
            validate_id(template_id)?;
            validate_variable_values(values)
        }
    }
}

pub fn validate_composition_receipt(
    receipt: &MailCompositionMutationReceiptV1,
) -> Result<(), MailCompositionContractErrorV1> {
    validate_id(&receipt.operation_id)?;
    validate_id(&receipt.connection_id)?;
    validate_id(&receipt.entity_id)?;
    if receipt.revision == 0 {
        return Err(MailCompositionContractErrorV1::InvalidRevision);
    }
    Ok(())
}

pub fn validate_composition_response(
    response: &MailCompositionQueryResponseV1,
) -> Result<(), MailCompositionContractErrorV1> {
    match response {
        MailCompositionQueryResponseV1::Drafts(page) => {
            validate_page(page, validate_draft)?;
        }
        MailCompositionQueryResponseV1::Draft(value) => validate_draft(value)?,
        MailCompositionQueryResponseV1::Templates(page) => {
            validate_page(page, validate_template)?;
        }
        MailCompositionQueryResponseV1::Template(value) => validate_template(value)?,
        MailCompositionQueryResponseV1::TemplatePreview(value) => {
            validate_template_preview(value)?;
        }
        MailCompositionQueryResponseV1::Signatures(page) => {
            validate_page(page, validate_signature)?;
        }
        MailCompositionQueryResponseV1::Signature(value) => validate_signature(value)?,
        MailCompositionQueryResponseV1::NotFound => {}
    }
    Ok(())
}

pub fn validate_draft_input(
    draft: &MailDraftInputV1,
) -> Result<(), MailCompositionContractErrorV1> {
    validate_id(&draft.connection_id)?;
    validate_id(&draft.draft_id)?;
    validate_optional_id(draft.provider_conversation_id.as_deref())?;
    validate_optional_id(draft.in_reply_to_provider_message_id.as_deref())?;
    validate_optional_id(draft.template_id.as_deref())?;
    validate_optional_id(draft.signature_id.as_deref())?;
    validate_recipients(&draft.to_recipients)?;
    validate_recipients(&draft.cc_recipients)?;
    validate_recipients(&draft.bcc_recipients)?;
    validate_text(&draft.subject, MAX_COMPOSITION_SUBJECT_BYTES, true)?;
    validate_text(&draft.text_body, MAX_PLAIN_TEXT_BYTES, true)?;
    Ok(())
}

pub fn validate_template_input(
    template: &MailTemplateInputV1,
) -> Result<(), MailCompositionContractErrorV1> {
    validate_id(&template.connection_id)?;
    validate_id(&template.template_id)?;
    validate_text(&template.name, MAX_COMPOSITION_NAME_BYTES, false)?;
    validate_text(
        &template.subject_template,
        MAX_COMPOSITION_SUBJECT_BYTES,
        true,
    )?;
    validate_text(&template.text_body_template, MAX_PLAIN_TEXT_BYTES, true)?;
    validate_variables(&template.variables)?;
    if let Some(locale) = template.locale.as_deref() {
        validate_text(locale, MAX_COMPOSITION_LOCALE_BYTES, false)?;
    }
    let validation = inspect_template(
        &template.subject_template,
        &template.text_body_template,
        &template.variables,
    );
    if !validation.undeclared_variables.is_empty() || !validation.malformed_placeholders.is_empty()
    {
        return Err(MailCompositionContractErrorV1::InvalidVariable);
    }
    Ok(())
}

pub fn validate_signature_input(
    signature: &MailSignatureInputV1,
) -> Result<(), MailCompositionContractErrorV1> {
    validate_id(&signature.connection_id)?;
    validate_id(&signature.signature_id)?;
    validate_text(&signature.name, MAX_COMPOSITION_NAME_BYTES, false)?;
    validate_text(&signature.text_body, MAX_PLAIN_TEXT_BYTES, true)
}

pub fn render_mail_template_preview(
    template: &MailTemplateV1,
    values: &[MailTemplateVariableValueV1],
) -> Result<MailTemplatePreviewV1, MailCompositionContractErrorV1> {
    validate_template(template)?;
    validate_variable_values(values)?;
    let values = values
        .iter()
        .map(|value| (value.name.as_str(), value.value.as_str()))
        .collect::<BTreeMap<_, _>>();
    let subject = render_template_text(&template.subject_template, &values);
    let body = render_template_text(&template.text_body_template, &values);
    let missing_variables = template
        .variables
        .iter()
        .filter(|variable| {
            values
                .get(variable.as_str())
                .map(|value| value.trim().is_empty())
                .unwrap_or(true)
        })
        .cloned()
        .collect::<Vec<_>>();
    let unresolved_variables =
        unique_strings(subject.unresolved.iter().chain(body.unresolved.iter()));
    let malformed_placeholders =
        unique_strings(subject.malformed.iter().chain(body.malformed.iter()));
    let ready = missing_variables.is_empty()
        && unresolved_variables.is_empty()
        && malformed_placeholders.is_empty();
    Ok(MailTemplatePreviewV1 {
        template_id: template.template_id.clone(),
        subject: subject.text,
        text_body: body.text,
        missing_variables,
        unresolved_variables,
        malformed_placeholders,
        ready,
    })
}

fn validate_draft(draft: &MailDraftV1) -> Result<(), MailCompositionContractErrorV1> {
    validate_draft_input(&MailDraftInputV1 {
        connection_id: draft.connection_id.clone(),
        draft_id: draft.draft_id.clone(),
        mode: draft.mode,
        provider_conversation_id: draft.provider_conversation_id.clone(),
        in_reply_to_provider_message_id: draft.in_reply_to_provider_message_id.clone(),
        to_recipients: draft.to_recipients.clone(),
        cc_recipients: draft.cc_recipients.clone(),
        bcc_recipients: draft.bcc_recipients.clone(),
        subject: draft.subject.clone(),
        text_body: draft.text_body.clone(),
        template_id: draft.template_id.clone(),
        signature_id: draft.signature_id.clone(),
    })?;
    validate_entity_revision_and_timestamps(
        draft.revision,
        draft.created_at_unix_seconds,
        draft.updated_at_unix_seconds,
    )
}

fn validate_template(template: &MailTemplateV1) -> Result<(), MailCompositionContractErrorV1> {
    validate_template_input(&MailTemplateInputV1 {
        connection_id: template.connection_id.clone(),
        template_id: template.template_id.clone(),
        name: template.name.clone(),
        subject_template: template.subject_template.clone(),
        text_body_template: template.text_body_template.clone(),
        variables: template.variables.clone(),
        locale: template.locale.clone(),
    })?;
    validate_entity_revision_and_timestamps(
        template.revision,
        template.created_at_unix_seconds,
        template.updated_at_unix_seconds,
    )
}

fn validate_signature(signature: &MailSignatureV1) -> Result<(), MailCompositionContractErrorV1> {
    validate_signature_input(&MailSignatureInputV1 {
        connection_id: signature.connection_id.clone(),
        signature_id: signature.signature_id.clone(),
        name: signature.name.clone(),
        text_body: signature.text_body.clone(),
        is_default: signature.is_default,
    })?;
    validate_entity_revision_and_timestamps(
        signature.revision,
        signature.created_at_unix_seconds,
        signature.updated_at_unix_seconds,
    )
}

fn validate_template_preview(
    preview: &MailTemplatePreviewV1,
) -> Result<(), MailCompositionContractErrorV1> {
    validate_id(&preview.template_id)?;
    validate_text(&preview.subject, MAX_COMPOSITION_SUBJECT_BYTES, true)?;
    validate_text(&preview.text_body, MAX_PLAIN_TEXT_BYTES, true)?;
    validate_diagnostic_variables(&preview.missing_variables)?;
    validate_diagnostic_variables(&preview.unresolved_variables)?;
    if preview.malformed_placeholders.len() > MAX_COMPOSITION_MALFORMED_PLACEHOLDERS
        || preview
            .malformed_placeholders
            .iter()
            .any(|value| validate_text(value, MAX_COMPOSITION_VARIABLE_VALUE_BYTES, false).is_err())
    {
        return Err(MailCompositionContractErrorV1::InvalidResponse);
    }
    let expected_ready = preview.missing_variables.is_empty()
        && preview.unresolved_variables.is_empty()
        && preview.malformed_placeholders.is_empty();
    if preview.ready != expected_ready {
        return Err(MailCompositionContractErrorV1::InvalidResponse);
    }
    Ok(())
}

fn validate_page<T>(
    page: &MailCompositionPageV1<T>,
    validate_item: impl Fn(&T) -> Result<(), MailCompositionContractErrorV1>,
) -> Result<(), MailCompositionContractErrorV1> {
    if page.items.len() > MAX_COMPOSITION_PAGE_SIZE as usize {
        return Err(MailCompositionContractErrorV1::InvalidResponse);
    }
    validate_cursor(page.next_cursor.as_deref())?;
    page.items.iter().try_for_each(validate_item)
}

fn validate_delete(
    connection_id: &str,
    entity_id: &str,
    expected_revision: u64,
) -> Result<(), MailCompositionContractErrorV1> {
    validate_id(connection_id)?;
    validate_id(entity_id)?;
    if expected_revision == 0 {
        return Err(MailCompositionContractErrorV1::InvalidRevision);
    }
    Ok(())
}

fn validate_optional_revision(revision: Option<u64>) -> Result<(), MailCompositionContractErrorV1> {
    if revision == Some(0) {
        return Err(MailCompositionContractErrorV1::InvalidRevision);
    }
    Ok(())
}

fn validate_entity_revision_and_timestamps(
    revision: u64,
    created_at_unix_seconds: i64,
    updated_at_unix_seconds: i64,
) -> Result<(), MailCompositionContractErrorV1> {
    if revision == 0
        || created_at_unix_seconds <= 0
        || updated_at_unix_seconds < created_at_unix_seconds
    {
        return Err(MailCompositionContractErrorV1::InvalidEntity);
    }
    Ok(())
}

fn validate_recipients(values: &[String]) -> Result<(), MailCompositionContractErrorV1> {
    if values.len() > MAX_COMPOSITION_RECIPIENTS_PER_KIND
        || values.iter().any(|value| !valid_mailbox(value))
    {
        return Err(MailCompositionContractErrorV1::InvalidRecipient);
    }
    Ok(())
}

fn validate_variables(values: &[String]) -> Result<(), MailCompositionContractErrorV1> {
    if values.len() > MAX_COMPOSITION_TEMPLATE_VARIABLES {
        return Err(MailCompositionContractErrorV1::InvalidVariable);
    }
    let mut unique = BTreeSet::new();
    for value in values {
        if !valid_variable_name(value) || !unique.insert(value) {
            return Err(MailCompositionContractErrorV1::InvalidVariable);
        }
    }
    Ok(())
}

fn validate_variable_values(
    values: &[MailTemplateVariableValueV1],
) -> Result<(), MailCompositionContractErrorV1> {
    if values.len() > MAX_COMPOSITION_TEMPLATE_VARIABLES {
        return Err(MailCompositionContractErrorV1::InvalidVariable);
    }
    let mut unique = BTreeSet::new();
    for value in values {
        if !valid_variable_name(&value.name)
            || value.value.as_bytes().len() > MAX_COMPOSITION_VARIABLE_VALUE_BYTES
            || value.value.chars().any(char::is_control)
            || !unique.insert(&value.name)
        {
            return Err(MailCompositionContractErrorV1::InvalidVariable);
        }
    }
    Ok(())
}

fn validate_diagnostic_variables(values: &[String]) -> Result<(), MailCompositionContractErrorV1> {
    if values.len() > MAX_COMPOSITION_TEMPLATE_VARIABLES
        || values.iter().any(|value| !valid_variable_name(value))
    {
        return Err(MailCompositionContractErrorV1::InvalidResponse);
    }
    Ok(())
}

fn validate_optional_id(value: Option<&str>) -> Result<(), MailCompositionContractErrorV1> {
    value.map_or(Ok(()), validate_id)
}

fn validate_id(value: &str) -> Result<(), MailCompositionContractErrorV1> {
    if value.is_empty()
        || value.as_bytes().len() > MAX_COMPOSITION_ID_BYTES
        || value.chars().any(char::is_control)
    {
        return Err(MailCompositionContractErrorV1::InvalidId);
    }
    Ok(())
}

fn validate_cursor(value: Option<&str>) -> Result<(), MailCompositionContractErrorV1> {
    if value.is_some_and(|value| {
        value.is_empty()
            || value.as_bytes().len() > MAX_COMPOSITION_CURSOR_BYTES
            || value.chars().any(char::is_control)
    }) {
        return Err(MailCompositionContractErrorV1::InvalidCursor);
    }
    Ok(())
}

fn validate_text(
    value: &str,
    max_bytes: usize,
    allow_empty: bool,
) -> Result<(), MailCompositionContractErrorV1> {
    if (!allow_empty && value.trim().is_empty())
        || value.as_bytes().len() > max_bytes
        || value
            .chars()
            .any(|character| character.is_control() && !matches!(character, '\n' | '\r' | '\t'))
    {
        return Err(MailCompositionContractErrorV1::InvalidText);
    }
    Ok(())
}

fn valid_variable_name(value: &str) -> bool {
    !value.is_empty()
        && value.as_bytes().len() <= MAX_COMPOSITION_VARIABLE_NAME_BYTES
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'-'))
}

struct TemplateInspection {
    undeclared_variables: Vec<String>,
    malformed_placeholders: Vec<String>,
}

fn inspect_template(subject: &str, body: &str, declared: &[String]) -> TemplateInspection {
    let empty = BTreeMap::new();
    let subject = render_template_text(subject, &empty);
    let body = render_template_text(body, &empty);
    let placeholders = unique_strings(subject.unresolved.iter().chain(body.unresolved.iter()));
    let malformed_placeholders =
        unique_strings(subject.malformed.iter().chain(body.malformed.iter()));
    let undeclared_variables = placeholders
        .into_iter()
        .filter(|placeholder| !declared.contains(placeholder))
        .collect();
    TemplateInspection {
        undeclared_variables,
        malformed_placeholders,
    }
}

struct RenderedTemplateText {
    text: String,
    unresolved: Vec<String>,
    malformed: Vec<String>,
}

fn render_template_text(template: &str, values: &BTreeMap<&str, &str>) -> RenderedTemplateText {
    let mut text = String::with_capacity(template.len());
    let mut unresolved = Vec::new();
    let mut malformed = Vec::new();
    let mut rest = template;
    while let Some(start) = rest.find("{{") {
        text.push_str(&rest[..start]);
        let after_open = &rest[start + 2..];
        let Some(end) = after_open.find("}}") else {
            let value = &rest[start..];
            text.push_str(value);
            push_unique(&mut malformed, value);
            return RenderedTemplateText {
                text,
                unresolved,
                malformed,
            };
        };
        let original = &rest[start..start + 2 + end + 2];
        let name = after_open[..end].trim();
        if !valid_variable_name(name) {
            text.push_str(original);
            push_unique(&mut malformed, original);
        } else if let Some(value) = values.get(name).filter(|value| !value.trim().is_empty()) {
            text.push_str(value);
        } else {
            text.push_str(original);
            push_unique(&mut unresolved, name);
        }
        rest = &after_open[end + 2..];
    }
    text.push_str(rest);
    RenderedTemplateText {
        text,
        unresolved,
        malformed,
    }
}

fn push_unique(values: &mut Vec<String>, value: &str) {
    if !values.iter().any(|existing| existing == value) {
        values.push(value.to_owned());
    }
}

fn unique_strings<'a>(values: impl Iterator<Item = &'a String>) -> Vec<String> {
    let mut unique = Vec::new();
    for value in values {
        push_unique(&mut unique, value);
    }
    unique
}

#[cfg(test)]
mod tests {
    use super::*;

    fn template() -> MailTemplateV1 {
        MailTemplateV1 {
            connection_id: "mail-account".to_owned(),
            template_id: "template-1".to_owned(),
            revision: 1,
            name: "Greeting".to_owned(),
            subject_template: "Hello {{name}}".to_owned(),
            text_body_template: "Hi {{name}}, {{message}}".to_owned(),
            variables: vec!["name".to_owned(), "message".to_owned()],
            locale: Some("en".to_owned()),
            created_at_unix_seconds: 1,
            updated_at_unix_seconds: 1,
        }
    }

    #[test]
    fn template_preview_is_bounded_and_reports_missing_values() {
        let preview = render_mail_template_preview(
            &template(),
            &[MailTemplateVariableValueV1 {
                name: "name".to_owned(),
                value: "Alice".to_owned(),
            }],
        )
        .expect("preview");
        assert_eq!(preview.subject, "Hello Alice");
        assert_eq!(preview.text_body, "Hi Alice, {{message}}");
        assert_eq!(preview.missing_variables, ["message"]);
        assert_eq!(preview.unresolved_variables, ["message"]);
        assert!(!preview.ready);
    }

    #[test]
    fn template_contract_rejects_undeclared_or_malformed_placeholders() {
        let mut value = template();
        value.subject_template = "Hello {{unknown}}".to_owned();
        assert_eq!(
            validate_template(&value),
            Err(MailCompositionContractErrorV1::InvalidVariable)
        );
        value.subject_template = "Hello {{".to_owned();
        assert_eq!(
            validate_template(&value),
            Err(MailCompositionContractErrorV1::InvalidVariable)
        );
    }

    #[test]
    fn mutation_requires_positive_expected_revision_and_valid_recipient() {
        let command = MailCompositionCommandV1::DeleteDraft {
            operation_id: "operation-1".to_owned(),
            connection_id: "mail-account".to_owned(),
            draft_id: "draft-1".to_owned(),
            expected_revision: 0,
        };
        assert_eq!(
            validate_composition_command(&command),
            Err(MailCompositionContractErrorV1::InvalidRevision)
        );
        let mut draft = MailDraftInputV1 {
            connection_id: "mail-account".to_owned(),
            draft_id: "draft-1".to_owned(),
            mode: MailCompositionModeV1::New,
            provider_conversation_id: None,
            in_reply_to_provider_message_id: None,
            to_recipients: vec!["not-an-address".to_owned()],
            cc_recipients: Vec::new(),
            bcc_recipients: Vec::new(),
            subject: String::new(),
            text_body: String::new(),
            template_id: None,
            signature_id: None,
        };
        assert_eq!(
            validate_draft_input(&draft),
            Err(MailCompositionContractErrorV1::InvalidRecipient)
        );
        draft.to_recipients = Vec::new();
        assert_eq!(validate_draft_input(&draft), Ok(()));
    }
}
