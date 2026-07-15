use super::*;

pub(super) fn local_language(
    language: &str,
    confidence: f32,
    script: Option<&str>,
) -> LocalLanguageDetection {
    LocalLanguageDetection {
        language: language.to_owned(),
        confidence,
        script: script.map(str::to_owned),
        source: LOCAL_AGENT_SOURCE.to_owned(),
    }
}

pub(super) fn usage_event_id() -> String {
    format!("ai_hub_usage_{}", Uuid::now_v7())
}

pub(super) fn elapsed_ms(started: Instant) -> i64 {
    i64::try_from(started.elapsed().as_millis()).unwrap_or(i64::MAX)
}

pub(super) fn char_count_i32(value: &str) -> i32 {
    usize_to_i32(value.chars().count())
}

pub(super) fn usize_to_i32(value: usize) -> i32 {
    i32::try_from(value).unwrap_or(i32::MAX)
}

pub(super) fn estimate_tokens(chars: i32) -> i32 {
    if chars <= 0 {
        return 0;
    }
    (chars + 3) / 4
}

pub(super) fn u64_to_i64(value: u64) -> Option<i64> {
    i64::try_from(value).ok()
}

pub(super) fn truncate_error(error: &str) -> String {
    const LIMIT: usize = 240;
    let mut value = error.trim().replace(['\n', '\r'], " ");
    if value.chars().count() > LIMIT {
        value = value.chars().take(LIMIT).collect();
    }
    value
}

pub(super) fn detect_pem_blocks(text: &str, findings: &mut Vec<SensitiveFinding>) {
    let upper = text.to_ascii_uppercase();
    if upper.contains("-----BEGIN ")
        && (upper.contains("PRIVATE KEY-----")
            || upper.contains("RSA PRIVATE KEY-----")
            || upper.contains("OPENSSH PRIVATE KEY-----"))
    {
        push_finding(
            findings,
            "private_key_pem",
            SensitivityLevel::Critical,
            0.99,
            "PEM private key marker",
        );
    }
    if upper.contains("-----BEGIN ") && upper.contains("CERTIFICATE-----") {
        push_finding(
            findings,
            "certificate_pem",
            SensitivityLevel::Public,
            0.95,
            "PEM certificate marker",
        );
    }
    if text.contains("ssh-ed25519") || text.contains("ssh-rsa") {
        push_finding(
            findings,
            "ssh_public_key",
            SensitivityLevel::Public,
            0.90,
            "SSH public key marker",
        );
    }
}

pub(super) fn detect_secret_assignments(text: &str, findings: &mut Vec<SensitiveFinding>) {
    for line in text.lines() {
        let Some((label, value)) = explicit_secret_assignment(line) else {
            continue;
        };
        if is_placeholder_secret_value(value) {
            continue;
        }
        push_finding(
            findings,
            "secret_assignment",
            SensitivityLevel::Critical,
            0.85,
            &bounded_evidence(&label),
        );
    }
}

pub(super) fn detect_token_shapes(text: &str, findings: &mut Vec<SensitiveFinding>) {
    for line in text.lines() {
        let has_secret_context = has_secret_context(line);
        for raw_token in line.split(|c: char| c.is_whitespace() || "'\"`<>()[]{};,".contains(c)) {
            let token = raw_token.trim_matches(|c: char| {
                !(c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | '/' | '+' | '='))
            });
            if token.len() < 8 {
                continue;
            }

            if looks_like_github_token(token) {
                push_finding(
                    findings,
                    "github_token",
                    SensitivityLevel::Critical,
                    0.95,
                    &masked_token(token),
                );
                continue;
            }
            if looks_like_aws_access_key(token) {
                push_finding(
                    findings,
                    "aws_access_key_id",
                    SensitivityLevel::Critical,
                    0.95,
                    &masked_token(token),
                );
                continue;
            }
            if looks_like_google_api_key(token) {
                push_finding(
                    findings,
                    "google_api_key",
                    SensitivityLevel::Critical,
                    0.90,
                    &masked_token(token),
                );
                continue;
            }
            if looks_like_jwt(token) {
                push_finding(
                    findings,
                    "jwt",
                    SensitivityLevel::High,
                    0.85,
                    &masked_token(token),
                );
                continue;
            }
            if has_secret_context && looks_like_high_entropy_secret(token) {
                push_finding(
                    findings,
                    "high_entropy_token",
                    SensitivityLevel::High,
                    0.70,
                    &masked_token(token),
                );
            }
        }
    }
}

pub(super) fn has_secret_context(line: &str) -> bool {
    if explicit_secret_assignment(line)
        .is_some_and(|(_, value)| !is_placeholder_secret_value(value))
    {
        return true;
    }
    line.to_ascii_lowercase().contains("authorization: bearer ")
}

pub(super) fn explicit_secret_assignment(line: &str) -> Option<(String, &str)> {
    let (label, value) = line.split_once('=').or_else(|| line.split_once(':'))?;
    if value.trim().is_empty() {
        return None;
    }
    let label = label.trim().strip_prefix("export ").unwrap_or(label.trim());
    if label.bytes().any(|byte| byte.is_ascii_whitespace()) {
        return None;
    }
    let normalized = label
        .chars()
        .filter_map(|character| match character {
            'A'..='Z' => Some(character.to_ascii_lowercase()),
            'a'..='z' | '0'..='9' | '_' => Some(character),
            '-' => Some('_'),
            _ => None,
        })
        .collect::<String>();
    matches!(
        normalized.as_str(),
        "password"
            | "passwd"
            | "secret"
            | "token"
            | "api_key"
            | "apikey"
            | "private_key"
            | "client_secret"
            | "access_token"
    )
    .then_some((normalized, value.trim()))
}

pub(super) fn is_placeholder_secret_value(value: &str) -> bool {
    let value = value
        .trim()
        .trim_matches(|character| matches!(character, '\'' | '"' | '`' | '<' | '>'))
        .to_ascii_lowercase();
    matches!(
        value.as_str(),
        "example"
            | "example-value"
            | "example_value"
            | "changeme"
            | "change-me"
            | "redacted"
            | "placeholder"
            | "replace-me"
            | "replace_me"
            | "your-api-key"
            | "your_api_key"
            | "your-token"
            | "your_token"
    )
}

pub(super) fn detect_financial_identifiers(text: &str, findings: &mut Vec<SensitiveFinding>) {
    let mut card_candidate = String::new();
    let mut separator_before_next_digit = false;
    let mut has_internal_separator = false;
    for ch in text.chars() {
        if ch.is_ascii_digit() {
            if separator_before_next_digit && !card_candidate.is_empty() {
                has_internal_separator = true;
            }
            card_candidate.push(ch);
            separator_before_next_digit = false;
            continue;
        }
        if matches!(ch, ' ' | '-') && !card_candidate.is_empty() {
            separator_before_next_digit = true;
            continue;
        }
        flush_payment_card_candidate(&card_candidate, has_internal_separator, findings);
        card_candidate.clear();
        separator_before_next_digit = false;
        has_internal_separator = false;
    }
    flush_payment_card_candidate(&card_candidate, has_internal_separator, findings);

    for raw_token in text.split_whitespace() {
        let compact: String = raw_token
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .collect::<String>()
            .to_ascii_uppercase();
        if compact.len() >= 15
            && compact.len() <= 34
            && looks_like_iban(&compact)
            && iban_checksum_valid(&compact)
        {
            push_finding(
                findings,
                "iban",
                SensitivityLevel::High,
                0.80,
                &masked_token(&compact),
            );
        }
    }
}

pub(super) fn flush_payment_card_candidate(
    digits: &str,
    has_internal_separator: bool,
    findings: &mut Vec<SensitiveFinding>,
) {
    if has_internal_separator && (13..=19).contains(&digits.len()) && luhn_valid(digits) {
        push_finding(
            findings,
            "payment_card_number",
            SensitivityLevel::High,
            0.85,
            &masked_token(digits),
        );
    }
}

pub(super) fn looks_like_github_token(token: &str) -> bool {
    ["ghp_", "gho_", "ghu_", "ghs_", "ghr_", "github_pat_"]
        .iter()
        .any(|prefix| token.starts_with(prefix))
        && token.len() >= 20
}

pub(super) fn looks_like_aws_access_key(token: &str) -> bool {
    (token.starts_with("AKIA") || token.starts_with("ASIA"))
        && token.len() == 20
        && token
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
}

pub(super) fn looks_like_google_api_key(token: &str) -> bool {
    token.starts_with("AIza") && token.len() >= 24
}

pub(super) fn looks_like_jwt(token: &str) -> bool {
    let parts: Vec<&str> = token.split('.').collect();
    parts.len() == 3
        && parts[0].len() >= 8
        && parts[1].len() >= 8
        && parts.iter().all(|part| {
            part.chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '='))
        })
}

pub(super) fn looks_like_high_entropy_secret(token: &str) -> bool {
    if token.len() < 24 || token.len() > 256 {
        return false;
    }
    let has_lower = token.chars().any(|c| c.is_ascii_lowercase());
    let has_upper = token.chars().any(|c| c.is_ascii_uppercase());
    let has_digit = token.chars().any(|c| c.is_ascii_digit());
    let has_symbol = token
        .chars()
        .any(|c| matches!(c, '_' | '-' | '.' | '/' | '+' | '='));
    let class_count = [has_lower, has_upper, has_digit, has_symbol]
        .into_iter()
        .filter(|value| *value)
        .count();
    class_count >= 3 && shannon_entropy(token) >= 4.0
}

pub(super) fn looks_like_iban(value: &str) -> bool {
    value.len() >= 15
        && value.len() <= 34
        && value.chars().take(2).all(|c| c.is_ascii_uppercase())
        && value.chars().skip(2).take(2).all(|c| c.is_ascii_digit())
        && value.chars().skip(4).all(|c| c.is_ascii_alphanumeric())
}

pub(super) fn iban_checksum_valid(value: &str) -> bool {
    let mut remainder = 0u32;
    for byte in value[4..].bytes().chain(value[..4].bytes()) {
        let numeric = match byte {
            b'0'..=b'9' => u32::from(byte - b'0'),
            b'A'..=b'Z' => u32::from(byte - b'A') + 10,
            _ => return false,
        };
        if numeric >= 10 {
            remainder = (remainder * 10 + numeric / 10) % 97;
            remainder = (remainder * 10 + numeric % 10) % 97;
        } else {
            remainder = (remainder * 10 + numeric) % 97;
        }
    }
    remainder == 1
}

pub(super) fn luhn_valid(digits: &str) -> bool {
    let mut sum = 0u32;
    let mut double = false;
    for ch in digits.chars().rev() {
        let Some(mut digit) = ch.to_digit(10) else {
            return false;
        };
        if double {
            digit *= 2;
            if digit > 9 {
                digit -= 9;
            }
        }
        sum += digit;
        double = !double;
    }
    sum > 0 && sum.is_multiple_of(10)
}

pub(super) fn shannon_entropy(value: &str) -> f32 {
    let bytes = value.as_bytes();
    if bytes.is_empty() {
        return 0.0;
    }
    let mut counts = [0usize; 256];
    for byte in bytes {
        counts[*byte as usize] += 1;
    }
    let len = bytes.len() as f32;
    counts
        .into_iter()
        .filter(|count| *count > 0)
        .map(|count| {
            let p = count as f32 / len;
            -p * p.log2()
        })
        .sum()
}

pub(super) fn sensitivity_for(findings: &[SensitiveFinding]) -> SensitivityLevel {
    findings
        .iter()
        .map(|finding| finding.severity)
        .max()
        .unwrap_or(SensitivityLevel::Public)
}

pub(super) fn contains_any(value: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| value.contains(needle))
}

pub(super) fn push_finding(
    findings: &mut Vec<SensitiveFinding>,
    kind: &str,
    severity: SensitivityLevel,
    confidence: f32,
    evidence: &str,
) {
    if findings
        .iter()
        .any(|finding| finding.kind == kind && finding.evidence == evidence)
    {
        return;
    }
    findings.push(SensitiveFinding {
        kind: kind.to_owned(),
        severity,
        confidence,
        evidence: evidence.to_owned(),
    });
}

pub(super) fn masked_token(token: &str) -> String {
    let chars: Vec<char> = token.chars().collect();
    if chars.len() <= 12 {
        return "<redacted>".to_owned();
    }
    let head: String = chars.iter().take(4).collect();
    let tail: String = chars
        .iter()
        .rev()
        .take(4)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    format!("{head}…{tail}")
}

pub(super) fn bounded_evidence(value: &str) -> String {
    value.chars().take(80).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_pem_private_key_without_exposing_value() {
        let text = [
            "-----BEGIN ",
            "RSA PRIVATE KEY-----\nredacted\n-----END RSA PRIVATE KEY-----",
        ]
        .concat();
        let findings = AiHub::detect_sensitive_content(&text);
        assert!(
            findings
                .iter()
                .any(|finding| finding.kind == "private_key_pem")
        );
    }

    #[test]
    fn detects_credit_card_with_luhn_check() {
        let findings = AiHub::detect_sensitive_content("card 4111 1111 1111 1111");
        assert!(
            findings
                .iter()
                .any(|finding| finding.kind == "payment_card_number")
        );
    }

    #[test]
    fn does_not_classify_an_unlabelled_luhn_tracking_number_as_a_payment_card() {
        let findings = AiHub::detect_sensitive_content("Shipment reference 4111111111111111");

        assert!(
            !findings
                .iter()
                .any(|finding| finding.kind == "payment_card_number")
        );
    }

    #[test]
    fn does_not_classify_an_iban_shaped_tracking_value_without_a_valid_checksum() {
        let findings = AiHub::detect_sensitive_content("Tracking reference DE89370400440532013001");

        assert!(!findings.iter().any(|finding| finding.kind == "iban"));
    }

    #[test]
    fn detects_a_checksum_valid_iban() {
        let findings = AiHub::detect_sensitive_content("IBAN: DE89370400440532013000");

        assert!(findings.iter().any(|finding| finding.kind == "iban"));
    }

    #[test]
    fn does_not_classify_unlabelled_tracking_identifier_as_secret() {
        let findings = AiHub::detect_sensitive_content(
            "https://example.test/click?tracking=Abc9-Def8_Ghi7.Jkl6/Mno5Pqr4",
        );

        assert!(
            !findings
                .iter()
                .any(|finding| finding.kind == "high_entropy_token")
        );
    }

    #[test]
    fn does_not_classify_password_reset_copy_as_a_secret_assignment() {
        let findings = AiHub::detect_sensitive_content(
            "Reset your password: use the link in this email to choose a new one.",
        );

        assert!(
            !findings
                .iter()
                .any(|finding| finding.kind == "secret_assignment")
        );
    }

    #[test]
    fn detects_explicit_secret_assignment_keys() {
        let findings = AiHub::detect_sensitive_content("API_KEY=actual-secret-value-123");

        assert!(
            findings
                .iter()
                .any(|finding| finding.kind == "secret_assignment")
        );
    }

    #[test]
    fn ignores_documented_placeholder_secret_assignments() {
        let inspection = AiHub::inspect_text(
            "Configuration example:\nAPI_KEY=example-value\nTOKEN=<your-token>",
        );

        assert_eq!(inspection.sensitivity, SensitivityLevel::Public);
        assert!(inspection.sensitive_findings.is_empty());
    }

    #[test]
    fn does_not_escalate_public_certificate_or_ssh_key_markers() {
        let inspection = AiHub::inspect_text(
            "-----BEGIN CERTIFICATE-----\npublic certificate\n-----END CERTIFICATE-----\nssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIexample",
        );

        assert_eq!(inspection.sensitivity, SensitivityLevel::Public);
        assert!(
            inspection
                .sensitive_findings
                .iter()
                .all(|finding| finding.severity == SensitivityLevel::Public)
        );
    }

    #[test]
    fn detects_language_locally() {
        let detection = AiHub::detect_language("Привет, как дела?");
        assert_eq!(detection.language, "ru");
    }
}
