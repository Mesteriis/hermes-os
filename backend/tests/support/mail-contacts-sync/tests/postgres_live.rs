use hermes_mail_contacts_sync_core::{
    MailContactsSyncDirectionV1, MailContactsSyncDraftV1, MailContactsSyncStateV1,
    MailContactsSyncTransitionV1, MailContactsSyncTriggerV1,
};
use hermes_mail_contacts_sync_persistence::{
    CreateMailContactsSyncOutcomeV1, CreateMailContactsSyncRunV1, MailContactsSyncInboxOutcomeV1,
    MailContactsSyncPersistenceConformanceV1, MailContactsSyncPersistenceErrorV1,
    MailContactsSyncTransitionInputV1, OutboxEnvelopeV1,
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
    assert_eq!(realtime.len(), 2);
    assert_eq!(realtime[0].state, MailContactsSyncStateV1::Accepted);
    assert_eq!(
        realtime[1].state,
        MailContactsSyncStateV1::FetchingProviderPage
    );
    assert_eq!(
        persistence
            .client_realtime_window("owner-1", Some(realtime[0].sequence), 10)
            .await
            .expect("resume SSE replay"),
        vec![realtime[1]]
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
