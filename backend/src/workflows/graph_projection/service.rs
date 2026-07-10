use sqlx::postgres::PgPool;

use crate::domains::graph::core::GraphProjectionPort;
use crate::domains::projects::core::ProjectCommandPort;

use super::errors::GraphProjectionError;
use super::models::GraphProjectionReport;

#[derive(Clone)]
pub struct GraphProjectionService {
    pub(super) pool: PgPool,
    pub(super) graph: GraphProjectionPort,
    pub(super) projects: ProjectCommandPort,
}

impl GraphProjectionService {
    pub fn new(pool: PgPool) -> Self {
        Self {
            graph: GraphProjectionPort::new(pool.clone()),
            projects: ProjectCommandPort::new(pool.clone()),
            pool,
        }
    }

    pub async fn project_from_v1(&self) -> Result<GraphProjectionReport, GraphProjectionError> {
        let mut report = GraphProjectionReport::default();

        for persona in self.list_personas().await? {
            self.project_persona(&persona, &mut report).await?;
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
        for decision in self.list_decisions().await? {
            self.project_decision(&decision, &mut report).await?;
        }
        for obligation in self.list_obligations().await? {
            self.project_obligation(&obligation, &mut report).await?;
        }

        Ok(report)
    }
}
