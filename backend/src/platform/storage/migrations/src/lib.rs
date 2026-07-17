//! PostgreSQL migration admission boundary.

mod admission;

pub use admission::{
    MigrationAdmissionErrorV1, MigrationBundleAdmissionErrorV1, admit_owner_local_additive_sql,
    admit_storage_bundle,
};
