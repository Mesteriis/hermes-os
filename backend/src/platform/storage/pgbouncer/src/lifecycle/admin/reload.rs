//! Exact PgBouncer configuration reload command.

use crate::PoolLifecycleOutcomeV1;

use super::PgBouncerAdminPortV1;

pub async fn reload_configuration(
    admin: &mut impl PgBouncerAdminPortV1,
) -> Result<(), PoolLifecycleOutcomeV1> {
    match admin.execute_pool_command("RELOAD").await {
        PoolLifecycleOutcomeV1::Applied => Ok(()),
        outcome => Err(outcome),
    }
}
