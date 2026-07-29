use hermes_mail_api::MailSendMailRequestV1;
use hermes_mail_persistence::MailDeliveryIntentJobV1;
use sha2::{Digest, Sha256};

use crate::managed::{MailAdmittedRuntime, MailBootstrapError};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailDeliveryIntentExecutionErrorV1 {
    InvalidJob,
    InvalidBody,
    QueueUnavailable,
}

pub fn materialize_mail_delivery_intent_v1(
    job: &MailDeliveryIntentJobV1,
    body: &[u8],
) -> Result<MailSendMailRequestV1, MailDeliveryIntentExecutionErrorV1> {
    let body_len =
        u64::try_from(body.len()).map_err(|_| MailDeliveryIntentExecutionErrorV1::InvalidBody)?;
    if job.intent_id.iter().all(|byte| *byte == 0)
        || job.command_message_id.iter().all(|byte| *byte == 0)
        || job.connection_id.trim().is_empty()
        || job.provider_thread_id.trim().is_empty()
        || job.recipient.trim().is_empty()
        || job.subject.trim().is_empty()
        || job.body_reference_id.iter().all(|byte| *byte == 0)
        || job.provider_operation_id.trim().is_empty()
    {
        return Err(MailDeliveryIntentExecutionErrorV1::InvalidJob);
    }
    if body_len != job.body_declared_bytes || Sha256::digest(body).as_slice() != job.body_sha256 {
        return Err(MailDeliveryIntentExecutionErrorV1::InvalidBody);
    }
    let text_body = std::str::from_utf8(body)
        .map_err(|_| MailDeliveryIntentExecutionErrorV1::InvalidBody)?
        .to_owned();
    Ok(MailSendMailRequestV1 {
        operation_id: job.provider_operation_id.clone(),
        connection_id: job.connection_id.clone(),
        provider_conversation_id: job.provider_thread_id.clone(),
        recipients: vec![job.recipient.clone()],
        cc_recipients: Vec::new(),
        bcc_recipients: Vec::new(),
        subject: job.subject.clone(),
        text_body,
        attachment_anchor_ids: Vec::new(),
    })
}

pub async fn enqueue_mail_delivery_intent_v1(
    runtime: &mut MailAdmittedRuntime,
    job: &MailDeliveryIntentJobV1,
    body: &[u8],
    requested_at_unix_seconds: i64,
) -> Result<(), MailDeliveryIntentExecutionErrorV1> {
    if requested_at_unix_seconds <= 0 {
        return Err(MailDeliveryIntentExecutionErrorV1::InvalidJob);
    }
    let request = materialize_mail_delivery_intent_v1(job, body)?;
    runtime
        .select_account(&job.connection_id)
        .map_err(map_queue_error)?;
    let operation_id = runtime
        .submit_delivery(&request, requested_at_unix_seconds)
        .await
        .map_err(map_queue_error)?;
    if operation_id != job.provider_operation_id {
        return Err(MailDeliveryIntentExecutionErrorV1::QueueUnavailable);
    }
    Ok(())
}

fn map_queue_error(_: MailBootstrapError) -> MailDeliveryIntentExecutionErrorV1 {
    MailDeliveryIntentExecutionErrorV1::QueueUnavailable
}

#[cfg(test)]
mod tests {
    use super::*;

    fn job(body: &[u8]) -> MailDeliveryIntentJobV1 {
        MailDeliveryIntentJobV1 {
            intent_id: [1; 16],
            command_message_id: [2; 16],
            connection_id: "mail-account".to_owned(),
            provider_thread_id: "provider-thread".to_owned(),
            reply_to_provider_message_id: Some("provider-message".to_owned()),
            recipient: "recipient@example.com".to_owned(),
            subject: "Re: Subject".to_owned(),
            body_reference_id: [3; 16],
            body_declared_bytes: u64::try_from(body.len()).expect("test body length"),
            body_sha256: Sha256::digest(body).into(),
            custody_transfer_source_proof: vec![4; 32],
            provider_operation_id: "delivery-intent-01010101010101010101010101010101".to_owned(),
        }
    }

    #[test]
    fn materializes_exact_provider_route_into_existing_mail_queue_contract() {
        let body = b"Reply body";
        let request = materialize_mail_delivery_intent_v1(&job(body), body).expect("request");

        assert_eq!(request.connection_id, "mail-account");
        assert_eq!(request.provider_conversation_id, "provider-thread");
        assert_eq!(request.recipients, ["recipient@example.com"]);
        assert_eq!(request.subject, "Re: Subject");
        assert_eq!(request.text_body, "Reply body");
        assert!(request.cc_recipients.is_empty());
        assert!(request.bcc_recipients.is_empty());
        assert!(request.attachment_anchor_ids.is_empty());
    }

    #[test]
    fn rejects_body_bytes_that_do_not_match_the_admitted_receipt() {
        let admitted = job(b"expected");

        assert_eq!(
            materialize_mail_delivery_intent_v1(&admitted, b"different"),
            Err(MailDeliveryIntentExecutionErrorV1::InvalidBody),
        );
    }

    #[test]
    fn rejects_non_utf8_body_instead_of_reinterpreting_provider_content() {
        let body = [0xff, 0xfe];

        assert_eq!(
            materialize_mail_delivery_intent_v1(&job(&body), &body),
            Err(MailDeliveryIntentExecutionErrorV1::InvalidBody),
        );
    }
}
