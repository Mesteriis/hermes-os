use hermes_mail_contacts_sync_core::{
    MailContactsSyncDirectionV1, MailContactsSyncDraftV1, MailContactsSyncStateV1,
    MailContactsSyncTransitionV1, MailContactsSyncTriggerV1,
};
use hermes_mail_contacts_sync_persistence::{
    CreateMailContactsSyncOutcomeV1, CreateMailContactsSyncRunV1, MailContactsSyncContactOutcomeV1,
    MailContactsSyncEntryInputV1, MailContactsSyncEntryOutcomeInputV1,
    MailContactsSyncInboxOutcomeV1, MailContactsSyncPageResultInputV1,
    MailContactsSyncPersistenceConformanceV1, MailContactsSyncPersistenceErrorV1,
    MailContactsSyncPersistenceOutcomeV1, MailContactsSyncTransitionInputV1, OutboxEnvelopeV1,
};
use sha2::{Digest, Sha256};

#[tokio::test]
#[ignore = "requires the disposable authenticated PostgreSQL contour"]
async fn postgres_is_atomic_replayable_and_sse_replayable() {
    let database_url = std::env::var("HERMES_MAIL_CONTACTS_SYNC_POSTGRES_URL")
        .expect("HERMES_MAIL_CONTACTS_SYNC_POSTGRES_URL");
    let persistence = MailContactsSyncPersistenceConformanceV1::connect_url(&database_url)
        .await
        .expect("connect workflow persistence");
    MailContactsSyncPersistenceConformanceV1::install_schema(&persistence)
        .await
        .expect("install workflow schema");

    let create = create_run(1);
    let created = persistence
        .create_run(create.clone())
        .await
        .expect("create run");
    assert!(matches!(
        created,
        CreateMailContactsSyncOutcomeV1::Created(_)
    ));
    let replayed = persistence
        .create_run(create.clone())
        .await
        .expect("replay start");
    assert!(matches!(
        replayed,
        CreateMailContactsSyncOutcomeV1::Existing(_)
    ));

    let pending = persistence
        .unpublished_commands("owner-1", 10)
        .await
        .expect("load initial outbox");
    assert_eq!(pending, vec![create.initial_command.clone()]);
    persistence
        .mark_command_published(
            "owner-1",
            &pending[0].message_id,
            &pending[0].envelope_sha256,
            1_800_000_000_100,
        )
        .await
        .expect("mark initial command published");
    assert!(
        persistence
            .unpublished_commands("owner-1", 10)
            .await
            .expect("load empty outbox")
            .is_empty()
    );

    let transition = MailContactsSyncTransitionInputV1 {
        logical_owner_id: "owner-1".to_owned(),
        run_id: [1; 16],
        direction: MailContactsSyncDirectionV1::ProviderToContacts,
        message_id: [3; 16],
        envelope_sha256: [4; 32],
        transition: MailContactsSyncTransitionV1::BeginProviderPage,
        next_command: Some(envelope(5, b"fetch-page-command")),
        occurred_at_unix_millis: 1_800_000_000_200,
    };
    let applied = persistence
        .apply_transition(transition.clone())
        .await
        .expect("apply transition");
    let applied = match applied {
        MailContactsSyncInboxOutcomeV1::Applied(run) => run,
        MailContactsSyncInboxOutcomeV1::Duplicate(_) => panic!("first delivery cannot replay"),
    };
    assert_eq!(
        applied.status.state,
        MailContactsSyncStateV1::FetchingProviderPage
    );
    assert!(matches!(
        persistence
            .apply_transition(transition.clone())
            .await
            .expect("replay transition"),
        MailContactsSyncInboxOutcomeV1::Duplicate(_)
    ));

    let entry = MailContactsSyncEntryInputV1 {
        logical_owner_id: "owner-1".to_owned(),
        run_id: [1; 16],
        page_sequence: 1,
        observation_message_id: [21; 16],
        observation_envelope_sha256: [22; 32],
        contact_command_id: [23; 16],
        entry_digest: [24; 32],
        contact_command: envelope(23, b"contacts-upsert-command"),
        occurred_at_unix_millis: 1_800_000_000_300,
    };
    assert_eq!(
        persistence
            .accept_provider_entry(&entry)
            .await
            .expect("accept provider entry"),
        MailContactsSyncPersistenceOutcomeV1::Applied
    );
    assert_eq!(
        persistence
            .accept_provider_entry(&entry)
            .await
            .expect("replay provider entry"),
        MailContactsSyncPersistenceOutcomeV1::Duplicate
    );
    let outcome = MailContactsSyncEntryOutcomeInputV1 {
        logical_owner_id: "owner-1".to_owned(),
        contact_command_id: [23; 16],
        message_id: [25; 16],
        envelope_sha256: [26; 32],
        outcome: MailContactsSyncContactOutcomeV1::Created,
        occurred_at_unix_millis: 1_800_000_000_400,
    };
    persistence
        .accept_contact_outcome(&outcome)
        .await
        .expect("accept early Contacts result");
    let before_page = persistence
        .load_run("owner-1", &[1; 16])
        .await
        .expect("load before page completion");
    assert_eq!(before_page.status.counters.contacts_created, 0);

    let page = MailContactsSyncPageResultInputV1 {
        logical_owner_id: "owner-1".to_owned(),
        run_id: [1; 16],
        page_sequence: 1,
        message_id: [27; 16],
        envelope_sha256: [28; 32],
        observed_entries: 1,
        next_continuation_cursor: None,
        occurred_at_unix_millis: 1_800_000_000_500,
    };
    persistence
        .accept_provider_page(&page)
        .await
        .expect("complete provider page");
    let after_page = persistence
        .load_run("owner-1", &[1; 16])
        .await
        .expect("load after page completion");
    assert_eq!(
        after_page.status.state,
        MailContactsSyncStateV1::ApplyingContacts
    );
    assert_eq!(after_page.status.counters.provider_entries_seen, 1);
    assert_eq!(after_page.status.counters.contacts_created, 1);
    let progress = persistence
        .page_progress("owner-1", &[1; 16])
        .await
        .expect("load page progress");
    assert_eq!(progress.expected_entries, 1);
    assert_eq!(progress.recorded_entries, 1);
    assert_eq!(progress.accounted_entries, 1);

    let second = create_run(30);
    persistence
        .create_run(second)
        .await
        .expect("create concurrent-order run");
    persistence
        .apply_transition(MailContactsSyncTransitionInputV1 {
            logical_owner_id: "owner-1".to_owned(),
            run_id: [30; 16],
            direction: MailContactsSyncDirectionV1::ProviderToContacts,
            message_id: [41; 16],
            envelope_sha256: [42; 32],
            transition: MailContactsSyncTransitionV1::BeginProviderPage,
            next_command: Some(envelope(43, b"second-fetch-page")),
            occurred_at_unix_millis: 1_800_000_000_600,
        })
        .await
        .expect("begin concurrent-order page");
    persistence
        .accept_provider_entry(&MailContactsSyncEntryInputV1 {
            logical_owner_id: "owner-1".to_owned(),
            run_id: [30; 16],
            page_sequence: 1,
            observation_message_id: [44; 16],
            observation_envelope_sha256: [45; 32],
            contact_command_id: [46; 16],
            entry_digest: [47; 32],
            contact_command: envelope(46, b"second-contacts-upsert"),
            occurred_at_unix_millis: 1_800_000_000_700,
        })
        .await
        .expect("accept concurrent-order entry");
    let concurrent_outcome = MailContactsSyncEntryOutcomeInputV1 {
        logical_owner_id: "owner-1".to_owned(),
        contact_command_id: [46; 16],
        message_id: [48; 16],
        envelope_sha256: [49; 32],
        outcome: MailContactsSyncContactOutcomeV1::Updated,
        occurred_at_unix_millis: 1_800_000_000_800,
    };
    let concurrent_page = MailContactsSyncPageResultInputV1 {
        logical_owner_id: "owner-1".to_owned(),
        run_id: [30; 16],
        page_sequence: 1,
        message_id: [50; 16],
        envelope_sha256: [51; 32],
        observed_entries: 1,
        next_continuation_cursor: None,
        occurred_at_unix_millis: 1_800_000_000_800,
    };
    let (outcome_result, page_result) = tokio::join!(
        persistence.accept_contact_outcome(&concurrent_outcome),
        persistence.accept_provider_page(&concurrent_page),
    );
    outcome_result.expect("concurrent Contacts result");
    page_result.expect("concurrent page completion");
    let concurrent_run = persistence
        .load_run("owner-1", &[30; 16])
        .await
        .expect("load concurrent-order run");
    assert_eq!(concurrent_run.status.counters.contacts_updated, 1);

    let mut conflict = transition;
    conflict.envelope_sha256 = [9; 32];
    assert_eq!(
        persistence.apply_transition(conflict).await,
        Err(MailContactsSyncPersistenceErrorV1::InboxConflict)
    );
    let realtime = persistence
        .client_realtime_window("owner-1", None, 10)
        .await
        .expect("initial SSE replay");
    assert_eq!(realtime.len(), 6);
    assert_eq!(
        realtime.iter().map(|item| item.state).collect::<Vec<_>>(),
        [
            MailContactsSyncStateV1::Accepted,
            MailContactsSyncStateV1::FetchingProviderPage,
            MailContactsSyncStateV1::ApplyingContacts,
            MailContactsSyncStateV1::Accepted,
            MailContactsSyncStateV1::FetchingProviderPage,
            MailContactsSyncStateV1::ApplyingContacts,
        ]
    );
    assert_eq!(
        persistence
            .client_realtime_window("owner-1", Some(realtime[0].sequence), 10)
            .await
            .expect("resume SSE replay"),
        realtime[1..].to_vec()
    );
}

fn create_run(seed: u8) -> CreateMailContactsSyncRunV1 {
    CreateMailContactsSyncRunV1 {
        logical_owner_id: "owner-1".to_owned(),
        draft: MailContactsSyncDraftV1 {
            run_id: [seed; 16],
            operation_id: [seed.wrapping_add(1); 16],
            account_id: "mail-account-1".to_owned(),
            direction: MailContactsSyncDirectionV1::ProviderToContacts,
            trigger: MailContactsSyncTriggerV1::Manual,
        },
        initial_command: envelope(seed.wrapping_add(10), b"initial-command"),
        created_at_unix_millis: 1_800_000_000_000,
    }
}

fn envelope(seed: u8, value: &[u8]) -> OutboxEnvelopeV1 {
    OutboxEnvelopeV1 {
        message_id: [seed; 16],
        envelope_sha256: Sha256::digest(value).into(),
        envelope_bytes: value.to_vec(),
    }
}
