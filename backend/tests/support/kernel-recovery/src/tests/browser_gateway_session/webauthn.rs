use super::*;

#[test]
fn browser_webauthn_registration_challenge_is_origin_bound_and_user_verifying() {
    let verifier =
        BrowserWebauthnVerifier::new("hub.local", "https://hub.local").expect("verifier");
    let ceremony = verifier
        .begin_registration("owner-1")
        .expect("registration");
    let options = &ceremony.options().public_key;
    assert_eq!(options.rp.id, "hub.local");
    assert_eq!(options.pub_key_cred_params.len(), 1);
    assert_eq!(options.pub_key_cred_params[0].alg, -7);
    assert_eq!(
        format!(
            "{:?}",
            options
                .authenticator_selection
                .as_ref()
                .expect("selection")
                .user_verification
        ),
        "Required"
    );
    assert!(BrowserWebauthnVerifier::new("hub.local", "http://hub.local").is_err());
    assert!(BrowserWebauthnVerifier::new("hub.local", "https://other.local").is_err());
    let malformed =
        BrowserCredentialMaterialV1::new(vec![1], vec![2; 16], 0, false, false).expect("material");
    assert!(verifier.credential_from_material(malformed).is_err());
}
