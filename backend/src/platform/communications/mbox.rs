#[derive(Debug, thiserror::Error, Eq, PartialEq)]
pub enum MboxParseError {
    #[error("MBOX source is empty")]
    Empty,
    #[error("MBOX source is missing a From_ separator")]
    MissingSeparator,
    #[error("MBOX contains an empty message")]
    EmptyMessage,
    #[error("MBOX exceeds the configured message limit")]
    MessageLimitExceeded,
}

/// Splits a standard mbox stream into its RFC822 message payloads.
///
/// The `From_` envelope line is deliberately not passed to the RFC822 parser.
/// Escaped `>From ` body lines remain byte-for-byte intact as source evidence.
pub fn split_mbox_messages(
    source: &[u8],
    max_messages: usize,
) -> Result<Vec<Vec<u8>>, MboxParseError> {
    if source.is_empty() || max_messages == 0 {
        return Err(MboxParseError::Empty);
    }

    let mut messages = Vec::new();
    let mut current = Vec::new();
    let mut saw_separator = false;

    for line in source.split_inclusive(|byte| *byte == b'\n') {
        if is_from_separator(line) {
            if saw_separator {
                push_message(&mut messages, &mut current, max_messages)?;
            }
            saw_separator = true;
            continue;
        }
        if saw_separator {
            current.extend_from_slice(line);
        }
    }

    if !saw_separator {
        return Err(MboxParseError::MissingSeparator);
    }
    push_message(&mut messages, &mut current, max_messages)?;
    Ok(messages)
}

fn is_from_separator(line: &[u8]) -> bool {
    line.strip_suffix(b"\n")
        .and_then(|line| line.strip_suffix(b"\r").or(Some(line)))
        .unwrap_or(line)
        .starts_with(b"From ")
}

fn push_message(
    messages: &mut Vec<Vec<u8>>,
    current: &mut Vec<u8>,
    max_messages: usize,
) -> Result<(), MboxParseError> {
    if current.iter().all(u8::is_ascii_whitespace) {
        return Err(MboxParseError::EmptyMessage);
    }
    if messages.len() >= max_messages {
        return Err(MboxParseError::MessageLimitExceeded);
    }
    messages.push(std::mem::take(current));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_crlf_mbox_and_keeps_escaped_body_lines() {
        let source = concat!(
            "From sender@example.com Fri Jul 11 12:00:00 2026\r\n",
            "Subject: First\r\n\r\n",
            ">From remains body text\r\n",
            "From sender@example.com Fri Jul 11 12:01:00 2026\r\n",
            "Subject: Second\r\n\r\nBody\r\n"
        );

        let messages = split_mbox_messages(source.as_bytes(), 10).expect("split mbox");
        assert_eq!(messages.len(), 2);
        assert!(messages[0].starts_with(b"Subject: First"));
        assert!(messages[0].windows(6).any(|line| line == b">From "));
        assert!(messages[1].starts_with(b"Subject: Second"));
    }

    #[test]
    fn rejects_missing_separator_empty_messages_and_limit_overflow() {
        assert_eq!(
            split_mbox_messages(b"Subject: no envelope\n\nbody", 10),
            Err(MboxParseError::MissingSeparator)
        );
        assert_eq!(
            split_mbox_messages(b"From sender@example.com\n", 10),
            Err(MboxParseError::EmptyMessage)
        );
        assert_eq!(
            split_mbox_messages(b"From sender@example.com\nSubject: one\n\nbody\n", 0),
            Err(MboxParseError::Empty)
        );
        assert_eq!(
            split_mbox_messages(
                concat!(
                    "From sender@example.com\nSubject: one\n\nbody\n",
                    "From sender@example.com\nSubject: two\n\nbody\n",
                )
                .as_bytes(),
                1,
            ),
            Err(MboxParseError::MessageLimitExceeded)
        );
    }
}
