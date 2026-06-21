use super::decoding::{
    decode_header_value_bytes, decode_rfc2047_words, decode_text_bytes, percent_decode_bytes,
};
use super::wire::{strip_trailing_cr, trim_ascii_whitespace};

pub(crate) fn parse_headers(header_block: &[u8]) -> Vec<(String, String)> {
    let mut raw_headers: Vec<(String, Vec<u8>)> = Vec::new();

    for line in header_block.split(|byte| *byte == b'\n') {
        let line = strip_trailing_cr(line);
        if line.starts_with(b" ") || line.starts_with(b"\t") {
            if let Some((_, value)) = raw_headers.last_mut() {
                value.push(b' ');
                value.extend_from_slice(trim_ascii_whitespace(line));
            }
            continue;
        }

        if let Some(separator_index) = line.iter().position(|byte| *byte == b':') {
            let name = decode_ascii_header_name(trim_ascii_whitespace(&line[..separator_index]));
            let value = trim_ascii_whitespace(&line[separator_index + 1..]).to_vec();
            headers_push_if_valid(&mut raw_headers, name, value);
        }
    }

    raw_headers
        .into_iter()
        .map(|(name, value)| (name, decode_header_value_bytes(&value)))
        .collect()
}

fn headers_push_if_valid(headers: &mut Vec<(String, Vec<u8>)>, name: String, value: Vec<u8>) {
    if !name.is_empty() {
        headers.push((name, value));
    }
}

fn decode_ascii_header_name(value: &[u8]) -> String {
    String::from_utf8_lossy(value).trim().to_owned()
}

pub(crate) fn header_value(headers: &[(String, String)], name: &str) -> Option<String> {
    headers
        .iter()
        .find(|(header_name, _)| header_name.eq_ignore_ascii_case(name))
        .map(|(_, value)| decode_rfc2047_words(value.trim()))
}

pub(crate) fn header_media_type(value: &str) -> String {
    value
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
}

pub(crate) fn header_parameter(value: &str, parameter: &str) -> Option<String> {
    let mut continuation_segments = Vec::new();
    let mut plain_parameter = None;
    let mut encoded_parameter = None;

    for part in value.split(';').skip(1) {
        let Some((name, value)) = part.split_once('=') else {
            continue;
        };
        let name = name.trim();
        if name.eq_ignore_ascii_case(parameter) {
            plain_parameter = Some(unquote_header_parameter_value(value));
            continue;
        }
        if let Some(segment) = rfc2231_continuation_segment(name, parameter, value) {
            continuation_segments.push(segment);
            continue;
        }
        if let Some(base_name) = name.strip_suffix('*')
            && base_name.eq_ignore_ascii_case(parameter)
        {
            encoded_parameter = Some(decode_rfc2231_parameter_value(value));
        }
    }

    encoded_parameter
        .or_else(|| decode_rfc2231_continuation_segments(continuation_segments))
        .or(plain_parameter)
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Rfc2231ContinuationSegment {
    index: usize,
    encoded: bool,
    value: String,
}

fn rfc2231_continuation_segment(
    name: &str,
    parameter: &str,
    value: &str,
) -> Option<Rfc2231ContinuationSegment> {
    if name.len() <= parameter.len() || !name[..parameter.len()].eq_ignore_ascii_case(parameter) {
        return None;
    }
    let rest = &name[parameter.len()..];
    let rest = rest.strip_prefix('*')?;
    let encoded = rest.ends_with('*');
    let index = rest.trim_end_matches('*').parse::<usize>().ok()?;

    Some(Rfc2231ContinuationSegment {
        index,
        encoded,
        value: unquote_header_parameter_value(value),
    })
}

fn decode_rfc2231_continuation_segments(
    mut segments: Vec<Rfc2231ContinuationSegment>,
) -> Option<String> {
    if segments.is_empty() {
        return None;
    }

    segments.sort_by_key(|segment| segment.index);
    if segments.first().map(|segment| segment.index) != Some(0) {
        return None;
    }

    let mut output = Vec::new();
    let mut charset = None;
    for (expected_index, segment) in segments.into_iter().enumerate() {
        if segment.index != expected_index {
            return None;
        }

        let value = if expected_index == 0 {
            let (segment_charset, payload) = rfc2231_charset_and_payload(&segment.value);
            charset = segment_charset.map(str::to_owned);
            payload
        } else {
            segment.value.as_str()
        };
        if segment.encoded {
            output.extend(percent_decode_bytes(value));
        } else {
            output.extend_from_slice(value.as_bytes());
        }
    }

    Some(decode_text_bytes(&output, charset.as_deref()))
}

fn rfc2231_charset_and_payload(value: &str) -> (Option<&str>, &str) {
    let Some((charset, rest)) = value.split_once('\'') else {
        return (None, value);
    };
    let Some((_, encoded)) = rest.split_once('\'') else {
        return (None, value);
    };
    let charset = charset.trim();
    let charset = if charset.is_empty() {
        None
    } else {
        Some(charset)
    };
    (charset, encoded)
}

fn unquote_header_parameter_value(value: &str) -> String {
    let value = value.trim();
    let value = value
        .strip_prefix('"')
        .and_then(|stripped| stripped.strip_suffix('"'))
        .unwrap_or(value);
    let mut output = String::with_capacity(value.len());
    let mut escaped = false;

    for character in value.chars() {
        if escaped {
            output.push(character);
            escaped = false;
            continue;
        }
        if character == '\\' {
            escaped = true;
            continue;
        }
        output.push(character);
    }

    if escaped {
        output.push('\\');
    }

    output
}

fn decode_rfc2231_parameter_value(value: &str) -> String {
    let value = unquote_header_parameter_value(value);
    let (charset, encoded_value) = rfc2231_charset_and_payload(&value);

    decode_text_bytes(&percent_decode_bytes(encoded_value), charset)
}
