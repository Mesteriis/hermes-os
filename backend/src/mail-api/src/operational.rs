//! Public Mail-owned operational read contract.

pub const MAX_OPERATIONAL_CURSOR_BYTES: usize = 512;
pub const MAX_OPERATIONAL_PAGE_SIZE: u32 = 200;
pub const MAX_OPERATIONAL_ID_BYTES: usize = 512;
pub const MAX_OPERATIONAL_SUBJECT_BYTES: usize = 998;
pub const MAX_OPERATIONAL_SNIPPET_BYTES: usize = 4_096;
pub const MAX_OPERATIONAL_ADDRESS_BYTES: usize = 1_024;
pub const MAX_OPERATIONAL_RECIPIENTS: usize = 256;
pub const MAX_OPERATIONAL_FOLDERS_PER_MESSAGE: usize = 128;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MailOperationalQueryV1 {
    ListFolders {
        connection_id: String,
        cursor: Option<String>,
        limit: u32,
    },
    ListThreads {
        connection_id: String,
        folder_id: Option<String>,
        cursor: Option<String>,
        limit: u32,
    },
    ListMessages {
        connection_id: String,
        folder_id: Option<String>,
        provider_thread_id: Option<String>,
        cursor: Option<String>,
        limit: u32,
    },
    GetMessage {
        connection_id: String,
        provider_message_id: String,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailFolderKindV1 {
    Inbox,
    Sent,
    Drafts,
    Trash,
    Spam,
    Archive,
    ProviderLabel,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailFolderV1 {
    pub connection_id: String,
    pub folder_id: String,
    pub display_name: String,
    pub kind: MailFolderKindV1,
    pub total_messages: u64,
    pub unread_messages: u64,
    pub projection_revision: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailThreadV1 {
    pub connection_id: String,
    pub provider_thread_id: String,
    pub subject: Option<String>,
    pub latest_snippet: Option<String>,
    pub latest_at_unix_seconds: Option<i64>,
    pub message_count: u64,
    pub unread_count: u64,
    pub projection_revision: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailMessageFlagV1 {
    Read,
    Starred,
    Draft,
    Sent,
    Trashed,
    Spam,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailMessageSummaryV1 {
    pub connection_id: String,
    pub provider_message_id: String,
    pub provider_thread_id: String,
    pub folder_ids: Vec<String>,
    pub subject: Option<String>,
    pub sender: Option<String>,
    pub recipients: Vec<String>,
    pub snippet: Option<String>,
    pub sent_at_unix_seconds: Option<i64>,
    pub flags: Vec<MailMessageFlagV1>,
    pub has_plain_text: bool,
    pub has_attachments: bool,
    pub observation_anchor_id: [u8; 16],
    pub projection_revision: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailMessageDetailV1 {
    pub summary: MailMessageSummaryV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailOperationalPageV1<T> {
    pub items: Vec<T>,
    pub next_cursor: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MailOperationalQueryResponseV1 {
    Folders(MailOperationalPageV1<MailFolderV1>),
    Threads(MailOperationalPageV1<MailThreadV1>),
    Messages(MailOperationalPageV1<MailMessageSummaryV1>),
    Message(Box<MailMessageDetailV1>),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailOperationalContractErrorV1 {
    InvalidId,
    InvalidCursor,
    InvalidLimit,
}

#[must_use]
pub fn operational_query_connection_id(query: &MailOperationalQueryV1) -> &str {
    match query {
        MailOperationalQueryV1::ListFolders { connection_id, .. }
        | MailOperationalQueryV1::ListThreads { connection_id, .. }
        | MailOperationalQueryV1::ListMessages { connection_id, .. }
        | MailOperationalQueryV1::GetMessage { connection_id, .. } => connection_id,
    }
}

pub fn validate_operational_query(
    query: &MailOperationalQueryV1,
) -> Result<(), MailOperationalContractErrorV1> {
    validate_id(operational_query_connection_id(query))?;
    match query {
        MailOperationalQueryV1::ListFolders { cursor, limit, .. } => {
            validate_page(cursor.as_deref(), *limit)
        }
        MailOperationalQueryV1::ListThreads {
            folder_id,
            cursor,
            limit,
            ..
        } => {
            validate_optional_id(folder_id.as_deref())?;
            validate_page(cursor.as_deref(), *limit)
        }
        MailOperationalQueryV1::ListMessages {
            folder_id,
            provider_thread_id,
            cursor,
            limit,
            ..
        } => {
            validate_optional_id(folder_id.as_deref())?;
            validate_optional_id(provider_thread_id.as_deref())?;
            validate_page(cursor.as_deref(), *limit)
        }
        MailOperationalQueryV1::GetMessage {
            provider_message_id,
            ..
        } => validate_id(provider_message_id),
    }
}

pub fn validate_operational_message(
    message: &MailMessageSummaryV1,
) -> Result<(), MailOperationalContractErrorV1> {
    validate_id(&message.connection_id)?;
    validate_id(&message.provider_message_id)?;
    validate_id(&message.provider_thread_id)?;
    if message.folder_ids.is_empty()
        || message.folder_ids.len() > MAX_OPERATIONAL_FOLDERS_PER_MESSAGE
        || message
            .folder_ids
            .iter()
            .enumerate()
            .any(|(index, folder_id)| {
                validate_id(folder_id).is_err() || message.folder_ids[..index].contains(folder_id)
            })
        || message
            .subject
            .as_deref()
            .is_some_and(|value| !valid_text(value, MAX_OPERATIONAL_SUBJECT_BYTES))
        || message
            .sender
            .as_deref()
            .is_some_and(|value| !valid_text(value, MAX_OPERATIONAL_ADDRESS_BYTES))
        || message.recipients.len() > MAX_OPERATIONAL_RECIPIENTS
        || message
            .recipients
            .iter()
            .any(|value| !valid_text(value, MAX_OPERATIONAL_ADDRESS_BYTES))
        || message
            .snippet
            .as_deref()
            .is_some_and(|value| !valid_text(value, MAX_OPERATIONAL_SNIPPET_BYTES))
        || message
            .flags
            .iter()
            .enumerate()
            .any(|(index, flag)| message.flags[..index].contains(flag))
        || message.observation_anchor_id.iter().all(|byte| *byte == 0)
        || message.projection_revision == 0
    {
        return Err(MailOperationalContractErrorV1::InvalidId);
    }
    Ok(())
}

fn validate_optional_id(value: Option<&str>) -> Result<(), MailOperationalContractErrorV1> {
    if let Some(value) = value {
        validate_id(value)?;
    }
    Ok(())
}

fn validate_id(value: &str) -> Result<(), MailOperationalContractErrorV1> {
    if value.trim().is_empty()
        || value.len() > MAX_OPERATIONAL_ID_BYTES
        || value.contains(['\0', '\r', '\n'])
    {
        return Err(MailOperationalContractErrorV1::InvalidId);
    }
    Ok(())
}

fn valid_text(value: &str, max_bytes: usize) -> bool {
    !value.is_empty() && value.len() <= max_bytes && !value.contains(['\0', '\r', '\n'])
}

fn validate_page(cursor: Option<&str>, limit: u32) -> Result<(), MailOperationalContractErrorV1> {
    if limit == 0 || limit > MAX_OPERATIONAL_PAGE_SIZE {
        return Err(MailOperationalContractErrorV1::InvalidLimit);
    }
    if cursor.is_some_and(|cursor| {
        cursor.is_empty()
            || cursor.len() > MAX_OPERATIONAL_CURSOR_BYTES
            || cursor.contains(['\0', '\r', '\n'])
    }) {
        return Err(MailOperationalContractErrorV1::InvalidCursor);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_only_bounded_scoped_queries() {
        assert_eq!(
            validate_operational_query(&MailOperationalQueryV1::ListMessages {
                connection_id: "mail-account".into(),
                folder_id: Some("INBOX".into()),
                provider_thread_id: None,
                cursor: Some("opaque-cursor".into()),
                limit: 200,
            }),
            Ok(())
        );
        assert_eq!(
            validate_operational_query(&MailOperationalQueryV1::ListFolders {
                connection_id: "mail-account".into(),
                cursor: None,
                limit: 0,
            }),
            Err(MailOperationalContractErrorV1::InvalidLimit)
        );
        assert_eq!(
            validate_operational_query(&MailOperationalQueryV1::GetMessage {
                connection_id: "mail-account".into(),
                provider_message_id: "bad\nmessage".into(),
            }),
            Err(MailOperationalContractErrorV1::InvalidId)
        );
    }

    #[test]
    fn operational_messages_are_bounded_and_have_one_nonzero_anchor() {
        let message = MailMessageSummaryV1 {
            connection_id: "mail-account".into(),
            provider_message_id: "message-1".into(),
            provider_thread_id: "thread-1".into(),
            folder_ids: vec!["INBOX".into()],
            subject: Some("Subject".into()),
            sender: Some("sender@example.test".into()),
            recipients: vec!["owner@example.test".into()],
            snippet: Some("Plain text preview".into()),
            sent_at_unix_seconds: Some(1),
            flags: vec![MailMessageFlagV1::Read],
            has_plain_text: true,
            has_attachments: false,
            observation_anchor_id: [1; 16],
            projection_revision: 1,
        };
        assert_eq!(validate_operational_message(&message), Ok(()));

        let mut duplicate_folders = message.clone();
        duplicate_folders.folder_ids.push("INBOX".into());
        assert_eq!(
            validate_operational_message(&duplicate_folders),
            Err(MailOperationalContractErrorV1::InvalidId)
        );
        let mut missing_anchor = message;
        missing_anchor.observation_anchor_id = [0; 16];
        assert_eq!(
            validate_operational_message(&missing_anchor),
            Err(MailOperationalContractErrorV1::InvalidId)
        );
    }
}
