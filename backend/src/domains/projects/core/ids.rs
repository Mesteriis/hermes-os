use crate::domains::graph::core::{GraphNodeKind, node_id};

pub fn project_graph_node_id(project_id: &str) -> String {
    node_id(GraphNodeKind::Project, project_id)
}
