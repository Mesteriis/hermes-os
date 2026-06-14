use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EventAnalysis {
    pub event_type: String,
    pub importance_score: f64,
    pub readiness_score: f64,
    pub risks: Vec<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EventFingerprint {
    pub event_type: String,
    pub importance: f64,
    pub language: Option<String>,
    pub recurrence_hint: Option<String>,
    pub topics: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LocationInfo {
    pub is_online: bool,
    pub parsed_name: Option<String>,
    pub travel_buffer_minutes: Option<i32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BackToBackGroup {
    pub titles: Vec<String>,
    pub count: usize,
}
