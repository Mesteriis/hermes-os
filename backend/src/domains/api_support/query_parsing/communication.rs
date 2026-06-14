use url::form_urlencoded;

use crate::app::ApiError;

pub(crate) struct CommunicationMessagesQuery {
    pub(crate) account_id: Option<String>,
    pub(crate) workflow_state: Option<String>,
    pub(crate) channel_kind: Option<String>,
    pub(crate) q: Option<String>,
    pub(crate) local_state: Option<String>,
    pub(crate) limit: Option<i64>,
}

pub(crate) fn parse_communication_messages_query(
    raw_query: Option<&str>,
) -> Result<CommunicationMessagesQuery, ApiError> {
    let mut query = CommunicationMessagesQuery {
        account_id: None,
        workflow_state: None,
        channel_kind: None,
        q: None,
        local_state: None,
        limit: None,
    };

    if let Some(raw_query) = raw_query {
        for (key, value) in form_urlencoded::parse(raw_query.as_bytes()) {
            match key.as_ref() {
                "account_id" => query.account_id = non_empty_query_value(value.as_ref()),
                "workflow_state" => query.workflow_state = non_empty_query_value(value.as_ref()),
                "channel_kind" => query.channel_kind = non_empty_query_value(value.as_ref()),
                "q" => query.q = non_empty_query_value(value.as_ref()),
                "local_state" => query.local_state = non_empty_query_value(value.as_ref()),
                "limit" => {
                    query.limit = Some(value.parse::<i64>().map_err(|_| {
                        ApiError::InvalidCommunicationQuery("limit must be an integer")
                    })?);
                }
                _ => {}
            }
        }
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
