use super::errors::PersonProjectionError;

pub(super) fn normalize_email_address(
    email_address: &str,
) -> Result<String, PersonProjectionError> {
    let normalized_email = email_addr_spec(email_address).trim().to_ascii_lowercase();
    if normalized_email.is_empty() {
        return Err(PersonProjectionError::EmptyEmailAddress);
    }
    if !normalized_email.contains('@') {
        return Err(PersonProjectionError::InvalidEmailAddress(normalized_email));
    }

    Ok(normalized_email)
}

fn email_addr_spec(value: &str) -> &str {
    let value = value.trim();
    if let Some((_, tail)) = value.rsplit_once('<')
        && let Some((addr, _)) = tail.split_once('>')
    {
        return addr.trim();
    }
    value.trim_matches('"')
}

pub(super) fn person_id_for_email(normalized_email: &str) -> String {
    let mut encoded = String::from("person:v1:email:");
    encoded.push_str(&normalized_email.len().to_string());
    encoded.push(':');
    encoded.push_str(normalized_email);
    encoded
}

pub(super) fn normalize_ai_agent_id(agent_id: &str) -> Result<String, PersonProjectionError> {
    let normalized = agent_id.trim().to_ascii_uppercase();
    if normalized.is_empty() {
        return Err(PersonProjectionError::EmptyAiAgentId);
    }
    if !normalized
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
    {
        return Err(PersonProjectionError::InvalidAiAgentId(agent_id.to_owned()));
    }
    Ok(normalized)
}

pub(super) fn validate_display_name(display_name: &str) -> Result<String, PersonProjectionError> {
    let display_name = display_name.trim();
    if display_name.is_empty() {
        return Err(PersonProjectionError::EmptyDisplayName);
    }
    Ok(display_name.to_owned())
}

pub(super) fn ai_agent_person_id(agent_id: &str) -> String {
    format!("persona:v1:ai_agent:{agent_id}")
}

pub(super) fn ai_agent_email_address(agent_id: &str) -> String {
    format!("{}@sh-inc.ru", agent_id.to_ascii_lowercase())
}
