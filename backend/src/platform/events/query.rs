use chrono::{DateTime, Utc};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EventLogQuery {
    pub event_type: Option<String>,
    pub source_code: Option<String>,
    pub subject_kind: Option<String>,
    pub subject_entity_id: Option<String>,
    pub correlation_id: Option<String>,
    pub position_after: Option<i64>,
    pub position_before: Option<i64>,
    pub occurred_after: Option<DateTime<Utc>>,
    pub occurred_before: Option<DateTime<Utc>>,
    pub limit: Option<u32>,
}

impl EventLogQuery {
    pub fn event_type(mut self, event_type: impl Into<String>) -> Self {
        self.event_type = some_non_empty(event_type);
        self
    }

    pub fn source_code(mut self, source_code: impl Into<String>) -> Self {
        self.source_code = some_non_empty(source_code);
        self
    }

    pub fn subject_kind(mut self, subject_kind: impl Into<String>) -> Self {
        self.subject_kind = some_non_empty(subject_kind);
        self
    }

    pub fn subject_entity_id(mut self, subject_entity_id: impl Into<String>) -> Self {
        self.subject_entity_id = some_non_empty(subject_entity_id);
        self
    }

    pub fn correlation_id(mut self, correlation_id: impl Into<String>) -> Self {
        self.correlation_id = some_non_empty(correlation_id);
        self
    }

    pub fn position_between(mut self, position_after: i64, position_before: i64) -> Self {
        self.position_after = Some(position_after);
        self.position_before = Some(position_before);
        self
    }

    pub fn position_after(mut self, position_after: i64) -> Self {
        self.position_after = Some(position_after);
        self
    }

    pub fn position_before(mut self, position_before: i64) -> Self {
        self.position_before = Some(position_before);
        self
    }

    pub fn occurred_between(
        mut self,
        occurred_after: DateTime<Utc>,
        occurred_before: DateTime<Utc>,
    ) -> Self {
        self.occurred_after = Some(occurred_after);
        self.occurred_before = Some(occurred_before);
        self
    }

    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }
}

fn some_non_empty(value: impl Into<String>) -> Option<String> {
    let value = value.into();
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_owned())
}
