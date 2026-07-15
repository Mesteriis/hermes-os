use hermes_projects_api::ProjectReadPort;
use hermes_projects_postgres::ProjectPostgresReadQuery;
use sqlx::PgPool;

pub fn project_read_port(pool: PgPool) -> impl ProjectReadPort {
    ProjectPostgresReadQuery::new(pool)
}
