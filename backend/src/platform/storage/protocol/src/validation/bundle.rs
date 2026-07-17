//! Structural validation for immutable owner-local migration bundles.

use crate::v1::StorageBundleV1;
use sha2::{Digest, Sha256};

const MAX_STEPS: usize = 256;
const MAX_SQL_BYTES: usize = 1_048_576;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorageBundleValidationErrorV1 {
    Version,
    Identifier,
    Steps,
    StepOrder,
    StepDigest,
    StepDigestMismatch,
    Sql,
}

pub fn validate_storage_bundle(
    bundle: &StorageBundleV1,
) -> Result<(), StorageBundleValidationErrorV1> {
    if bundle.major != 1 || bundle.revision == 0 {
        return Err(StorageBundleValidationErrorV1::Version);
    }
    if !valid_identifier(&bundle.bundle_id) || !valid_identifier(&bundle.owner_id) {
        return Err(StorageBundleValidationErrorV1::Identifier);
    }
    if bundle.steps.is_empty() || bundle.steps.len() > MAX_STEPS {
        return Err(StorageBundleValidationErrorV1::Steps);
    }
    let mut previous_revision = 0;
    for step in &bundle.steps {
        if step.revision <= previous_revision {
            return Err(StorageBundleValidationErrorV1::StepOrder);
        }
        if !valid_identifier(&step.migration_id) {
            return Err(StorageBundleValidationErrorV1::Identifier);
        }
        if step.forward_sql_utf8.is_empty()
            || step.forward_sql_utf8.len() > MAX_SQL_BYTES
            || std::str::from_utf8(&step.forward_sql_utf8).is_err()
        {
            return Err(StorageBundleValidationErrorV1::Sql);
        }
        if step.sha256.len() != 32 {
            return Err(StorageBundleValidationErrorV1::StepDigest);
        }
        if Sha256::digest(&step.forward_sql_utf8).as_slice() != step.sha256.as_slice() {
            return Err(StorageBundleValidationErrorV1::StepDigestMismatch);
        }
        previous_revision = step.revision;
    }
    Ok(())
}

fn valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 96
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}
