//! Canonical PostgreSQL schema boundary.
//!
//! The migration set is temporarily sourced from the legacy checkout while
//! owners are moved into their API/adapter crates. Consumers must call this
//! crate instead of embedding a migration path in their own package. The
//! migration directory moves here in the canonical-schema reset slice.

use sqlx::migrate::{MigrateError, Migrator};

pub static MIGRATOR: Migrator = sqlx::migrate!("../../backend/migrations");

pub async fn apply(pool: &sqlx::PgPool) -> Result<(), MigrateError> {
    MIGRATOR.run(pool).await
}

#[cfg(test)]
mod tests {
    use super::MIGRATOR;

    #[test]
    fn canonical_bundle_is_non_empty_and_ordered() {
        let migrations = MIGRATOR.iter().collect::<Vec<_>>();
        assert!(
            !migrations.is_empty(),
            "schema bundle must contain migrations"
        );
        assert!(
            migrations
                .windows(2)
                .all(|pair| pair[0].version < pair[1].version),
            "schema migrations must be strictly ordered"
        );
    }
}
