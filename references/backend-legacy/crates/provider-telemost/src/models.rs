use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

fn default_json_object() -> Value {
    json!({})
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TelemostCohost {
    pub email: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TelemostLiveStreamRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_level: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TelemostLiveStreamResponse {
    #[serde(default)]
    pub watch_url: Option<String>,
    #[serde(default)]
    pub access_level: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct YandexTelemostConferenceRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub waiting_room_level: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub live_stream: Option<TelemostLiveStreamRequest>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cohosts: Vec<TelemostCohost>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_auto_summarization_enabled: Option<bool>,
    #[serde(default = "default_json_object", skip_serializing)]
    pub metadata: Value,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct YandexTelemostConferencePatchRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub waiting_room_level: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub live_stream: Option<TelemostLiveStreamRequest>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cohosts: Vec<TelemostCohost>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_auto_summarization_enabled: Option<bool>,
    #[serde(default = "default_json_object", skip_serializing)]
    pub metadata: Value,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct YandexTelemostConference {
    pub id: String,
    pub join_url: String,
    #[serde(default)]
    pub access_level: Option<String>,
    #[serde(default)]
    pub waiting_room_level: Option<String>,
    #[serde(default)]
    pub live_stream: Option<TelemostLiveStreamResponse>,
    #[serde(default)]
    pub sip_uri_meeting: Option<String>,
    #[serde(default)]
    pub sip_uri_telemost: Option<String>,
    #[serde(default)]
    pub sip_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct YandexTelemostCreateConferenceCommand {
    pub account_id: String,
    pub body: YandexTelemostConferenceRequest,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct YandexTelemostCohostPage {
    #[serde(default)]
    pub cohosts: Vec<TelemostCohost>,
}
