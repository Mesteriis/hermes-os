use hermes_events_protocol::{
    delivery::{OutboxRecordError, OutboxRecordV1},
    v1::{
        ActorKindV1, ActorRefV1, CommandMetadataV1, ContractRefV1, DurableEnvelopeV1, FenceKindV1,
        SourceFenceV1, SourceRefV1, durable_envelope_v1::Semantics,
    },
    validation::envelope::validate_envelope_v1,
};
use prost::Message;
use prost_types::Timestamp;
use sha2::{Digest, Sha256};

use crate::{
    MAIL_ADDRESS_BOOK_CAPABILITY_ID_V1, MAIL_ADDRESS_BOOK_CONTRACT_MAJOR_V1,
    MAIL_ADDRESS_BOOK_CONTRACT_REVISION_V1, MAIL_ADDRESS_BOOK_MAX_CURSOR_BYTES_V1,
    MAIL_ADDRESS_BOOK_MAX_PAGE_SIZE_V1, MAIL_ADDRESS_BOOK_SCHEMA_SHA256_V1, MAIL_OWNER_ID_V1,
    MailAddressBookContractV1, wire::FetchMailAddressBookPageCommandV1,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailAddressBookEnvelopeContextV1 {
    pub module_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub recorded_at_unix_seconds: i64,
    pub recorded_at_nanos: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailAddressBookEnvelopeBuildErrorV1 {
    InvalidContext,
    InvalidPayload,
    InvalidEnvelope,
    OutboxRejected,
}

pub fn build_fetch_mail_address_book_page_command_v1(
    payload: FetchMailAddressBookPageCommandV1,
    deadline_unix_seconds: i64,
    context: &MailAddressBookEnvelopeContextV1,
) -> Result<OutboxRecordV1, MailAddressBookEnvelopeBuildErrorV1> {
    validate_context(context)?;
    let command_id = id16(&payload.command_id)?;
    let run_id = id16(&payload.run_id)?;
    if !valid_identity(&payload.logical_owner_id)
        || !valid_bounded(&payload.account_id, 256)
        || payload.page_size == 0
        || payload.page_size > MAIL_ADDRESS_BOOK_MAX_PAGE_SIZE_V1
        || payload.continuation_cursor.as_ref().is_some_and(|value| {
            value.is_empty() || value.len() > MAIL_ADDRESS_BOOK_MAX_CURSOR_BYTES_V1
        })
        || deadline_unix_seconds <= context.recorded_at_unix_seconds
    {
        return Err(MailAddressBookEnvelopeBuildErrorV1::InvalidPayload);
    }
    let contract = MailAddressBookContractV1::FetchPageCommand;
    let envelope = DurableEnvelopeV1 {
        envelope_major: 1,
        envelope_revision: 1,
        message_id: command_id.to_vec(),
        contract: Some(ContractRefV1 {
            owner: MAIL_OWNER_ID_V1.to_owned(),
            name: contract.name().to_owned(),
            major: MAIL_ADDRESS_BOOK_CONTRACT_MAJOR_V1,
            revision: MAIL_ADDRESS_BOOK_CONTRACT_REVISION_V1,
            schema_sha256: MAIL_ADDRESS_BOOK_SCHEMA_SHA256_V1.to_vec(),
        }),
        source: Some(SourceRefV1 {
            module_id: context.module_id.clone(),
            runtime_instance_id: digest16(
                b"mail-contacts-sync-runtime-instance-v1",
                context.runtime_instance_id.as_bytes(),
                b"mail-address-book",
            )
            .to_vec(),
            runtime_generation: context.runtime_generation,
        }),
        recorded_at: Some(timestamp(context)),
        partition_key: run_id.to_vec(),
        causation_message_id: Vec::new(),
        correlation_id: run_id.to_vec(),
        actor: Some(ActorRefV1 {
            kind: ActorKindV1::Module as i32,
            actor_id: context.module_id.as_bytes().to_vec(),
        }),
        trace: None,
        source_fence: Some(SourceFenceV1 {
            kind: FenceKindV1::RuntimeLease as i32,
            scope_id: context.module_id.as_bytes().to_vec(),
            epoch: context.runtime_generation,
        }),
        semantics: Some(Semantics::Command(CommandMetadataV1 {
            command_id: command_id.to_vec(),
            target_capability: MAIL_ADDRESS_BOOK_CAPABILITY_ID_V1.to_owned(),
            idempotency_key: digest16(
                b"mail-address-book-fetch-page-idempotency-v1",
                &run_id,
                &payload.page_size.to_be_bytes(),
            )
            .to_vec(),
            deadline: Some(Timestamp {
                seconds: deadline_unix_seconds,
                nanos: 0,
            }),
            logical_attempt: 1,
        })),
        payload: payload.encode_to_vec(),
    };
    validate_envelope_v1(&envelope)
        .map_err(|_| MailAddressBookEnvelopeBuildErrorV1::InvalidEnvelope)?;
    OutboxRecordV1::accept(envelope.encode_to_vec()).map_err(outbox_error)
}

fn validate_context(
    context: &MailAddressBookEnvelopeContextV1,
) -> Result<(), MailAddressBookEnvelopeBuildErrorV1> {
    if !valid_bounded(&context.module_id, 128)
        || !valid_bounded(&context.runtime_instance_id, 128)
        || context.runtime_generation == 0
        || context.recorded_at_unix_seconds <= 0
        || !(0..1_000_000_000).contains(&context.recorded_at_nanos)
    {
        return Err(MailAddressBookEnvelopeBuildErrorV1::InvalidContext);
    }
    Ok(())
}

fn timestamp(context: &MailAddressBookEnvelopeContextV1) -> Timestamp {
    Timestamp {
        seconds: context.recorded_at_unix_seconds,
        nanos: context.recorded_at_nanos,
    }
}

fn id16(value: &[u8]) -> Result<[u8; 16], MailAddressBookEnvelopeBuildErrorV1> {
    let value: [u8; 16] = value
        .try_into()
        .map_err(|_| MailAddressBookEnvelopeBuildErrorV1::InvalidPayload)?;
    if value.iter().all(|byte| *byte == 0) {
        return Err(MailAddressBookEnvelopeBuildErrorV1::InvalidPayload);
    }
    Ok(value)
}

fn valid_identity(value: &str) -> bool {
    valid_bounded(value, 128)
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
}

fn valid_bounded(value: &str, max: usize) -> bool {
    !value.is_empty() && value.len() <= max && value.is_ascii() && value.trim() == value
}

fn digest16(label: &[u8], left: &[u8], right: &[u8]) -> [u8; 16] {
    let mut digest = Sha256::new();
    digest.update(label);
    digest.update([0]);
    digest.update(left);
    digest.update([0]);
    digest.update(right);
    digest.finalize()[..16]
        .try_into()
        .expect("SHA-256 prefix has exact length")
}

const fn outbox_error(_: OutboxRecordError) -> MailAddressBookEnvelopeBuildErrorV1 {
    MailAddressBookEnvelopeBuildErrorV1::OutboxRejected
}

#[cfg(test)]
mod tests {
    use hermes_events_protocol::validation::envelope::decode_envelope_v1;

    use super::*;

    #[test]
    fn fetch_command_is_mail_targeted_and_provider_neutral() {
        let record = build_fetch_mail_address_book_page_command_v1(
            FetchMailAddressBookPageCommandV1 {
                command_id: vec![1; 16],
                run_id: vec![2; 16],
                logical_owner_id: "owner-1".to_owned(),
                account_id: "mail-account-1".to_owned(),
                continuation_cursor: None,
                page_size: 100,
            },
            1_800_000_030,
            &MailAddressBookEnvelopeContextV1 {
                module_id: "hermes-mail-contacts-sync-runtime".to_owned(),
                runtime_instance_id: "runtime-1".to_owned(),
                runtime_generation: 1,
                recorded_at_unix_seconds: 1_800_000_000,
                recorded_at_nanos: 0,
            },
        )
        .expect("command");
        let envelope = decode_envelope_v1(record.exact_bytes()).expect("envelope");
        assert_eq!(envelope.contract.expect("contract").owner, MAIL_OWNER_ID_V1);
        assert_eq!(envelope.partition_key, vec![2; 16]);
    }
}
