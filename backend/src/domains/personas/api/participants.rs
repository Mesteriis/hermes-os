use std::collections::HashSet;

use super::errors::PersonaProjectionError;
use super::models::Persona;
use super::store::PersonaProjectionStore;
use super::validation::normalize_email_address;

pub async fn upsert_personas_from_message_participants(
    store: &PersonaProjectionStore,
    email_addresses: &[String],
) -> Result<Vec<Persona>, PersonaProjectionError> {
    let normalized_email_addresses = normalize_email_addresses(email_addresses)?;
    let mut personas = Vec::new();

    for email_address in normalized_email_addresses {
        personas.push(store.upsert_email_persona(&email_address).await?);
    }

    Ok(personas)
}

fn normalize_email_addresses(
    email_addresses: &[String],
) -> Result<Vec<String>, PersonaProjectionError> {
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
