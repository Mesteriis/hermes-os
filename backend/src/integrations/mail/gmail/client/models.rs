use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct GmailListResponse {
    pub(super) messages: Option<Vec<GmailListedMessage>>,
    pub(super) next_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct GmailListedMessage {
    pub(super) id: String,
    pub(super) thread_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct GmailRawMessage {
    pub(super) id: Option<String>,
    pub(super) thread_id: Option<String>,
    pub(super) label_ids: Option<Vec<String>>,
    pub(super) history_id: Option<String>,
    pub(super) internal_date: Option<String>,
    pub(super) raw: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(super) struct GmailSendResponse {
    pub(super) id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct GmailHistoryResponse {
    pub(super) history: Option<Vec<GmailHistoryItem>>,
    pub(super) history_id: Option<String>,
    pub(super) next_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct GmailHistoryItem {
    pub(super) messages_added: Option<Vec<GmailHistoryMessageAdded>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct GmailHistoryMessageAdded {
    pub(super) message: GmailHistoryMessage,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct GmailHistoryMessage {
    pub(super) id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct GooglePeopleConnectionsResponse {
    pub(super) connections: Option<Vec<GooglePeoplePerson>>,
    pub(super) next_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct GooglePeoplePerson {
    pub(super) resource_name: Option<String>,
    pub(super) etag: Option<String>,
    pub(super) names: Option<Vec<GooglePeopleName>>,
    pub(super) email_addresses: Option<Vec<GooglePeopleEmailAddress>>,
    pub(super) phone_numbers: Option<Vec<GooglePeoplePhoneNumber>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct GooglePeopleName {
    pub(super) display_name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct GooglePeopleEmailAddress {
    pub(super) value: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct GooglePeoplePhoneNumber {
    pub(super) value: Option<String>,
}
