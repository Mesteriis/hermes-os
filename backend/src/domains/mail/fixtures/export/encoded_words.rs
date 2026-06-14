use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;

use super::encoding::decode_quoted_printable;

pub(super) fn decode_rfc2047_words(input: &str) -> String {
    let mut output = String::new();
    let mut rest = input;

    while let Some(start) = rest.find("=?") {
        output.push_str(&rest[..start]);
        let candidate = &rest[start + 2..];
        let Some(charset_end) = candidate.find('?') else {
            output.push_str(&rest[start..]);
            return output;
        };
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
        append_decoded_word(
            &mut output,
            rest,
            charset_end,
            encoding_end,
            encoded_end,
            encoding,
        );
        rest = &candidate[encoded_end + 2..];
    }

    output.push_str(rest);
    output
}

fn append_decoded_word(
    output: &mut String,
    rest: &str,
    charset_end: usize,
    encoding_end: usize,
    encoded_end: usize,
    encoding: &str,
) {
    let candidate = &rest[2 + charset_end + 1 + encoding_end + 1..];
    let encoded = &candidate[..encoded_end];
    let decoded = match encoding.to_ascii_lowercase().as_str() {
        "b" => BASE64_STANDARD
            .decode(encoded)
            .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
            .ok(),
        "q" => Some(decode_quoted_printable(&encoded.replace('_', " "))),
        _ => None,
    };

    if let Some(decoded) = decoded {
        output.push_str(&decoded);
    } else {
        output.push_str(&rest[..2 + charset_end + 1 + encoding_end + 1 + encoded_end + 2]);
    }
}
