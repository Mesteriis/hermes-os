use super::*;

#[test]
fn browser_gateway_authority_persists_the_verified_assertion_counter() {
    let root = unique_target_root("hermes-browser-gateway-assertion");
    std::fs::create_dir_all(&root).expect("create fixture directory");
    let path = root.join("control.sqlite");
    let store =
        Arc::new(SqliteControlStore::create(&path, "instance-browser", 1).expect("create store"));
    store
        .claim_initial_owner(&InitialOwnerIdentity::new("owner-1", "desktop-1", [4; 65]))
        .expect("claim initial owner");
    let enrollment = BrowserDeviceEnrollmentV1::new(
        "owner-1",
        "browser-1",
        vec![1],
        vec![2; 16],
        vec![4; 65],
        "hub.local",
        0,
        false,
        false,
    )
    .expect("valid browser enrollment");
    store
        .admit_browser_device(&enrollment, 1)
        .expect("admit browser");
    let authority = browser_authority(Arc::clone(&store));
    authority
        .accept_verified_browser_assertion(&[1], 3, false, false)
        .expect("persist verified assertion");
    let identity = store
        .browser_device_identity("browser-1")
        .expect("read browser device")
        .expect("browser device exists");
    assert_eq!(identity.enrollment().sign_count(), 3);
    assert!(
        authority
            .accept_verified_browser_assertion(&[1], 3, false, false)
            .is_err()
    );
    std::fs::remove_dir_all(root).expect("remove fixture directory");
}
