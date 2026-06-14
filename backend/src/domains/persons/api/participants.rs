use std::collections::HashSet;

use super::errors::PersonProjectionError;
use super::models::Person;
use super::store::PersonProjectionStore;
use super::validation::normalize_email_address;

pub async fn upsert_persons_from_message_participants(
    store: &PersonProjectionStore,
    email_addresses: &[String],
) -> Result<Vec<Person>, PersonProjectionError> {
    let normalized_email_addresses = normalize_email_addresses(email_addresses)?;
    let mut persons = Vec::new();

    for email_address in normalized_email_addresses {
        persons.push(store.upsert_email_person(&email_address).await?);
    }

    Ok(persons)
}

fn normalize_email_addresses(
    email_addresses: &[String],
) -> Result<Vec<String>, PersonProjectionError> {
    let mut seen = HashSet::new();
    let mut normalized_email_addresses = Vec::new();

    for email_address in email_addresses {
        let normalized_email = normalize_email_address(email_address)?;
        if seen.insert(normalized_email.clone()) {
            normalized_email_addresses.push(normalized_email);
        }
    }

    Ok(normalized_email_addresses)
}
