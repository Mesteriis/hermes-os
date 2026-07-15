use chrono::{DateTime, Utc};

#[derive(Clone, Debug, PartialEq)]
pub struct MemoryCardDraft {
    pub title: String,
    pub description: String,
    pub source: String,
    pub confidence: f64,
    pub importance: i16,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MemoryFactDraft {
    pub affected_entity_kind: String,
    pub affected_entity_id: String,
    pub fact_type: String,
    pub value: String,
    pub source: String,
    pub confidence: f64,
    pub review_state: String,
    pub produced_by: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MemoryFactState {
    pub affected_entity_kind: String,
    pub affected_entity_id: String,
    pub fact_type: String,
    pub value: String,
    pub source: String,
    pub confidence: f64,
    pub review_state: String,
    pub last_verified_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MemoryContextPack {
    pub affected_entity_kind: String,
    pub affected_entity_id: String,
    pub items: Vec<MemoryContextItem>,
    pub source_citations: Vec<String>,
    pub confidence: f64,
    pub produced_by: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MemoryContextItem {
    pub item_kind: String,
    pub title: String,
    pub body: String,
    pub source: String,
    pub confidence: f64,
    pub review_state: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MemoryGap {
    pub affected_entity_kind: String,
    pub affected_entity_id: String,
    pub missing_fact_type: String,
    pub source: String,
    pub review_state: String,
    pub produced_by: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MemoryStaleCandidate {
    pub affected_entity_kind: String,
    pub affected_entity_id: String,
    pub fact_type: String,
    pub value: String,
    pub source: String,
    pub confidence: f64,
    pub last_verified_at: Option<DateTime<Utc>>,
    pub review_state: String,
    pub produced_by: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MemoryEntityRef {
    pub entity_kind: String,
    pub entity_id: String,
    pub relation_kind: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MemoryContextSource {
    pub entity_kind: String,
    pub entity_id: String,
    pub item_kind: String,
    pub title: String,
    pub body: String,
    pub source: String,
    pub confidence: f64,
    pub review_state: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CrossDomainMemoryContextPack {
    pub root_entity_kind: String,
    pub root_entity_id: String,
    pub items: Vec<MemoryCrossDomainContextItem>,
    pub entity_citations: Vec<String>,
    pub source_citations: Vec<String>,
    pub confidence: f64,
    pub produced_by: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MemoryCrossDomainContextItem {
    pub entity_kind: String,
    pub entity_id: String,
    pub relation_kind: String,
    pub item_kind: String,
    pub title: String,
    pub body: String,
    pub source: String,
    pub confidence: f64,
    pub review_state: String,
}
