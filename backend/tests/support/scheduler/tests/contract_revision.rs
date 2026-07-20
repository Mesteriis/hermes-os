use hermes_scheduler_persistence::scheduler_storage_bundle_v1;
use hermes_scheduler_protocol::{JobContractBindingV1, JobKindV1};

#[test]
fn scheduler_persists_an_exact_nonzero_owner_job_contract_revision() {
    let job_kind =
        JobKindV1::new("notes".to_owned(), "refresh".to_owned(), 1).expect("valid job kind");
    assert!(
        JobContractBindingV1::new(job_kind.clone(), "notes.refresh".to_owned(), 0, [7; 32],)
            .is_err()
    );
    let binding = JobContractBindingV1::new(job_kind, "notes.refresh".to_owned(), 3, [7; 32])
        .expect("valid exact contract binding");
    assert_eq!(binding.contract_revision(), 3);

    let bundle = scheduler_storage_bundle_v1();
    assert_eq!(bundle.revision, 7);
    let step = bundle
        .steps
        .iter()
        .find(|step| step.revision == 7)
        .expect("job contract revision migration");
    assert_eq!(step.migration_id, "scheduler_job_contract_revision");
    assert!(
        String::from_utf8_lossy(&step.forward_sql_utf8).contains("contract_revision"),
        "migration must persist the exact owner job contract revision"
    );
}
