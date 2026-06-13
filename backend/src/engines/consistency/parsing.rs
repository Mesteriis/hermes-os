pub(super) fn parse_evidence_claim_line(line: &str) -> Option<(String, String)> {
    parse_structured_claim_line(line).or_else(|| parse_natural_language_claim_line(line))
}

fn parse_structured_claim_line(line: &str) -> Option<(String, String)> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }

    let delimiter_index = match (trimmed.find(':'), trimmed.find('=')) {
        (Some(colon), Some(equals)) => Some(colon.min(equals)),
        (Some(colon), None) => Some(colon),
        (None, Some(equals)) => Some(equals),
        (None, None) => None,
    }?;

    let raw_claim_type = trimmed[..delimiter_index].trim();
    let value = trimmed[delimiter_index + 1..].trim();
    if raw_claim_type.is_empty() || value.is_empty() {
        return None;
    }

    let claim_type = raw_claim_type
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("_")
        .to_lowercase();
    if claim_type.is_empty()
        || !claim_type
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '-'))
        || !is_supported_deterministic_claim_type(&claim_type)
    {
        return None;
    }

    Some((claim_type, normalize_extracted_claim_value(value)?))
}

fn parse_natural_language_claim_line(line: &str) -> Option<(String, String)> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }

    let lower = trimmed.to_ascii_lowercase();
    for prefix in [
        "i am now in ",
        "i am in ",
        "i'm now in ",
        "i'm in ",
        "location is ",
        "location changed to ",
        "location became ",
    ] {
        if let Some(value) = value_after_case_insensitive_pattern(trimmed, &lower, prefix) {
            return Some(("location".to_owned(), value));
        }
    }

    for prefix in ["status is ", "status changed to ", "status became "] {
        if let Some(value) = value_after_case_insensitive_pattern(trimmed, &lower, prefix) {
            return Some(("status".to_owned(), value));
        }
    }

    None
}

fn value_after_case_insensitive_pattern(
    original: &str,
    lower: &str,
    pattern: &str,
) -> Option<String> {
    let start = lower.find(pattern)? + pattern.len();
    normalize_extracted_claim_value(&original[start..])
}

fn normalize_extracted_claim_value(value: &str) -> Option<String> {
    let value = value
        .trim()
        .trim_matches(|character: char| {
            matches!(
                character,
                '.' | ',' | ';' | ':' | '!' | '?' | '"' | '\'' | ')' | '('
            )
        })
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    if value.is_empty() { None } else { Some(value) }
}

fn is_supported_deterministic_claim_type(claim_type: &str) -> bool {
    matches!(claim_type, "location" | "status")
}
