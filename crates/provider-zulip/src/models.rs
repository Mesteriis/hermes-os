use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ZulipRegisterQueueResponse {
    pub result: String,
    pub msg: String,
    pub queue_id: String,
    pub last_event_id: i64,
    #[serde(default)]
    pub event_queue_longpoll_timeout_seconds: Option<u64>,
    #[serde(default)]
    pub idle_queue_timeout_secs: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ZulipEventsResponse {
    pub result: String,
    pub msg: String,
    #[serde(default)]
    pub queue_id: Option<String>,
    #[serde(default)]
    pub events: Vec<ZulipEvent>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ZulipEvent {
    pub id: i64,
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(flatten)]
    pub data: Map<String, Value>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ZulipSendMessageResponse {
    pub result: String,
    pub msg: String,
    #[serde(default)]
    pub id: Option<i64>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ZulipBasicResponse {
    pub result: String,
    pub msg: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ZulipUploadFileResponse {
    pub result: String,
    pub msg: String,
    pub uri: String,
}
