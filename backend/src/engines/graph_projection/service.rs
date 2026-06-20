use sqlx::postgres::PgPool;

use crate::domains::graph::core::GraphStore;
use crate::domains::projects::core::ProjectStore;

use super::errors::GraphProjectionError;
use super::models::GraphProjectionReport;

#[derive(Clone)]
pub struct GraphProjectionService {
    pub(super) pool: PgPool,
    pub(super) graph: GraphStore,
    pub(super) projects: ProjectStore,
}

impl GraphProjectionService {
    pub fn new(pool: PgPool) -> Self {
        Self {
            graph: GraphStore::new(pool.clone()),
            projects: ProjectStore::new(pool.clone()),
            pool,
        }
    }

    pub async fn project_from_v1(&self) -> Result<GraphProjectionReport, GraphProjectionError> {
        let mut report = GraphProjectionReport::default();

        for person in self.list_persons().await? {
            self.project_person(&person, &mut report).await?;
        }
        for message in self.list_messages().await? {
            self.project_message(&message, &mut report).await?;
        }
        for document in self.list_documents().await? {
            self.project_document(&document, &mut report).await?;
        }
        for project in self.projects.graph_projection_projects().await? {
            self.project_project(&project, &mut report).await?;
        }

        Ok(report)
    }
}
