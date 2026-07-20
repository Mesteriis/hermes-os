use super::*;

#[test]
fn browser_pairing_is_single_use_and_fenced_by_owner_identity_epoch() {
    let root = unique_target_root("hermes-browser-gateway-pairing");
    std::fs::create_dir_all(&root).expect("create fixture directory");
    let store = Arc::new(
        SqliteControlStore::create(&root.join("control.sqlite"), "instance-browser", 1)
            .expect("create store"),
    );
    store
        .claim_initial_owner(&InitialOwnerIdentity::new("owner-1", "desktop-1", [4; 65]))
        .expect("claim initial owner");
    let authority = browser_authority(Arc::clone(&store));
    let approval = OwnerPairingApprovalV1::new("owner-1", "desktop-1").expect("approval");
    let mut pairings = BrowserPairingManager::default();
    let challenge = pairings
        .begin(&authority, approval.clone(), "hub.local", 1_000)
        .expect("begin pairing");
    let consumed = pairings
        .consume(&authority, challenge.pairing_id(), |presented| {
            (presented.challenge_bytes() == challenge.challenge_bytes())
                .then_some("verified")
                .ok_or_else(|| "challenge mismatch".to_owned())
        })
        .expect("consume pairing");
    assert_eq!(consumed, "verified");
    assert!(
        pairings
            .consume(&authority, challenge.pairing_id(), |_| Ok(()))
            .is_err()
    );
    let stale = pairings
        .begin(&authority, approval, "hub.local", 1_000)
        .expect("begin pairing");
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
    .expect("enrollment");
    store
        .admit_browser_device(&enrollment, 1)
        .expect("admit browser");
    store
        .revoke_browser_device("browser-1", 1)
        .expect("rotate epoch");
    assert!(
        pairings
            .consume(&authority, stale.pairing_id(), |_| Ok(()))
            .is_err()
    );
    std::fs::remove_dir_all(root).expect("remove fixture directory");
}
