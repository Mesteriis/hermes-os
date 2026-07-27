//! Public Mail-owned operational read contract.

pub const MAX_OPERATIONAL_CURSOR_BYTES: usize = 512;
pub const MAX_OPERATIONAL_PAGE_SIZE: u32 = 200;
pub const MAX_OPERATIONAL_ID_BYTES: usize = 512;
pub const MAX_OPERATIONAL_SUBJECT_BYTES: usize = 998;
pub const MAX_OPERATIONAL_SNIPPET_BYTES: usize = 4_096;
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
}
