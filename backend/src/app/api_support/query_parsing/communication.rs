use url::form_urlencoded;

use crate::app::ApiError;
use crate::domains::communications::messages::{
    MessageSearchMatchMode, MessageSearchQuery, parse_communication_message_search_query,
};

#[derive(Debug)]
pub(crate) struct CommunicationMessagesQuery {
    pub(crate) account_id: Option<String>,
    pub(crate) workflow_state: Option<String>,
    pub(crate) channel_kind: Option<String>,
    pub(crate) conversation_id: Option<String>,
    pub(crate) q: Option<String>,
    pub(crate) match_mode: MessageSearchMatchMode,
    pub(crate) search: MessageSearchQuery,
    pub(crate) local_state: Option<String>,
    pub(crate) cursor: Option<String>,
    pub(crate) limit: Option<i64>,
}

pub(crate) fn parse_communication_messages_query(
    raw_query: Option<&str>,
) -> Result<CommunicationMessagesQuery, ApiError> {
    let mut query = CommunicationMessagesQuery {
        account_id: None,
        workflow_state: None,
        channel_kind: None,
        conversation_id: None,
        q: None,
        local_state: None,
        cursor: None,
        limit: None,
        match_mode: MessageSearchMatchMode::All,
        search: MessageSearchQuery::default(),
    };

    if let Some(raw_query) = raw_query {
        for (key, value) in form_urlencoded::parse(raw_query.as_bytes()) {
            match key.as_ref() {
                "account_id" => query.account_id = non_empty_query_value(value.as_ref()),
                "workflow_state" => query.workflow_state = non_empty_query_value(value.as_ref()),
                "channel_kind" => query.channel_kind = non_empty_query_value(value.as_ref()),
                "conversation_id" => query.conversation_id = non_empty_query_value(value.as_ref()),
                "q" => query.q = non_empty_query_value(value.as_ref()),
                "local_state" => query.local_state = non_empty_query_value(value.as_ref()),
                "cursor" => query.cursor = non_empty_query_value(value.as_ref()),
                "limit" => {
                    query.limit = Some(value.parse::<i64>().map_err(|_| {
                        ApiError::InvalidCommunicationQuery("limit must be an integer")
                    })?);
                }
                _ => {}
            }
        }
    }

    if let Some(raw_query) = query.q.as_deref() {
        let parsed = parse_communication_message_search_query(Some(raw_query));
        query.match_mode = parsed.match_mode;
        query.search = parsed;
    }

    Ok(query)
}

fn non_empty_query_value(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_owned())
    }
}
