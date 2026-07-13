use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GmailListResponse {
    pub messages: Option<Vec<GmailListedMessage>>,
    pub next_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GmailListedMessage {
    pub id: String,
    pub thread_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GmailRawMessage {
    pub id: Option<String>,
    pub thread_id: Option<String>,
    pub label_ids: Option<Vec<String>>,
    pub history_id: Option<String>,
    pub internal_date: Option<String>,
    pub raw: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GmailLabelsResponse {
    pub labels: Option<Vec<GmailLabel>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GmailLabel {
    pub id: Option<String>,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub label_type: Option<String>,
    pub message_list_visibility: Option<String>,
    pub label_list_visibility: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GmailSendResponse {
    pub id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GmailHistoryResponse {
    pub history: Option<Vec<GmailHistoryItem>>,
    pub history_id: Option<String>,
    pub next_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GmailHistoryItem {
    pub messages_added: Option<Vec<GmailHistoryMessageAdded>>,
    pub labels_added: Option<Vec<GmailHistoryMessageAdded>>,
    pub labels_removed: Option<Vec<GmailHistoryMessageAdded>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GmailHistoryMessageAdded {
    pub message: GmailHistoryMessage,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GmailHistoryMessage {
    pub id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GooglePeopleConnectionsResponse {
    pub connections: Option<Vec<GooglePeoplePerson>>,
    pub next_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GooglePeoplePerson {
    pub resource_name: Option<String>,
    pub etag: Option<String>,
    pub names: Option<Vec<GooglePeopleName>>,
    pub email_addresses: Option<Vec<GooglePeopleEmailAddress>>,
    pub phone_numbers: Option<Vec<GooglePeoplePhoneNumber>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GooglePeopleName {
    pub display_name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GooglePeopleEmailAddress {
    pub value: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GooglePeoplePhoneNumber {
    pub value: Option<String>,
}
