//! Kernel launch proof for the production Blob service binary.

use super::super::common::*;
use super::fixture::BlobServiceFixture;
use crate::platform::blob::{binding, status};

#[test]
#[ignore = "builds and launches the real Blob service binary"]
fn kernel_starts_signed_blob_service_with_current_vault_status() {
    let fixture = BlobServiceFixture::new();
    let store = fixture.store();
    let supervisor = ManagedRuntimeSupervisor::new(Arc::new(AtomicBool::new(false)));

    fixture.start_vault(&supervisor);
    binding::bind_installed_release(&store, fixture.release_kernel())
        .expect("bind signed Blob release");
    assert_eq!(fixture.start_blob(&supervisor, &store), 1);

    let launch = store
        .platform_managed_process_launch("blob")
        .expect("read Blob launch")
        .expect("Blob launch record");
    assert_eq!(launch.runtime_generation(), 1);
    assert_eq!(launch.grant_epoch(), store.snapshot().grant_epoch());
    assert_eq!(
        status::read_current(&store, &supervisor.relay_port())
            .expect("Blob status")
            .runtime_generation(),
        1
    );

    supervisor.shutdown().expect("stop managed children");
}
