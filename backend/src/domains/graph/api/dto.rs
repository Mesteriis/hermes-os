use serde::Deserialize;

#[derive(Deserialize)]
pub struct NeighborhoodQuery {
    pub node_id: Option<String>,
    pub depth: Option<u8>,
}
#[derive(Deserialize)]
pub struct NodesQuery {
    pub limit: Option<i64>,
}
#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
    pub limit: Option<i64>,
}
