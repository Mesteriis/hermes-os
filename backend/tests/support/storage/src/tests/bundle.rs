use hermes_storage_migrations::{
    MigrationAdmissionErrorV1, MigrationBundleAdmissionErrorV1, admit_storage_bundle,
};
use hermes_storage_protocol::{
    v1::{StorageBundleV1, StorageMigrationStepV1},
    validation::{StorageBundleValidationErrorV1, validate_storage_bundle},
};
use sha2::{Digest, Sha256};

fn valid_bundle() -> StorageBundleV1 {
    StorageBundleV1 {
        major: 1,
        revision: 1,
        bundle_id: "notes_schema".into(),
        owner_id: "notes".into(),
        steps: vec![StorageMigrationStepV1 {
            revision: 1,
            migration_id: "create_entries".into(),
            forward_sql_utf8: b"CREATE TABLE hermes_data.notes_entries (entry_id uuid);".to_vec(),
            sha256: Sha256::digest(b"CREATE TABLE hermes_data.notes_entries (entry_id uuid);")
                .to_vec(),
        }],
    }
}

#[test]
fn accepts_a_bounded_ordered_bundle() {
    assert_eq!(validate_storage_bundle(&valid_bundle()), Ok(()));
    assert_eq!(admit_storage_bundle(&valid_bundle()), Ok(()));
}

#[test]
fn rejects_unversioned_or_reordered_steps() {
    let mut unversioned = valid_bundle();
    unversioned.major = 0;
    let mut reordered = valid_bundle();
    reordered.steps.push(StorageMigrationStepV1 {
        revision: 1,
        migration_id: "duplicate_revision".into(),
        forward_sql_utf8: b"ALTER TABLE hermes_data.notes_entries ADD COLUMN title text;".to_vec(),
        sha256: Sha256::digest(b"ALTER TABLE hermes_data.notes_entries ADD COLUMN title text;")
            .to_vec(),
    });

    assert_eq!(
        validate_storage_bundle(&unversioned),
        Err(StorageBundleValidationErrorV1::Version)
    );
    assert_eq!(
        validate_storage_bundle(&reordered),
        Err(StorageBundleValidationErrorV1::StepOrder)
    );
}

#[test]
fn rejects_malformed_step_content() {
    let mut wrong_digest = valid_bundle();
    wrong_digest.steps[0].sha256.clear();
    let mut invalid_utf8 = valid_bundle();
    invalid_utf8.steps[0].forward_sql_utf8 = vec![0xff];

    assert_eq!(
        validate_storage_bundle(&wrong_digest),
        Err(StorageBundleValidationErrorV1::StepDigest)
    );
    assert_eq!(
        validate_storage_bundle(&invalid_utf8),
        Err(StorageBundleValidationErrorV1::Sql)
    );
}

#[test]
fn rejects_a_digest_that_does_not_match_step_bytes() {
    let mut bundle = valid_bundle();
    bundle.steps[0].forward_sql_utf8.push(b' ');

    assert_eq!(
        validate_storage_bundle(&bundle),
        Err(StorageBundleValidationErrorV1::StepDigestMismatch)
    );
}

#[test]
fn rejects_bundle_step_outside_the_additive_ast_allowlist() {
    let mut bundle = valid_bundle();
    bundle.steps[0].forward_sql_utf8 = b"SELECT 1;".to_vec();
    bundle.steps[0].sha256 = Sha256::digest(&bundle.steps[0].forward_sql_utf8).to_vec();

    assert_eq!(
        admit_storage_bundle(&bundle),
        Err(MigrationBundleAdmissionErrorV1::Step {
            revision: 1,
            error: MigrationAdmissionErrorV1::Forbidden,
        })
    );
}
