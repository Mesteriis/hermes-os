//! Plaintext-to-sealed persistence boundary owned by the workflow runtime.

use hermes_communication_delivery_intent_core::PlannedDeliveryIntentV1;
use hermes_communication_delivery_intent_persistence::{
    CreateDeliveryIntentV1, SealedDeliveryBodyV1,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeliveryIntentCoordinatorErrorV1 {
    InvalidInput,
    SealingUnavailable,
}

pub trait DeliveryIntentBodySealerV1 {
    fn seal_body(
        &mut self,
        logical_owner_id: &str,
        intent_id: [u8; 16],
        body_utf8: &[u8],
    ) -> Result<SealedDeliveryBodyV1, DeliveryIntentCoordinatorErrorV1>;
}

pub fn prepare_create_delivery_intent_v1<S: DeliveryIntentBodySealerV1>(
    logical_owner_id: String,
    planned: PlannedDeliveryIntentV1,
    created_at_unix_seconds: i64,
    sealer: &mut S,
) -> Result<CreateDeliveryIntentV1, DeliveryIntentCoordinatorErrorV1> {
    if !valid_logical_owner_id(&logical_owner_id) || created_at_unix_seconds <= 0 {
        return Err(DeliveryIntentCoordinatorErrorV1::InvalidInput);
    }
    let sealed_body = sealer.seal_body(
        &logical_owner_id,
        planned.intent_id,
        planned.body.as_bytes(),
    )?;
    Ok(CreateDeliveryIntentV1 {
        logical_owner_id,
        intent_id: planned.intent_id,
        canonical_conversation_id: planned.canonical_conversation_id,
        canonical_reply_message_id: planned.canonical_reply_to_message_id,
        route: planned.route,
        sealed_body,
        created_at_unix_seconds,
    })
}

fn valid_logical_owner_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._:-".contains(&byte))
}

#[cfg(test)]
mod tests {
    use hermes_communication_delivery_intent_core::{
        CommunicationConversationIdV1, CommunicationDeliveryRouteV1,
        CommunicationProviderProvenanceV1, CommunicationSourceCursorV1, ValidatedDeliveryBodyV1,
    };

    use super::*;

    struct RecordingSealer {
        observed_body: Vec<u8>,
    }

    impl DeliveryIntentBodySealerV1 for RecordingSealer {
        fn seal_body(
            &mut self,
            _logical_owner_id: &str,
            _intent_id: [u8; 16],
            body_utf8: &[u8],
        ) -> Result<SealedDeliveryBodyV1, DeliveryIntentCoordinatorErrorV1> {
            self.observed_body = body_utf8.to_vec();
            Ok(SealedDeliveryBodyV1 {
                ciphertext: vec![9; 17],
                nonce: [8; 12],
                key_epoch: 7,
                request_fingerprint: [6; 32],
            })
        }
    }

    fn planned() -> PlannedDeliveryIntentV1 {
        PlannedDeliveryIntentV1 {
            intent_id: [1; 16],
            canonical_conversation_id: CommunicationConversationIdV1::new([2; 16]),
            canonical_reply_to_message_id: None,
            route: CommunicationDeliveryRouteV1 {
                provider: CommunicationProviderProvenanceV1::Telegram,
                account_cursor: CommunicationSourceCursorV1::new([3; 32]),
                conversation_cursor: CommunicationSourceCursorV1::new([4; 32]),
                reply_to_source_cursor: None,
            },
            body: ValidatedDeliveryBodyV1::try_from(b"private body".to_vec()).expect("body"),
        }
    }

    #[test]
    fn plaintext_is_consumed_by_sealer_and_not_returned_to_persistence() {
        let mut sealer = RecordingSealer {
            observed_body: Vec::new(),
        };
        let command =
            prepare_create_delivery_intent_v1("owner:test".to_owned(), planned(), 10, &mut sealer)
                .expect("sealed command");
        assert_eq!(sealer.observed_body, b"private body");
        assert_eq!(command.sealed_body.ciphertext, vec![9; 17]);
        assert_eq!(command.intent_id, [1; 16]);
    }

    #[test]
    fn invalid_owner_or_time_never_reaches_sealer() {
        let mut sealer = RecordingSealer {
            observed_body: Vec::new(),
        };
        assert!(matches!(
            prepare_create_delivery_intent_v1(
                "owner invalid".to_owned(),
                planned(),
                10,
                &mut sealer
            ),
            Err(DeliveryIntentCoordinatorErrorV1::InvalidInput)
        ));
        assert!(sealer.observed_body.is_empty());
    }
}
