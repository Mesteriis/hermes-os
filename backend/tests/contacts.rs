use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use hermes_hub_backend::contacts::{
    ContactProjectionError, ContactProjectionStore, upsert_contacts_from_message_participants,
};
use hermes_hub_backend::storage::Database;

#[tokio::test]
async fn contacts_projection_upserts_email_identities_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live contacts projection test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let store = ContactProjectionStore::new(database.pool().expect("configured pool").clone());
    let suffix = unique_suffix();

    let contacts = upsert_contacts_from_message_participants(
        &store,
        &[
            format!("alice-{suffix}@example.com"),
            format!("bob-{suffix}@example.com"),
        ],
    )
    .await
    .expect("upsert contacts");

    assert_eq!(contacts.len(), 2);
    assert!(
        contacts
            .iter()
            .any(|contact| contact.email_address == format!("alice-{suffix}@example.com"))
    );
    assert!(
        contacts
            .iter()
            .any(|contact| contact.email_address == format!("bob-{suffix}@example.com"))
    );
}

#[tokio::test]
async fn contacts_projection_normalizes_and_deduplicates_participants_against_postgres() {
    let Some(store) = live_contacts_store("contacts normalization and deduplication").await else {
        return;
    };
    let suffix = unique_suffix();
    let normalized_email = format!("alice-{suffix}@example.com");

    let contacts = upsert_contacts_from_message_participants(
        &store,
        &[
            format!(" Alice-{suffix}@Example.com "),
            format!("alice-{suffix}@example.com"),
        ],
    )
    .await
    .expect("upsert normalized contacts");

    assert_eq!(contacts.len(), 1);
    assert_eq!(contacts[0].email_address, normalized_email);
    assert_eq!(contacts[0].display_name, normalized_email);
}

#[tokio::test]
async fn contacts_projection_rejects_blank_email_participant() {
    let store = disconnected_contacts_store();

    let error = upsert_contacts_from_message_participants(&store, &[String::from("   ")])
        .await
        .expect_err("blank email input must fail");

    assert!(
        matches!(error, ContactProjectionError::EmptyEmailAddress),
        "expected EmptyEmailAddress, got {error:?}"
    );
}

#[tokio::test]
async fn contacts_projection_rejects_invalid_batch_before_writing_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live contacts invalid batch atomicity test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let store = ContactProjectionStore::new(pool.clone());
    let suffix = unique_suffix();
    let valid_email = format!("valid-before-blank-{suffix}@example.com");

    let error = upsert_contacts_from_message_participants(
        &store,
        &[valid_email.clone(), String::from("   ")],
    )
    .await
    .expect_err("invalid participant batch must fail");

    assert!(
        matches!(error, ContactProjectionError::EmptyEmailAddress),
        "expected EmptyEmailAddress, got {error:?}"
    );

    let count =
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM contacts WHERE email_address = $1")
            .bind(&valid_email)
            .fetch_one(&pool)
            .await
            .expect("contact count after rejected batch");
    assert_eq!(count, 0);
}

#[tokio::test]
async fn contacts_projection_distinguishes_delimiter_bearing_email_identities_against_postgres() {
    let Some(store) = live_contacts_store("delimiter-bearing contact identities").await else {
        return;
    };
    let suffix = unique_suffix();

    let left = store
        .upsert_email_contact(&format!("person:{suffix}@example.com"))
        .await
        .expect("upsert delimiter-bearing contact");
    let right = store
        .upsert_email_contact(&format!("person-{suffix}@example.com"))
        .await
        .expect("upsert non-delimiter contact");

    assert_ne!(left.contact_id, right.contact_id);
    assert!(left.contact_id.starts_with("contact:v1:email:"));
    assert!(right.contact_id.starts_with("contact:v1:email:"));
}

async fn live_contacts_store(test_name: &str) -> Option<ContactProjectionStore> {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live {test_name} test: HERMES_TEST_DATABASE_URL is not set");
        return None;
    };

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    Some(ContactProjectionStore::new(
        database.pool().expect("configured pool").clone(),
    ))
}

fn disconnected_contacts_store() -> ContactProjectionStore {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .connect_lazy("postgres://hermes:unused@127.0.0.1:1/hermes_hub")
        .expect("create lazy test pool");
    ContactProjectionStore::new(pool)
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}
