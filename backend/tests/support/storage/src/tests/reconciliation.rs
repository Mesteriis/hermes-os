use hermes_storage_control::{StorageReconciliationErrorV1, StorageReconciliationV1};
use hermes_storage_protocol::{
    StorageBindingAccessV1, StorageBindingFencesV1, StorageBindingIdentityV1, StorageBindingV1,
    StorageEffectiveBudgetsV1,
};

fn binding(storage_generation: u64) -> StorageBindingV1 {
    let identity = StorageBindingIdentityV1::new(
        "storage_main".into(),
        "hermes".into(),
        "notes".into(),
        "registration_notes".into(),
        "runtime_notes".into(),
    )
    .expect("valid storage identity");
    let fences =
        StorageBindingFencesV1::new(storage_generation, 1, 1, 1, 1, 1).expect("non-zero fences");
    let budgets = StorageEffectiveBudgetsV1::new(4, 5_000).expect("valid budgets");
    let access = StorageBindingAccessV1::new(
        "runtime_notes".into(),
        "pool_notes".into(),
        budgets,
        [1; 32],
    )
    .expect("valid access");
    StorageBindingV1::new(identity, fences, access)
}

#[test]
fn accepts_the_latest_verified_binding() {
    let first = binding(1);
    let latest = binding(2);
    let mut reconciliation = StorageReconciliationV1::default();

    reconciliation
        .accept(first)
        .expect("first binding is accepted");
    reconciliation
        .accept(latest)
        .expect("newer binding is accepted");

    assert_eq!(
        reconciliation
            .binding()
            .map(|binding| binding.fences().storage_generation()),
        Some(2)
    );
}

#[test]
fn rejects_a_stale_storage_generation() {
    let mut reconciliation = StorageReconciliationV1::default();

    reconciliation.accept(binding(2)).expect("initial binding");

    assert_eq!(
        reconciliation.accept(binding(1)),
        Err(StorageReconciliationErrorV1::StaleStorageGeneration)
    );
}
