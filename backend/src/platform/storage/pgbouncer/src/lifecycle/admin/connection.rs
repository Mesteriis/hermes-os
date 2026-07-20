//! Executes an already-fenced PgBouncer command through its simple-query console.

use std::future::Future;
use std::time::Duration;

use tokio_postgres::{Client, Config, NoTls};

use crate::PoolLifecycleOutcomeV1;

use super::{
    PgBouncerAdminConnectionErrorV1, PgBouncerAdminCredentialV1, PgBouncerAdminEndpointV1,
    PgBouncerAdminPortV1,
};

const ADMIN_CONNECTION_TIMEOUT: Duration = Duration::from_secs(2);

pub struct TokioPostgresPgBouncerAdminPortV1 {
    pub(super) client: Client,
}

impl TokioPostgresPgBouncerAdminPortV1 {
    pub async fn connect(
        endpoint: &PgBouncerAdminEndpointV1,
        credential: &PgBouncerAdminCredentialV1,
    ) -> Result<Self, PgBouncerAdminConnectionErrorV1> {
        let options = connection_options(endpoint, credential);
        let connection = options.connect(NoTls);
        let (client, connection) = tokio::time::timeout(ADMIN_CONNECTION_TIMEOUT, connection)
            .await
            .map_err(|_| PgBouncerAdminConnectionErrorV1::Unavailable)?
            .map_err(|_| PgBouncerAdminConnectionErrorV1::Unavailable)?;
        tokio::spawn(async move {
            let _ = connection.await;
        });
        Ok(Self { client })
    }
}

impl PgBouncerAdminPortV1 for TokioPostgresPgBouncerAdminPortV1 {
    fn execute_pool_command(
        &mut self,
        command: &str,
    ) -> impl Future<Output = PoolLifecycleOutcomeV1> + Send {
        let command = command.to_owned();
        async move {
            match self.client.simple_query(&command).await {
                Ok(_) => PoolLifecycleOutcomeV1::Applied,
                Err(error) if error.as_db_error().is_some() => PoolLifecycleOutcomeV1::Rejected,
                Err(_) => PoolLifecycleOutcomeV1::Unavailable,
            }
        }
    }
}

fn connection_options(
    endpoint: &PgBouncerAdminEndpointV1,
    credential: &PgBouncerAdminCredentialV1,
) -> Config {
    let mut options = Config::new();
    options
        .host(endpoint.host())
        .port(endpoint.port())
        .dbname("pgbouncer")
        .user(credential.username())
        .password(credential.password());
    options
}
