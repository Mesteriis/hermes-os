//! One bounded authentication check for a Storage-owned PgBouncer admin port.

use std::time::Duration;

use crate::PoolLifecycleOutcomeV1;

use super::{
    PgBouncerAdminConnectionErrorV1, PgBouncerAdminCredentialV1, PgBouncerAdminEndpointV1,
    PgBouncerAdminPortV1, TokioPostgresPgBouncerAdminPortV1,
};

const ADMIN_COMMAND_TIMEOUT: Duration = Duration::from_secs(2);

pub async fn verify_admin_connection(
    endpoint: &PgBouncerAdminEndpointV1,
    credential: &PgBouncerAdminCredentialV1,
) -> Result<(), PgBouncerAdminConnectionErrorV1> {
    let mut port = TokioPostgresPgBouncerAdminPortV1::connect(endpoint, credential).await?;
    match tokio::time::timeout(
        ADMIN_COMMAND_TIMEOUT,
        port.execute_pool_command("SHOW VERSION"),
    )
    .await
    {
        Ok(PoolLifecycleOutcomeV1::Applied) => Ok(()),
        Ok(PoolLifecycleOutcomeV1::Rejected | PoolLifecycleOutcomeV1::Unavailable) | Err(_) => {
            Err(PgBouncerAdminConnectionErrorV1::Unavailable)
        }
    }
}
