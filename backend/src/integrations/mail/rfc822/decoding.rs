use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use encoding_rs::{Encoding, UTF_8};

pub(crate) fn decode_header_value_bytes(value: &[u8]) -> String {
    decode_rfc2047_words(decode_text_bytes(value, None).trim())
}

pub(crate) fn decode_transfer_body(
    body: &[u8],
    transfer_encoding: &str,
    charset: Option<&str>,
) -> String {
    decode_text_bytes(&decode_transfer_bytes(body, transfer_encoding), charset)
}

pub(crate) fn decode_transfer_bytes(body: &[u8], transfer_encoding: &str) -> Vec<u8> {
    match transfer_encoding.trim().to_ascii_lowercase().as_str() {
        "base64" => {
            let compact = body
                .iter()
                .copied()
                .filter(|byte| !byte.is_ascii_whitespace())
                .collect::<Vec<_>>();
            BASE64_STANDARD
                .decode(compact)
                .unwrap_or_else(|_| body.to_vec())
        }
        "quoted-printable" => decode_quoted_printable_bytes(body),
        _ => body.to_vec(),
    }
}

fn decode_quoted_printable(input: &str, charset: Option<&str>) -> String {
    decode_text_bytes(&decode_quoted_printable_bytes(input.as_bytes()), charset)
}

fn decode_quoted_printable_bytes(input: &[u8]) -> Vec<u8> {
    let mut output = Vec::with_capacity(input.len());
    let mut index = 0;

    while index < input.len() {
        if input[index] == b'=' {
            if input.get(index + 1) == Some(&b'\r') && input.get(index + 2) == Some(&b'\n') {
                index += 3;
                continue;
            }
            if input.get(index + 1) == Some(&b'\n') {
                index += 2;
                continue;
            }
            if let (Some(high), Some(low)) = (input.get(index + 1), input.get(index + 2))
                && let (Some(high), Some(low)) = (hex_value(*high), hex_value(*low))
            {
                output.push((high << 4) | low);
                index += 3;
                continue;
            }
        }
        output.push(input[index]);
        index += 1;
    }

    output
}

pub(crate) fn decode_text_bytes(bytes: &[u8], charset: Option<&str>) -> String {
    if bytes.is_empty() {
        return String::new();
    }

    let primary_encoding = charset
        .and_then(|label| Encoding::for_label(label.trim().as_bytes()))
        .unwrap_or(UTF_8);
    let primary = decode_with_encoding(bytes, primary_encoding);
    if charset.is_some() && !primary.had_errors {
        return primary.text;
    }
    if charset.is_none() && !primary.had_errors {
        return primary.text;
    }

    legacy_text_candidates(bytes, primary)
        .into_iter()
        .max_by_key(score_decoded_text)
        .map(|candidate| candidate.text)
        .unwrap_or_else(|| String::from_utf8_lossy(bytes).into_owned())
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DecodedTextCandidate {
    text: String,
    had_errors: bool,
    fallback_rank: i64,
}

fn decode_with_encoding(bytes: &[u8], encoding: &'static Encoding) -> DecodedTextCandidate {
    let (text, _, had_errors) = encoding.decode(bytes);
    DecodedTextCandidate {
        text: text.into_owned(),
        had_errors,
        fallback_rank: 0,
    }
}

fn legacy_text_candidates(
    bytes: &[u8],
    primary: DecodedTextCandidate,
) -> Vec<DecodedTextCandidate> {
    let mut candidates = vec![primary];
    for (fallback_rank, label) in [
        "windows-1251",
        "koi8-r",
        "iso-8859-5",
        "windows-1252",
        "iso-8859-1",
    ]
    .iter()
    .enumerate()
    {
        let Some(encoding) = Encoding::for_label(label.as_bytes()) else {
            continue;
        };
        let mut candidate = decode_with_encoding(bytes, encoding);
        candidate.fallback_rank = fallback_rank as i64 + 1;
        candidates.push(candidate);
    }
    candidates
}

fn score_decoded_text(candidate: &DecodedTextCandidate) -> i64 {
    let mut replacement_count = 0;
    let mut disallowed_control_count = 0;
    let mut cyrillic_count = 0;
    let mut printable_count = 0;

    for character in candidate.text.chars() {
        if character == '\u{fffd}' {
            replacement_count += 1;
        } else if character.is_control()
            && character != '\n'
            && character != '\r'
            && character != '\t'
        {
            disallowed_control_count += 1;
        } else if ('\u{0400}'..='\u{04ff}').contains(&character) {
            cyrillic_count += 1;
            printable_count += 1;
        } else if !character.is_control() {
            printable_count += 1;
        }
    }

    printable_count + (cyrillic_count * 8)
        - (replacement_count * 1_000)
        - (disallowed_control_count * 100)
        - candidate.fallback_rank
}

pub(crate) fn percent_decode_bytes(value: &str) -> Vec<u8> {
    let bytes = value.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] == b'%'
            && let (Some(high), Some(low)) = (bytes.get(index + 1), bytes.get(index + 2))
            && let (Some(high), Some(low)) = (hex_value(*high), hex_value(*low))
        {
            output.push((high << 4) | low);
            index += 3;
            continue;
        }
        output.push(bytes[index]);
        index += 1;
    }

    output
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

pub(crate) fn decode_rfc2047_words(input: &str) -> String {
    let mut output = String::new();
    let mut rest = input;

    while let Some(start) = rest.find("=?") {
        output.push_str(&rest[..start]);
        let candidate = &rest[start + 2..];
        let Some(charset_end) = candidate.find('?') else {
            output.push_str(&rest[start..]);
            return output;
        };
        let charset = &candidate[..charset_end];
        let candidate = &candidate[charset_end + 1..];
        let Some(encoding_end) = candidate.find('?') else {
            output.push_str(&rest[start..]);
            return output;
        };
        let encoding = &candidate[..encoding_end];
        let candidate = &candidate[encoding_end + 1..];
        let Some(encoded_end) = candidate.find("?=") else {
            output.push_str(&rest[start..]);
            return output;
        };
        let encoded = &candidate[..encoded_end];
        let decoded = match encoding.to_ascii_lowercase().as_str() {
            "b" => BASE64_STANDARD
                .decode(encoded)
                .map(|bytes| decode_text_bytes(&bytes, Some(charset)))
                .ok(),
            "q" => Some(decode_quoted_printable(
                &encoded.replace('_', " "),
                Some(charset),
            )),
            _ => None,
        };

        if let Some(decoded) = decoded {
            output.push_str(&decoded);
        } else {
            output.push_str(
                &rest[start..start + 2 + charset_end + 1 + encoding_end + 1 + encoded_end + 2],
            );
        }
        rest = &candidate[encoded_end + 2..];
    }

    output.push_str(rest);
    output
}
