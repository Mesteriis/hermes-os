use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::future::Future;
use std::pin::Pin;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GraphCount {
    pub key: String,
    pub count: i64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GraphSummary {
    pub node_counts: Vec<GraphCount>,
    pub edge_counts: Vec<GraphCount>,
    pub evidence_count: i64,
    pub latest_projection_at: Option<DateTime<Utc>>,
    pub is_empty: bool,
}

pub type GraphSummaryFuture<'a> =
    Pin<Box<dyn Future<Output = Result<GraphSummary, GraphQueryError>> + Send + 'a>>;

pub trait GraphSummaryQueryPort: Send + Sync {
    fn summary<'a>(&'a self) -> GraphSummaryFuture<'a>;
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GraphNodeRead {
    pub node_id: String,
    pub node_kind: String,
    pub stable_key: String,
    pub label: String,
    pub properties: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub type GraphNodeListFuture<'a> =
    Pin<Box<dyn Future<Output = Result<Vec<GraphNodeRead>, GraphQueryError>> + Send + 'a>>;

pub trait GraphNodeReadPort: Send + Sync {
    fn list_nodes<'a>(&'a self, limit: i64) -> GraphNodeListFuture<'a>;
}

pub trait GraphNodeSearchPort: Send + Sync {
    fn search_nodes<'a>(&'a self, query: &'a str, limit: i64) -> GraphNodeListFuture<'a>;
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct GraphEdgeRead {
    pub edge_id: String,
    pub source_node_id: String,
    pub target_node_id: String,
    pub relationship_type: String,
    pub confidence: f64,
    pub review_state: String,
    pub properties: Value,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_to: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct GraphEvidenceRead {
    pub edge_id: String,
    pub source_kind: String,
    pub source_id: String,
    pub observation_id: Option<String>,
    pub excerpt: Option<String>,
    pub metadata: Value,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct GraphNeighborhoodRead {
    pub selected_node: GraphNodeRead,
    pub nodes: Vec<GraphNodeRead>,
    pub edges: Vec<GraphEdgeRead>,
    pub evidence: Vec<GraphEvidenceRead>,
    pub edge_limit: i64,
    pub truncated: bool,
    pub evidence_limit: i64,
    pub evidence_truncated: bool,
}

pub type GraphNeighborhoodFuture<'a> = Pin<
    Box<dyn Future<Output = Result<Option<GraphNeighborhoodRead>, GraphQueryError>> + Send + 'a>,
>;

pub trait GraphNeighborhoodQueryPort: Send + Sync {
    fn neighborhood<'a>(&'a self, node_id: &'a str) -> GraphNeighborhoodFuture<'a>;
}

#[derive(Debug, thiserror::Error)]
#[error("graph query failed: {0}")]
pub struct GraphQueryError(pub String);
