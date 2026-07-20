use super::*;

#[test]
fn browser_pairing_enrollment_is_persisted_only_at_its_exact_identity_fence() {
    let root = unique_target_root("hermes-browser-gateway-enrollment");
    std::fs::create_dir_all(&root).expect("create fixture directory");
    let store = Arc::new(
        SqliteControlStore::create(&root.join("control.sqlite"), "instance-browser", 1)
            .expect("create store"),
    );
    store
        .claim_initial_owner(&InitialOwnerIdentity::new("owner-1", "desktop-1", [4; 65]))
        .expect("claim initial owner");
    let authority = browser_authority(Arc::clone(&store));
    let enrollment = BrowserEnrollmentV1::new(
        "owner-1",
        "browser-1",
        "hub.local",
        vec![1; 16],
        vec![2; 16],
        vec![4; 65],
        0,
        false,
        false,
        GatewayIdentityFenceV1::new("instance-browser", 1, 1).expect("identity fence"),
    )
    .expect("browser enrollment");
    let principal = authority
        .admit_browser_device(&enrollment)
        .expect("admit paired browser");
    assert_eq!(principal.device_id(), "browser-1");
    let stale = BrowserEnrollmentV1::new(
        "owner-1",
        "browser-2",
        "hub.local",
        vec![3; 16],
        vec![4; 16],
        vec![4; 65],
        0,
        false,
        false,
        GatewayIdentityFenceV1::new("instance-browser", 1, 2).expect("stale fence"),
    )
    .expect("stale enrollment");
    assert!(authority.admit_browser_device(&stale).is_err());
    std::fs::remove_dir_all(root).expect("remove fixture directory");
}
