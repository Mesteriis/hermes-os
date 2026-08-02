use hermes_contacts_core::{
    ContactProviderKindV1, ContactProviderProvenanceV1, ContactTimestampV1, ContactUpsertDraftV1,
    ContactUpsertOutcomeV1,
};
use hermes_contacts_persistence::{
    ApplyMailEntryCommandV1, ContactMailEntryRejectCodeV1, ContactsOutboxRecordV1,
    ContactsPersistenceConformanceV1, ContactsPersistenceErrorV1, RejectMailEntryCommandV1,
};
use sha2::{Digest, Sha256};

#[tokio::test]
#[ignore = "requires the disposable authenticated PostgreSQL contour"]
async fn postgres_replays_exact_result_and_fences_conflicts() {
    let database_url =
        std::env::var("HERMES_CONTACTS_POSTGRES_URL").expect("HERMES_CONTACTS_POSTGRES_URL");
    let persistence = ContactsPersistenceConformanceV1::connect_url(&database_url)
        .await
        .expect("connect Contacts persistence");
    ContactsPersistenceConformanceV1::install_schema(&persistence)
        .await
        .expect("install Contacts schema");
    let first = command(1, 1, "ada@example.test", "+34910000001");
    let created = persistence
        .apply_mail_entry(&first, |contact, outcome| {
            terminal(11, contact.contact_id, outcome)
        })
        .await
        .expect("create contact");
    assert_eq!(created.outcome, ContactUpsertOutcomeV1::Created);
    assert_eq!(created.contact_revision, 1);
    assert!(!created.replayed);

    let first_replay = persistence
        .apply_mail_entry(&first, |_, _| {
            panic!("replay must load the persisted exact result")
        })
        .await
        .expect("replay first command");
    assert!(first_replay.replayed);
    assert_eq!(first_replay.contact_revision, 1);
    assert_eq!(first_replay.terminal_result, created.terminal_result);

    let updated_input = command(2, 2, "ada@example.test", "+34910000001");
    let updated = persistence
        .apply_mail_entry(&updated_input, |contact, outcome| {
            terminal(12, contact.contact_id, outcome)
        })
        .await
        .expect("update contact");
    assert_eq!(updated.contact_id, created.contact_id);
    assert_eq!(updated.outcome, ContactUpsertOutcomeV1::Updated);
    assert_eq!(updated.contact_revision, 2);

    let replay_after_update = persistence
        .apply_mail_entry(&first, |_, _| panic!("replay must not rebuild the result"))
        .await
        .expect("replay after later update");
    assert_eq!(replay_after_update.contact_revision, 1);
    assert_eq!(replay_after_update.terminal_result, created.terminal_result);

    let unchanged_input = command(3, 2, "ada@example.test", "+34910000001");
    let unchanged = persistence
        .apply_mail_entry(&unchanged_input, |contact, outcome| {
            terminal(13, contact.contact_id, outcome)
        })
        .await
        .expect("unchanged contact");
    assert_eq!(unchanged.outcome, ContactUpsertOutcomeV1::Unchanged);
    assert_eq!(unchanged.contact_revision, 2);

    let mut reused_command_id = command(4, 3, "ada@example.test", "+34910000001");
    reused_command_id.command_id = first.command_id;
    assert_eq!(
        persistence
            .apply_mail_entry(&reused_command_id, |contact, outcome| {
                terminal(14, contact.contact_id, outcome)
            })
            .await,
        Err(ContactsPersistenceErrorV1::CommandConflict)
    );

    let second = command(5, 1, "grace@example.test", "+34910000002");
    persistence
        .apply_mail_entry(&second, |contact, outcome| {
            terminal(15, contact.contact_id, outcome)
        })
        .await
        .expect("create second contact");
    let ambiguous = command(6, 4, "ada@example.test", "+34910000002");
    assert_eq!(
        persistence
            .apply_mail_entry(&ambiguous, |contact, outcome| {
                terminal(16, contact.contact_id, outcome)
            })
            .await,
        Err(ContactsPersistenceErrorV1::IdentityAmbiguous)
    );
    let rejected_result = terminal(16, ambiguous.command_id, ContactUpsertOutcomeV1::Unchanged)
        .expect("rejected terminal fixture");
    let rejection = RejectMailEntryCommandV1 {
        command_message_id: ambiguous.command_message_id,
        command_envelope_sha256: ambiguous.command_envelope_sha256,
        command_id: ambiguous.command_id,
        logical_owner_id: ambiguous.draft.logical_owner_id.clone(),
        entry_digest: ambiguous.draft.provenance.entry_digest,
        received_at_unix_millis: ambiguous.received_at_unix_millis,
        completed_at_unix_millis: ambiguous.completed_at_unix_millis,
        code: ContactMailEntryRejectCodeV1::IdentityAmbiguous,
        terminal_result: rejected_result.clone(),
    };
    let rejected = persistence
        .reject_mail_entry(&rejection)
        .await
        .expect("persist rejection");
    assert!(!rejected.replayed);
    assert_eq!(rejected.terminal_result, rejected_result);
    let replayed_rejection = persistence
        .reject_mail_entry(&rejection)
        .await
        .expect("replay rejection");
    assert!(replayed_rejection.replayed);
    assert_eq!(replayed_rejection.terminal_result, rejected_result);
    assert_eq!(
        persistence
            .apply_mail_entry(&ambiguous, |_, _| panic!("rejected command cannot apply"))
            .await,
        Err(ContactsPersistenceErrorV1::IdentityAmbiguous)
    );

    let pending = persistence
        .load_pending_outbox("owner-1")
        .await
        .expect("pending outbox");
    assert_eq!(pending.len(), 5);
}

fn command(seed: u8, source_revision: u64, email: &str, phone: &str) -> ApplyMailEntryCommandV1 {
    ApplyMailEntryCommandV1 {
        command_message_id: [seed; 16],
        command_envelope_sha256: [seed.wrapping_add(20); 32],
        command_id: [seed.wrapping_add(40); 16],
        draft: ContactUpsertDraftV1 {
            logical_owner_id: "owner-1".to_owned(),
            display_name: if email.starts_with("ada") {
                "Ada"
            } else {
                "Grace"
            }
            .to_owned(),
            email_addresses: vec![email.to_owned()],
            phone_numbers: vec![phone.to_owned()],
            provenance: ContactProviderProvenanceV1 {
                source_account_id: "mail-1".to_owned(),
                provider_kind: ContactProviderKindV1::Gmail,
                provider_entry_id: if email.starts_with("ada") {
                    "people/ada"
                } else {
                    "people/grace"
                }
                .to_owned(),
                provider_etag: Some(format!("etag-{source_revision}")),
                source_revision,
                entry_digest: [source_revision as u8; 32],
                observed_at: ContactTimestampV1 {
                    unix_seconds: 1_800_000_000 + i64::try_from(source_revision).expect("revision"),
                    nanos: 0,
                },
            },
        },
        received_at_unix_millis: 1_800_000_000_000 + i64::from(seed),
        completed_at_unix_millis: 1_800_000_000_100 + i64::from(seed),
    }
}

fn terminal(
    seed: u8,
    contact_id: [u8; 16],
    outcome: ContactUpsertOutcomeV1,
) -> Result<ContactsOutboxRecordV1, ContactsPersistenceErrorV1> {
    let outcome = match outcome {
        ContactUpsertOutcomeV1::Created => b"created".as_slice(),
        ContactUpsertOutcomeV1::Updated => b"updated".as_slice(),
        ContactUpsertOutcomeV1::Unchanged => b"unchanged".as_slice(),
    };
    let mut bytes = b"contacts-terminal-v1:".to_vec();
    bytes.extend_from_slice(&contact_id);
    bytes.extend_from_slice(outcome);
    Ok(ContactsOutboxRecordV1 {
        message_id: [seed; 16],
        envelope_sha256: Sha256::digest(&bytes).into(),
        envelope_bytes: bytes,
    })
}
