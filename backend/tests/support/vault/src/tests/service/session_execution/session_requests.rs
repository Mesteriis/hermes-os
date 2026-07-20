use super::*;

pub(super) fn audience() -> LeaseAudienceV1 {
    LeaseAudienceV1::new(
        "registration-mail".to_owned(),
        "runtime-mail-1".to_owned(),
        1,
        7,
    )
    .expect("typed audience")
}

pub(super) fn lease_request(
    purpose: VaultPurposeRequestV1,
    audience: LeaseAudienceV1,
) -> VaultLeaseIssueRequestV1 {
    lease_request_at(purpose, audience, 1)
}

pub(super) fn lease_request_at(
    purpose: VaultPurposeRequestV1,
    audience: LeaseAudienceV1,
    revision: u64,
) -> VaultLeaseIssueRequestV1 {
    VaultLeaseIssueRequestV1::new(
        "vault-instance".to_owned(),
        3,
        revision,
        "mail".to_owned(),
        purpose,
        audience,
    )
    .expect("typed lease request")
}
