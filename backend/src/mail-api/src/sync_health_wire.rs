//! Canonical Protobuf mapping for the Mail sync-health query contract.

use prost::Message;

use crate::{
    client_wire::MailClientWireErrorV1,
    sync_health::{
        MailSyncFailureCodeV1, MailSyncHealthQueryResponseV1, MailSyncHealthQueryV1,
        MailSyncOutcomeV1, MailSyncProviderPathReadinessV1, MailSyncRunPageV1, MailSyncRunV1,
        MailSyncStatusV1, MailSyncTriggerV1, validate_sync_health_query,
        validate_sync_health_response,
    },
    sync_health_wire_generated as wire,
};

pub fn encode_sync_health_query(
    query: &MailSyncHealthQueryV1,
) -> Result<Vec<u8>, MailClientWireErrorV1> {
    validate_sync_health_query(query).map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    use wire::mail_sync_health_query_v1::Query;
    let query = match query {
        MailSyncHealthQueryV1::GetStatus { connection_id } => {
            Query::GetStatus(wire::GetMailSyncStatusQueryV1 {
                connection_id: connection_id.clone(),
            })
        }
        MailSyncHealthQueryV1::ListRuns {
            connection_id,
            cursor,
            limit,
        } => Query::ListRuns(wire::ListMailSyncRunsQueryV1 {
            connection_id: connection_id.clone(),
            cursor: cursor.clone(),
            limit: *limit,
        }),
        MailSyncHealthQueryV1::GetRun {
            connection_id,
            operation_id,
        } => Query::GetRun(wire::GetMailSyncRunQueryV1 {
            connection_id: connection_id.clone(),
            operation_id: operation_id.clone(),
        }),
    };
    Ok(wire::MailSyncHealthQueryV1 { query: Some(query) }.encode_to_vec())
}

pub fn decode_sync_health_query(
    bytes: &[u8],
) -> Result<MailSyncHealthQueryV1, MailClientWireErrorV1> {
    use wire::mail_sync_health_query_v1::Query;
    let query = wire::MailSyncHealthQueryV1::decode(bytes)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?
        .query
        .ok_or(MailClientWireErrorV1::InvalidPayload)?;
    let query = match query {
        Query::GetStatus(value) => MailSyncHealthQueryV1::GetStatus {
            connection_id: value.connection_id,
        },
        Query::ListRuns(value) => MailSyncHealthQueryV1::ListRuns {
            connection_id: value.connection_id,
            cursor: value.cursor,
            limit: value.limit,
        },
        Query::GetRun(value) => MailSyncHealthQueryV1::GetRun {
            connection_id: value.connection_id,
            operation_id: value.operation_id,
        },
    };
    validate_sync_health_query(&query).map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    if encode_sync_health_query(&query)? != bytes {
        return Err(MailClientWireErrorV1::InvalidPayload);
    }
    Ok(query)
}

pub fn encode_sync_health_response(
    response: &MailSyncHealthQueryResponseV1,
) -> Result<Vec<u8>, MailClientWireErrorV1> {
    validate_sync_health_response(response).map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    use wire::mail_sync_health_query_response_v1::Response;
    let response = match response {
        MailSyncHealthQueryResponseV1::Status(status) => Response::Status(status_to_wire(status)),
        MailSyncHealthQueryResponseV1::Runs(page) => Response::Runs(wire::MailSyncRunPageV1 {
            item: page.items.iter().map(run_to_wire).collect(),
            next_cursor: page.next_cursor.clone(),
        }),
        MailSyncHealthQueryResponseV1::Run(run) => Response::Run(wire::OptionalMailSyncRunV1 {
            run: run.as_ref().map(run_to_wire),
        }),
    };
    Ok(wire::MailSyncHealthQueryResponseV1 {
        response: Some(response),
    }
    .encode_to_vec())
}

pub fn decode_sync_health_response(
    bytes: &[u8],
) -> Result<MailSyncHealthQueryResponseV1, MailClientWireErrorV1> {
    use wire::mail_sync_health_query_response_v1::Response;
    let response = wire::MailSyncHealthQueryResponseV1::decode(bytes)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?
        .response
        .ok_or(MailClientWireErrorV1::InvalidPayload)?;
    let response = match response {
        Response::Status(status) => {
            MailSyncHealthQueryResponseV1::Status(status_from_wire(status)?)
        }
        Response::Runs(page) => MailSyncHealthQueryResponseV1::Runs(MailSyncRunPageV1 {
            items: page
                .item
                .into_iter()
                .map(run_from_wire)
                .collect::<Result<_, _>>()?,
            next_cursor: page.next_cursor,
        }),
        Response::Run(run) => {
            MailSyncHealthQueryResponseV1::Run(run.run.map(run_from_wire).transpose()?)
        }
    };
    validate_sync_health_response(&response).map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    if encode_sync_health_response(&response)? != bytes {
        return Err(MailClientWireErrorV1::InvalidPayload);
    }
    Ok(response)
}

fn run_to_wire(run: &MailSyncRunV1) -> wire::MailSyncRunV1 {
    wire::MailSyncRunV1 {
        operation_id: run.operation_id.clone(),
        connection_id: run.connection_id.clone(),
        trigger: trigger_to_wire(run.trigger) as i32,
        outcome: outcome_to_wire(run.outcome) as i32,
        observed_messages: run.observed_messages,
        started_at_unix_seconds: run.started_at_unix_seconds,
        completed_at_unix_seconds: run.completed_at_unix_seconds,
        failure_code: run.failure_code.map(|value| failure_to_wire(value) as i32),
        runtime_generation: run.runtime_generation,
        projection_revision: run.projection_revision,
    }
}

fn run_from_wire(run: wire::MailSyncRunV1) -> Result<MailSyncRunV1, MailClientWireErrorV1> {
    let run = MailSyncRunV1 {
        operation_id: run.operation_id,
        connection_id: run.connection_id,
        trigger: trigger_from_wire(run.trigger)?,
        outcome: outcome_from_wire(run.outcome)?,
        observed_messages: run.observed_messages,
        started_at_unix_seconds: run.started_at_unix_seconds,
        completed_at_unix_seconds: run.completed_at_unix_seconds,
        failure_code: run.failure_code.map(failure_from_wire).transpose()?,
        runtime_generation: run.runtime_generation,
        projection_revision: run.projection_revision,
    };
    crate::sync_health::validate_sync_run(&run)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    Ok(run)
}

fn status_to_wire(status: &MailSyncStatusV1) -> wire::MailSyncStatusV1 {
    wire::MailSyncStatusV1 {
        connection_id: status.connection_id.clone(),
        provider_path_readiness: readiness_to_wire(status.provider_path_readiness) as i32,
        latest_run: status.latest_run.as_deref().map(run_to_wire),
        consecutive_failures: status.consecutive_failures,
        last_success_at_unix_seconds: status.last_success_at_unix_seconds,
        projection_revision: status.projection_revision,
    }
}

fn status_from_wire(
    status: wire::MailSyncStatusV1,
) -> Result<MailSyncStatusV1, MailClientWireErrorV1> {
    Ok(MailSyncStatusV1 {
        connection_id: status.connection_id,
        provider_path_readiness: readiness_from_wire(status.provider_path_readiness)?,
        latest_run: status
            .latest_run
            .map(run_from_wire)
            .transpose()?
            .map(Box::new),
        consecutive_failures: status.consecutive_failures,
        last_success_at_unix_seconds: status.last_success_at_unix_seconds,
        projection_revision: status.projection_revision,
    })
}

const fn trigger_to_wire(value: MailSyncTriggerV1) -> wire::MailSyncTriggerV1 {
    match value {
        MailSyncTriggerV1::Manual => wire::MailSyncTriggerV1::MailSyncTriggerManual,
        MailSyncTriggerV1::Scheduled => wire::MailSyncTriggerV1::MailSyncTriggerScheduled,
    }
}

fn trigger_from_wire(value: i32) -> Result<MailSyncTriggerV1, MailClientWireErrorV1> {
    match wire::MailSyncTriggerV1::try_from(value)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?
    {
        wire::MailSyncTriggerV1::MailSyncTriggerManual => Ok(MailSyncTriggerV1::Manual),
        wire::MailSyncTriggerV1::MailSyncTriggerScheduled => Ok(MailSyncTriggerV1::Scheduled),
        wire::MailSyncTriggerV1::MailSyncTriggerUnspecified => {
            Err(MailClientWireErrorV1::InvalidPayload)
        }
    }
}

const fn outcome_to_wire(value: MailSyncOutcomeV1) -> wire::MailSyncOutcomeV1 {
    match value {
        MailSyncOutcomeV1::Running => wire::MailSyncOutcomeV1::MailSyncOutcomeRunning,
        MailSyncOutcomeV1::Succeeded => wire::MailSyncOutcomeV1::MailSyncOutcomeSucceeded,
        MailSyncOutcomeV1::Failed => wire::MailSyncOutcomeV1::MailSyncOutcomeFailed,
        MailSyncOutcomeV1::Interrupted => wire::MailSyncOutcomeV1::MailSyncOutcomeInterrupted,
    }
}

fn outcome_from_wire(value: i32) -> Result<MailSyncOutcomeV1, MailClientWireErrorV1> {
    match wire::MailSyncOutcomeV1::try_from(value)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?
    {
        wire::MailSyncOutcomeV1::MailSyncOutcomeRunning => Ok(MailSyncOutcomeV1::Running),
        wire::MailSyncOutcomeV1::MailSyncOutcomeSucceeded => Ok(MailSyncOutcomeV1::Succeeded),
        wire::MailSyncOutcomeV1::MailSyncOutcomeFailed => Ok(MailSyncOutcomeV1::Failed),
        wire::MailSyncOutcomeV1::MailSyncOutcomeInterrupted => Ok(MailSyncOutcomeV1::Interrupted),
        wire::MailSyncOutcomeV1::MailSyncOutcomeUnspecified => {
            Err(MailClientWireErrorV1::InvalidPayload)
        }
    }
}

const fn failure_to_wire(value: MailSyncFailureCodeV1) -> wire::MailSyncFailureCodeV1 {
    match value {
        MailSyncFailureCodeV1::AdmissionRejected => {
            wire::MailSyncFailureCodeV1::MailSyncFailureCodeAdmissionRejected
        }
        MailSyncFailureCodeV1::ControlUnavailable => {
            wire::MailSyncFailureCodeV1::MailSyncFailureCodeControlUnavailable
        }
        MailSyncFailureCodeV1::StorageUnavailable => {
            wire::MailSyncFailureCodeV1::MailSyncFailureCodeStorageUnavailable
        }
        MailSyncFailureCodeV1::CredentialUnavailable => {
            wire::MailSyncFailureCodeV1::MailSyncFailureCodeCredentialUnavailable
        }
        MailSyncFailureCodeV1::PersistenceUnavailable => {
            wire::MailSyncFailureCodeV1::MailSyncFailureCodePersistenceUnavailable
        }
        MailSyncFailureCodeV1::ProviderUnavailable => {
            wire::MailSyncFailureCodeV1::MailSyncFailureCodeProviderUnavailable
        }
        MailSyncFailureCodeV1::EventHubUnavailable => {
            wire::MailSyncFailureCodeV1::MailSyncFailureCodeEventHubUnavailable
        }
        MailSyncFailureCodeV1::AttachmentAnchorUnavailable => {
            wire::MailSyncFailureCodeV1::MailSyncFailureCodeAttachmentAnchorUnavailable
        }
        MailSyncFailureCodeV1::RuntimeRestarted => {
            wire::MailSyncFailureCodeV1::MailSyncFailureCodeRuntimeRestarted
        }
    }
}

fn failure_from_wire(value: i32) -> Result<MailSyncFailureCodeV1, MailClientWireErrorV1> {
    match wire::MailSyncFailureCodeV1::try_from(value)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?
    {
        wire::MailSyncFailureCodeV1::MailSyncFailureCodeAdmissionRejected => {
            Ok(MailSyncFailureCodeV1::AdmissionRejected)
        }
        wire::MailSyncFailureCodeV1::MailSyncFailureCodeControlUnavailable => {
            Ok(MailSyncFailureCodeV1::ControlUnavailable)
        }
        wire::MailSyncFailureCodeV1::MailSyncFailureCodeStorageUnavailable => {
            Ok(MailSyncFailureCodeV1::StorageUnavailable)
        }
        wire::MailSyncFailureCodeV1::MailSyncFailureCodeCredentialUnavailable => {
            Ok(MailSyncFailureCodeV1::CredentialUnavailable)
        }
        wire::MailSyncFailureCodeV1::MailSyncFailureCodePersistenceUnavailable => {
            Ok(MailSyncFailureCodeV1::PersistenceUnavailable)
        }
        wire::MailSyncFailureCodeV1::MailSyncFailureCodeProviderUnavailable => {
            Ok(MailSyncFailureCodeV1::ProviderUnavailable)
        }
        wire::MailSyncFailureCodeV1::MailSyncFailureCodeEventHubUnavailable => {
            Ok(MailSyncFailureCodeV1::EventHubUnavailable)
        }
        wire::MailSyncFailureCodeV1::MailSyncFailureCodeAttachmentAnchorUnavailable => {
            Ok(MailSyncFailureCodeV1::AttachmentAnchorUnavailable)
        }
        wire::MailSyncFailureCodeV1::MailSyncFailureCodeRuntimeRestarted => {
            Ok(MailSyncFailureCodeV1::RuntimeRestarted)
        }
        wire::MailSyncFailureCodeV1::MailSyncFailureCodeUnspecified => {
            Err(MailClientWireErrorV1::InvalidPayload)
        }
    }
}

const fn readiness_to_wire(
    value: MailSyncProviderPathReadinessV1,
) -> wire::MailSyncProviderPathReadinessV1 {
    match value {
        MailSyncProviderPathReadinessV1::Ready => {
            wire::MailSyncProviderPathReadinessV1::MailSyncProviderPathReadinessReady
        }
        MailSyncProviderPathReadinessV1::Unavailable => {
            wire::MailSyncProviderPathReadinessV1::MailSyncProviderPathReadinessUnavailable
        }
    }
}

fn readiness_from_wire(
    value: i32,
) -> Result<MailSyncProviderPathReadinessV1, MailClientWireErrorV1> {
    match wire::MailSyncProviderPathReadinessV1::try_from(value)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?
    {
        wire::MailSyncProviderPathReadinessV1::MailSyncProviderPathReadinessReady => {
            Ok(MailSyncProviderPathReadinessV1::Ready)
        }
        wire::MailSyncProviderPathReadinessV1::MailSyncProviderPathReadinessUnavailable => {
            Ok(MailSyncProviderPathReadinessV1::Unavailable)
        }
        wire::MailSyncProviderPathReadinessV1::MailSyncProviderPathReadinessUnspecified => {
            Err(MailClientWireErrorV1::InvalidPayload)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_query_and_response_round_trip_canonically() {
        let query = MailSyncHealthQueryV1::ListRuns {
            connection_id: "account-1".to_owned(),
            cursor: Some("cursor-1".to_owned()),
            limit: 25,
        };
        let encoded = encode_sync_health_query(&query).expect("query");
        assert_eq!(decode_sync_health_query(&encoded), Ok(query));

        let response = MailSyncHealthQueryResponseV1::Run(None);
        let encoded = encode_sync_health_response(&response).expect("response");
        assert_eq!(decode_sync_health_response(&encoded), Ok(response));
    }
}
