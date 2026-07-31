use prost::Message;

use crate::{COMMUNICATION_REPLY_SOURCE_MAX_BYTES_V1, wire::CommunicationReplySourceContentV1};

const MAX_SENDER_BYTES_V1: usize = 256;
const MAX_SUBJECT_BYTES_V1: usize = 998;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationReplySourceContentErrorV1 {
    Invalid,
    Limit,
}

pub fn encode_communication_reply_source_content_v1(
    content: &CommunicationReplySourceContentV1,
) -> Result<Vec<u8>, CommunicationReplySourceContentErrorV1> {
    validate_communication_reply_source_content_v1(content)?;
    Ok(content.encode_to_vec())
}

pub fn decode_communication_reply_source_content_v1(
    exact_bytes: &[u8],
) -> Result<CommunicationReplySourceContentV1, CommunicationReplySourceContentErrorV1> {
    let content = CommunicationReplySourceContentV1::decode(exact_bytes)
        .map_err(|_| CommunicationReplySourceContentErrorV1::Invalid)?;
    validate_communication_reply_source_content_v1(&content)?;
    if content.encode_to_vec() != exact_bytes {
        return Err(CommunicationReplySourceContentErrorV1::Invalid);
    }
    Ok(content)
}

pub fn validate_communication_reply_source_content_v1(
    content: &CommunicationReplySourceContentV1,
) -> Result<(), CommunicationReplySourceContentErrorV1> {
    if content.sender_utf8.len() > MAX_SENDER_BYTES_V1
        || content.subject_utf8.len() > MAX_SUBJECT_BYTES_V1
        || content.body_utf8.is_empty()
        || std::str::from_utf8(&content.sender_utf8).is_err()
        || std::str::from_utf8(&content.subject_utf8).is_err()
        || std::str::from_utf8(&content.body_utf8).is_err()
        || has_control(&content.sender_utf8)
        || has_control(&content.subject_utf8)
    {
        return Err(CommunicationReplySourceContentErrorV1::Invalid);
    }
    let encoded_len = content.encoded_len();
    if encoded_len == 0
        || encoded_len > usize::try_from(COMMUNICATION_REPLY_SOURCE_MAX_BYTES_V1).unwrap_or(0)
    {
        return Err(CommunicationReplySourceContentErrorV1::Limit);
    }
    Ok(())
}

fn has_control(value: &[u8]) -> bool {
    std::str::from_utf8(value).is_ok_and(|value| value.chars().any(char::is_control))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_round_trip_preserves_sender_subject_and_body() {
        let content = CommunicationReplySourceContentV1 {
            sender_utf8: b"Ada <ada@example.test>".to_vec(),
            subject_utf8: b"Quarterly update".to_vec(),
            body_utf8: b"Please review the attached status.".to_vec(),
        };
        let encoded = encode_communication_reply_source_content_v1(&content).expect("encode");
        assert_eq!(
            decode_communication_reply_source_content_v1(&encoded),
            Ok(content)
        );
    }

    #[test]
    fn rejects_invalid_utf8_control_metadata_and_oversized_content() {
        let invalid_sender = CommunicationReplySourceContentV1 {
            sender_utf8: vec![0xff],
            subject_utf8: Vec::new(),
            body_utf8: b"body".to_vec(),
        };
        assert_eq!(
            validate_communication_reply_source_content_v1(&invalid_sender),
            Err(CommunicationReplySourceContentErrorV1::Invalid)
        );
        let control_subject = CommunicationReplySourceContentV1 {
            sender_utf8: Vec::new(),
            subject_utf8: b"subject\ninjection".to_vec(),
            body_utf8: b"body".to_vec(),
        };
        assert_eq!(
            validate_communication_reply_source_content_v1(&control_subject),
            Err(CommunicationReplySourceContentErrorV1::Invalid)
        );
        let oversized = CommunicationReplySourceContentV1 {
            sender_utf8: Vec::new(),
            subject_utf8: Vec::new(),
            body_utf8: vec![
                b'x';
                usize::try_from(COMMUNICATION_REPLY_SOURCE_MAX_BYTES_V1)
                    .expect("bounded")
                    + 1
            ],
        };
        assert_eq!(
            validate_communication_reply_source_content_v1(&oversized),
            Err(CommunicationReplySourceContentErrorV1::Limit)
        );
    }
}
