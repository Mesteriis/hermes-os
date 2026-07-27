//! Exact Protobuf mapping for the Mail-owned operational query contract.

use prost::Message;

use crate::{
    client_wire::MailClientWireErrorV1,
    operational::{
        MailFolderKindV1, MailFolderV1, MailMessageDetailV1, MailMessageFlagV1,
        MailMessageSummaryV1, MailOperationalPageV1, MailOperationalQueryResponseV1,
        MailOperationalQueryV1, MailThreadV1, validate_operational_query,
        validate_operational_response,
    },
    operational_wire_generated as wire,
};

pub fn encode_operational_query(
    query: &MailOperationalQueryV1,
) -> Result<Vec<u8>, MailClientWireErrorV1> {
    validate_operational_query(query).map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    use wire::mail_operational_query_v1::Query;
    let query = match query {
        MailOperationalQueryV1::ListFolders {
            connection_id,
            cursor,
            limit,
        } => Query::ListFolders(wire::ListMailFoldersQueryV1 {
            connection_id: connection_id.clone(),
            cursor: cursor.clone(),
            limit: *limit,
        }),
        MailOperationalQueryV1::ListThreads {
            connection_id,
            folder_id,
            cursor,
            limit,
        } => Query::ListThreads(wire::ListMailThreadsQueryV1 {
            connection_id: connection_id.clone(),
            folder_id: folder_id.clone(),
            cursor: cursor.clone(),
            limit: *limit,
        }),
        MailOperationalQueryV1::ListMessages {
            connection_id,
            folder_id,
            provider_thread_id,
            cursor,
            limit,
        } => Query::ListMessages(wire::ListMailMessagesQueryV1 {
            connection_id: connection_id.clone(),
            folder_id: folder_id.clone(),
            provider_thread_id: provider_thread_id.clone(),
            cursor: cursor.clone(),
            limit: *limit,
        }),
        MailOperationalQueryV1::GetMessage {
            connection_id,
            provider_message_id,
        } => Query::GetMessage(wire::GetMailMessageQueryV1 {
            connection_id: connection_id.clone(),
            provider_message_id: provider_message_id.clone(),
        }),
    };
    Ok(wire::MailOperationalQueryV1 { query: Some(query) }.encode_to_vec())
}

pub fn decode_operational_query(
    bytes: &[u8],
) -> Result<MailOperationalQueryV1, MailClientWireErrorV1> {
    use wire::mail_operational_query_v1::Query;
    let query = wire::MailOperationalQueryV1::decode(bytes)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?
        .query
        .ok_or(MailClientWireErrorV1::InvalidPayload)?;
    let query = match query {
        Query::ListFolders(value) => MailOperationalQueryV1::ListFolders {
            connection_id: value.connection_id,
            cursor: value.cursor,
            limit: value.limit,
        },
        Query::ListThreads(value) => MailOperationalQueryV1::ListThreads {
            connection_id: value.connection_id,
            folder_id: value.folder_id,
            cursor: value.cursor,
            limit: value.limit,
        },
        Query::ListMessages(value) => MailOperationalQueryV1::ListMessages {
            connection_id: value.connection_id,
            folder_id: value.folder_id,
            provider_thread_id: value.provider_thread_id,
            cursor: value.cursor,
            limit: value.limit,
        },
        Query::GetMessage(value) => MailOperationalQueryV1::GetMessage {
            connection_id: value.connection_id,
            provider_message_id: value.provider_message_id,
        },
    };
    validate_operational_query(&query).map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    if encode_operational_query(&query)? != bytes {
        return Err(MailClientWireErrorV1::InvalidPayload);
    }
    Ok(query)
}

pub fn encode_operational_query_response(
    response: &MailOperationalQueryResponseV1,
) -> Result<Vec<u8>, MailClientWireErrorV1> {
    validate_operational_response(response).map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    use wire::mail_operational_query_response_v1::Response;
    let response = match response {
        MailOperationalQueryResponseV1::Folders(page) => {
            Response::Folders(wire::MailFolderPageV1 {
                item: page.items.iter().map(folder_to_wire).collect(),
                next_cursor: page.next_cursor.clone(),
            })
        }
        MailOperationalQueryResponseV1::Threads(page) => {
            Response::Threads(wire::MailThreadPageV1 {
                item: page.items.iter().map(thread_to_wire).collect(),
                next_cursor: page.next_cursor.clone(),
            })
        }
        MailOperationalQueryResponseV1::Messages(page) => {
            Response::Messages(wire::MailMessagePageV1 {
                item: page.items.iter().map(message_to_wire).collect(),
                next_cursor: page.next_cursor.clone(),
            })
        }
        MailOperationalQueryResponseV1::Message(message) => {
            Response::Message(Box::new(wire::MailMessageDetailV1 {
                summary: Some(message_to_wire(&message.summary)),
            }))
        }
    };
    Ok(wire::MailOperationalQueryResponseV1 {
        response: Some(response),
    }
    .encode_to_vec())
}

pub fn decode_operational_query_response(
    bytes: &[u8],
) -> Result<MailOperationalQueryResponseV1, MailClientWireErrorV1> {
    use wire::mail_operational_query_response_v1::Response;
    let response = wire::MailOperationalQueryResponseV1::decode(bytes)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?
        .response
        .ok_or(MailClientWireErrorV1::InvalidPayload)?;
    let response = match response {
        Response::Folders(page) => MailOperationalQueryResponseV1::Folders(MailOperationalPageV1 {
            items: page
                .item
                .into_iter()
                .map(folder_from_wire)
                .collect::<Result<_, _>>()?,
            next_cursor: page.next_cursor,
        }),
        Response::Threads(page) => MailOperationalQueryResponseV1::Threads(MailOperationalPageV1 {
            items: page.item.into_iter().map(thread_from_wire).collect(),
            next_cursor: page.next_cursor,
        }),
        Response::Messages(page) => {
            MailOperationalQueryResponseV1::Messages(MailOperationalPageV1 {
                items: page
                    .item
                    .into_iter()
                    .map(message_from_wire)
                    .collect::<Result<_, _>>()?,
                next_cursor: page.next_cursor,
            })
        }
        Response::Message(message) => {
            MailOperationalQueryResponseV1::Message(Box::new(MailMessageDetailV1 {
                summary: message_from_wire(
                    message
                        .summary
                        .ok_or(MailClientWireErrorV1::InvalidPayload)?,
                )?,
            }))
        }
    };
    validate_operational_response(&response).map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    if encode_operational_query_response(&response)? != bytes {
        return Err(MailClientWireErrorV1::InvalidPayload);
    }
    Ok(response)
}

fn folder_to_wire(folder: &MailFolderV1) -> wire::MailFolderV1 {
    wire::MailFolderV1 {
        connection_id: folder.connection_id.clone(),
        folder_id: folder.folder_id.clone(),
        display_name: folder.display_name.clone(),
        kind: folder_kind_to_wire(folder.kind) as i32,
        total_messages: folder.total_messages,
        unread_messages: folder.unread_messages,
        projection_revision: folder.projection_revision,
    }
}

fn folder_from_wire(folder: wire::MailFolderV1) -> Result<MailFolderV1, MailClientWireErrorV1> {
    Ok(MailFolderV1 {
        connection_id: folder.connection_id,
        folder_id: folder.folder_id,
        display_name: folder.display_name,
        kind: folder_kind_from_wire(folder.kind)?,
        total_messages: folder.total_messages,
        unread_messages: folder.unread_messages,
        projection_revision: folder.projection_revision,
    })
}

fn thread_to_wire(thread: &MailThreadV1) -> wire::MailThreadV1 {
    wire::MailThreadV1 {
        connection_id: thread.connection_id.clone(),
        provider_thread_id: thread.provider_thread_id.clone(),
        subject: thread.subject.clone(),
        latest_snippet: thread.latest_snippet.clone(),
        latest_at_unix_seconds: thread.latest_at_unix_seconds,
        message_count: thread.message_count,
        unread_count: thread.unread_count,
        projection_revision: thread.projection_revision,
    }
}

fn thread_from_wire(thread: wire::MailThreadV1) -> MailThreadV1 {
    MailThreadV1 {
        connection_id: thread.connection_id,
        provider_thread_id: thread.provider_thread_id,
        subject: thread.subject,
        latest_snippet: thread.latest_snippet,
        latest_at_unix_seconds: thread.latest_at_unix_seconds,
        message_count: thread.message_count,
        unread_count: thread.unread_count,
        projection_revision: thread.projection_revision,
    }
}

fn message_to_wire(message: &MailMessageSummaryV1) -> wire::MailMessageSummaryV1 {
    wire::MailMessageSummaryV1 {
        connection_id: message.connection_id.clone(),
        provider_message_id: message.provider_message_id.clone(),
        provider_thread_id: message.provider_thread_id.clone(),
        folder_id: message.folder_ids.clone(),
        subject: message.subject.clone(),
        sender: message.sender.clone(),
        recipient: message.recipients.clone(),
        snippet: message.snippet.clone(),
        sent_at_unix_seconds: message.sent_at_unix_seconds,
        flag: message
            .flags
            .iter()
            .map(|flag| message_flag_to_wire(*flag) as i32)
            .collect(),
        has_plain_text: message.has_plain_text,
        has_attachments: message.has_attachments,
        observation_anchor_id: message.observation_anchor_id.to_vec(),
        projection_revision: message.projection_revision,
    }
}

fn message_from_wire(
    message: wire::MailMessageSummaryV1,
) -> Result<MailMessageSummaryV1, MailClientWireErrorV1> {
    let observation_anchor_id = message
        .observation_anchor_id
        .as_slice()
        .try_into()
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    Ok(MailMessageSummaryV1 {
        connection_id: message.connection_id,
        provider_message_id: message.provider_message_id,
        provider_thread_id: message.provider_thread_id,
        folder_ids: message.folder_id,
        subject: message.subject,
        sender: message.sender,
        recipients: message.recipient,
        snippet: message.snippet,
        sent_at_unix_seconds: message.sent_at_unix_seconds,
        flags: message
            .flag
            .into_iter()
            .map(message_flag_from_wire)
            .collect::<Result<_, _>>()?,
        has_plain_text: message.has_plain_text,
        has_attachments: message.has_attachments,
        observation_anchor_id,
        projection_revision: message.projection_revision,
    })
}

const fn folder_kind_to_wire(kind: MailFolderKindV1) -> wire::MailFolderKindV1 {
    match kind {
        MailFolderKindV1::Inbox => wire::MailFolderKindV1::MailFolderKindInbox,
        MailFolderKindV1::Sent => wire::MailFolderKindV1::MailFolderKindSent,
        MailFolderKindV1::Drafts => wire::MailFolderKindV1::MailFolderKindDrafts,
        MailFolderKindV1::Trash => wire::MailFolderKindV1::MailFolderKindTrash,
        MailFolderKindV1::Spam => wire::MailFolderKindV1::MailFolderKindSpam,
        MailFolderKindV1::Archive => wire::MailFolderKindV1::MailFolderKindArchive,
        MailFolderKindV1::ProviderLabel => wire::MailFolderKindV1::MailFolderKindProviderLabel,
    }
}

fn folder_kind_from_wire(kind: i32) -> Result<MailFolderKindV1, MailClientWireErrorV1> {
    match wire::MailFolderKindV1::try_from(kind)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?
    {
        wire::MailFolderKindV1::MailFolderKindInbox => Ok(MailFolderKindV1::Inbox),
        wire::MailFolderKindV1::MailFolderKindSent => Ok(MailFolderKindV1::Sent),
        wire::MailFolderKindV1::MailFolderKindDrafts => Ok(MailFolderKindV1::Drafts),
        wire::MailFolderKindV1::MailFolderKindTrash => Ok(MailFolderKindV1::Trash),
        wire::MailFolderKindV1::MailFolderKindSpam => Ok(MailFolderKindV1::Spam),
        wire::MailFolderKindV1::MailFolderKindArchive => Ok(MailFolderKindV1::Archive),
        wire::MailFolderKindV1::MailFolderKindProviderLabel => Ok(MailFolderKindV1::ProviderLabel),
        wire::MailFolderKindV1::MailFolderKindUnspecified => {
            Err(MailClientWireErrorV1::InvalidPayload)
        }
    }
}

const fn message_flag_to_wire(flag: MailMessageFlagV1) -> wire::MailMessageFlagV1 {
    match flag {
        MailMessageFlagV1::Read => wire::MailMessageFlagV1::MailMessageFlagRead,
        MailMessageFlagV1::Starred => wire::MailMessageFlagV1::MailMessageFlagStarred,
        MailMessageFlagV1::Draft => wire::MailMessageFlagV1::MailMessageFlagDraft,
        MailMessageFlagV1::Sent => wire::MailMessageFlagV1::MailMessageFlagSent,
        MailMessageFlagV1::Trashed => wire::MailMessageFlagV1::MailMessageFlagTrashed,
        MailMessageFlagV1::Spam => wire::MailMessageFlagV1::MailMessageFlagSpam,
    }
}

fn message_flag_from_wire(flag: i32) -> Result<MailMessageFlagV1, MailClientWireErrorV1> {
    match wire::MailMessageFlagV1::try_from(flag)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?
    {
        wire::MailMessageFlagV1::MailMessageFlagRead => Ok(MailMessageFlagV1::Read),
        wire::MailMessageFlagV1::MailMessageFlagStarred => Ok(MailMessageFlagV1::Starred),
        wire::MailMessageFlagV1::MailMessageFlagDraft => Ok(MailMessageFlagV1::Draft),
        wire::MailMessageFlagV1::MailMessageFlagSent => Ok(MailMessageFlagV1::Sent),
        wire::MailMessageFlagV1::MailMessageFlagTrashed => Ok(MailMessageFlagV1::Trashed),
        wire::MailMessageFlagV1::MailMessageFlagSpam => Ok(MailMessageFlagV1::Spam),
        wire::MailMessageFlagV1::MailMessageFlagUnspecified => {
            Err(MailClientWireErrorV1::InvalidPayload)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn message() -> MailMessageSummaryV1 {
        MailMessageSummaryV1 {
            connection_id: "mail-account".to_owned(),
            provider_message_id: "message-1".to_owned(),
            provider_thread_id: "thread-1".to_owned(),
            folder_ids: vec!["INBOX".to_owned()],
            subject: Some("Subject".to_owned()),
            sender: Some("sender@example.test".to_owned()),
            recipients: vec!["owner@example.test".to_owned()],
            snippet: Some("Preview".to_owned()),
            sent_at_unix_seconds: Some(1),
            flags: vec![MailMessageFlagV1::Read],
            has_plain_text: true,
            has_attachments: false,
            observation_anchor_id: [7; 16],
            projection_revision: 1,
        }
    }

    #[test]
    fn query_and_all_response_variants_round_trip_canonically() {
        let query = MailOperationalQueryV1::ListMessages {
            connection_id: "mail-account".to_owned(),
            folder_id: Some("INBOX".to_owned()),
            provider_thread_id: None,
            cursor: None,
            limit: 100,
        };
        let encoded_query = encode_operational_query(&query).expect("encode query");
        assert_eq!(decode_operational_query(&encoded_query), Ok(query));

        let responses = [
            MailOperationalQueryResponseV1::Folders(MailOperationalPageV1 {
                items: vec![MailFolderV1 {
                    connection_id: "mail-account".to_owned(),
                    folder_id: "INBOX".to_owned(),
                    display_name: "Inbox".to_owned(),
                    kind: MailFolderKindV1::Inbox,
                    total_messages: 1,
                    unread_messages: 0,
                    projection_revision: 1,
                }],
                next_cursor: None,
            }),
            MailOperationalQueryResponseV1::Threads(MailOperationalPageV1 {
                items: vec![MailThreadV1 {
                    connection_id: "mail-account".to_owned(),
                    provider_thread_id: "thread-1".to_owned(),
                    subject: Some("Subject".to_owned()),
                    latest_snippet: Some("Preview".to_owned()),
                    latest_at_unix_seconds: Some(1),
                    message_count: 1,
                    unread_count: 0,
                    projection_revision: 1,
                }],
                next_cursor: None,
            }),
            MailOperationalQueryResponseV1::Messages(MailOperationalPageV1 {
                items: vec![message()],
                next_cursor: None,
            }),
            MailOperationalQueryResponseV1::Message(Box::new(MailMessageDetailV1 {
                summary: message(),
            })),
        ];
        for response in responses {
            let encoded = encode_operational_query_response(&response).expect("encode response");
            assert_eq!(decode_operational_query_response(&encoded), Ok(response));
        }
    }

    #[test]
    fn rejects_unknown_enum_and_noncanonical_payload() {
        let invalid = wire::MailOperationalQueryResponseV1 {
            response: Some(wire::mail_operational_query_response_v1::Response::Folders(
                wire::MailFolderPageV1 {
                    item: vec![wire::MailFolderV1 {
                        connection_id: "mail-account".to_owned(),
                        folder_id: "INBOX".to_owned(),
                        display_name: "Inbox".to_owned(),
                        kind: 99,
                        total_messages: 1,
                        unread_messages: 0,
                        projection_revision: 1,
                    }],
                    next_cursor: None,
                },
            )),
        }
        .encode_to_vec();
        assert_eq!(
            decode_operational_query_response(&invalid),
            Err(MailClientWireErrorV1::InvalidPayload)
        );

        let mut noncanonical = encode_operational_query(&MailOperationalQueryV1::GetMessage {
            connection_id: "mail-account".to_owned(),
            provider_message_id: "message-1".to_owned(),
        })
        .expect("query");
        noncanonical.extend_from_slice(&[0x98, 0x06, 0x01]);
        assert_eq!(
            decode_operational_query(&noncanonical),
            Err(MailClientWireErrorV1::InvalidPayload)
        );
    }
}
