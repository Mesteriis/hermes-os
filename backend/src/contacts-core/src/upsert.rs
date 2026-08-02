use std::collections::BTreeSet;

use crate::model::normalize_draft;
use crate::{
    ContactIdentityMatchV1, ContactUpsertDraftV1, ContactUpsertOutcomeV1, ContactV1,
    ContactsValidationErrorV1, STABLE_ID_BYTES_V1, derive_contact_id_v1, validate_contact_v1,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContactUpsertDecisionErrorV1 {
    InvalidDraft,
    IdentityAmbiguous,
    ProviderLinkConflict,
    ExistingContactRequired,
}

pub fn decide_contact_upsert_v1(
    draft: ContactUpsertDraftV1,
    identity_match: ContactIdentityMatchV1,
    existing: Option<&ContactV1>,
) -> Result<(ContactV1, ContactUpsertOutcomeV1), ContactUpsertDecisionErrorV1> {
    let normalized = normalize_draft(&draft).map_err(invalid_draft)?;
    let target = choose_target(&identity_match)?;
    if normalized.email_addresses.is_empty()
        && normalized.phone_numbers.is_empty()
        && target.is_none()
    {
        return Err(ContactUpsertDecisionErrorV1::ExistingContactRequired);
    }

    match (target, existing) {
        (Some(target), Some(current)) if current.contact_id == target => {
            validate_contact_v1(current).map_err(invalid_draft)?;
            if current.logical_owner_id != normalized.logical_owner_id {
                return Err(ContactUpsertDecisionErrorV1::ProviderLinkConflict);
            }
            let changed = current.display_name != normalized.display_name
                || current.email_addresses != normalized.email_addresses
                || current.phone_numbers != normalized.phone_numbers
                || current.provenance != normalized.provenance;
            if !changed {
                return Ok((current.clone(), ContactUpsertOutcomeV1::Unchanged));
            }
            let updated = ContactV1 {
                contact_id: current.contact_id,
                logical_owner_id: current.logical_owner_id.clone(),
                display_name: normalized.display_name,
                email_addresses: normalized.email_addresses,
                phone_numbers: normalized.phone_numbers,
                contact_revision: current.contact_revision + 1,
                provenance: normalized.provenance,
                created_at: current.created_at,
                updated_at: draft.provenance.observed_at,
            };
            validate_contact_v1(&updated).map_err(invalid_draft)?;
            Ok((updated, ContactUpsertOutcomeV1::Updated))
        }
        (Some(_), _) => Err(ContactUpsertDecisionErrorV1::ProviderLinkConflict),
        (None, Some(_)) => Err(ContactUpsertDecisionErrorV1::ProviderLinkConflict),
        (None, None) => {
            let contact = ContactV1 {
                contact_id: derive_contact_id_v1(
                    &normalized.logical_owner_id,
                    &normalized.provenance,
                )
                .map_err(invalid_draft)?,
                logical_owner_id: normalized.logical_owner_id,
                display_name: normalized.display_name,
                email_addresses: normalized.email_addresses,
                phone_numbers: normalized.phone_numbers,
                contact_revision: 1,
                provenance: normalized.provenance,
                created_at: draft.provenance.observed_at,
                updated_at: draft.provenance.observed_at,
            };
            validate_contact_v1(&contact).map_err(invalid_draft)?;
            Ok((contact, ContactUpsertOutcomeV1::Created))
        }
    }
}

fn choose_target(
    identity_match: &ContactIdentityMatchV1,
) -> Result<Option<[u8; STABLE_ID_BYTES_V1]>, ContactUpsertDecisionErrorV1> {
    let identity_targets: BTreeSet<_> = identity_match
        .email_contact_ids
        .iter()
        .chain(&identity_match.phone_contact_ids)
        .copied()
        .collect();
    if identity_targets.len() > 1 {
        return Err(ContactUpsertDecisionErrorV1::IdentityAmbiguous);
    }
    if let Some(linked) = identity_match.provider_link_contact_id {
        if identity_targets.iter().any(|target| *target != linked) {
            return Err(ContactUpsertDecisionErrorV1::ProviderLinkConflict);
        }
        return Ok(Some(linked));
    }
    Ok(identity_targets.into_iter().next())
}

fn invalid_draft(_: ContactsValidationErrorV1) -> ContactUpsertDecisionErrorV1 {
    ContactUpsertDecisionErrorV1::InvalidDraft
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ContactProviderKindV1, ContactProviderProvenanceV1, ContactTimestampV1};

    fn draft() -> ContactUpsertDraftV1 {
        ContactUpsertDraftV1 {
            logical_owner_id: "owner-1".to_owned(),
            display_name: " Ada Lovelace ".to_owned(),
            email_addresses: vec!["ADA@EXAMPLE.TEST".to_owned()],
            phone_numbers: vec!["+34 (910) 000-000".to_owned()],
            provenance: ContactProviderProvenanceV1 {
                source_account_id: "mail-account-1".to_owned(),
                provider_kind: ContactProviderKindV1::Gmail,
                provider_entry_id: "people/c123".to_owned(),
                provider_etag: Some("etag-1".to_owned()),
                source_revision: 4,
                entry_digest: [9; 32],
                observed_at: ContactTimestampV1 {
                    unix_seconds: 1_800_000_000,
                    nanos: 3,
                },
            },
        }
    }

    fn no_match() -> ContactIdentityMatchV1 {
        ContactIdentityMatchV1 {
            provider_link_contact_id: None,
            email_contact_ids: Vec::new(),
            phone_contact_ids: Vec::new(),
        }
    }

    #[test]
    fn provider_entry_creates_deterministic_normalized_contact() {
        let (first, outcome) =
            decide_contact_upsert_v1(draft(), no_match(), None).expect("contact");
        let (second, _) = decide_contact_upsert_v1(draft(), no_match(), None).expect("contact");
        assert_eq!(outcome, ContactUpsertOutcomeV1::Created);
        assert_eq!(first, second);
        assert_eq!(first.email_addresses, ["ada@example.test"]);
        assert_eq!(first.phone_numbers, ["+34910000000"]);
    }

    #[test]
    fn identities_pointing_to_different_contacts_fail_closed() {
        let matches = ContactIdentityMatchV1 {
            provider_link_contact_id: None,
            email_contact_ids: vec![[1; 16]],
            phone_contact_ids: vec![[2; 16]],
        };
        assert_eq!(
            decide_contact_upsert_v1(draft(), matches, None),
            Err(ContactUpsertDecisionErrorV1::IdentityAmbiguous)
        );
    }

    #[test]
    fn provider_link_cannot_be_overridden_by_identity_match() {
        let matches = ContactIdentityMatchV1 {
            provider_link_contact_id: Some([1; 16]),
            email_contact_ids: vec![[2; 16]],
            phone_contact_ids: Vec::new(),
        };
        assert_eq!(
            decide_contact_upsert_v1(draft(), matches, None),
            Err(ContactUpsertDecisionErrorV1::ProviderLinkConflict)
        );
    }

    #[test]
    fn name_only_entry_needs_existing_provider_link() {
        let mut name_only = draft();
        name_only.email_addresses.clear();
        name_only.phone_numbers.clear();
        assert_eq!(
            decide_contact_upsert_v1(name_only, no_match(), None),
            Err(ContactUpsertDecisionErrorV1::ExistingContactRequired)
        );
    }
}
