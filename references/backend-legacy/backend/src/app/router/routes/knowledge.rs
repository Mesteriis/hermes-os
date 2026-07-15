use crate::app::handlers::documents::*;
use crate::app::handlers::graph::*;
use crate::app::handlers::projects::*;
use crate::app::state::AppState;
use axum::Router;
use axum::routing::{get, post, put};

pub(super) fn add_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route("/api/v1/graph/summary", get(get_graph_summary))
        .route("/api/v1/graph/nodes", get(get_graph_nodes))
        .route("/api/v1/graph/neighborhood", get(get_graph_neighborhood))
        .route("/api/v1/graph/search", get(get_graph_search))
        .route("/api/v1/projects", get(get_projects))
        .route("/api/v1/projects/{project_id}", get(get_project_detail))
        .route(
            "/api/v1/projects/{project_id}/link-candidates",
            get(get_project_link_candidates),
        )
        .route(
            "/api/v1/projects/{project_id}/link-reviews",
            put(put_project_link_review),
        )
        .route(
            "/api/v1/documents/{document_id}/processing",
            get(get_document_processing),
        )
        .route(
            "/api/v1/document-processing/jobs",
            get(get_document_processing_jobs),
        )
        .route(
            "/api/v1/document-processing/jobs/{job_id}/retry",
            post(post_document_processing_job_retry),
        )
}
