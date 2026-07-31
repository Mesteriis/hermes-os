use crate::ArchiveInspectionReportV1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArchiveInspectionStateV1 {
    Accepted,
    AwaitingEvidence,
    Inspecting,
    Ready,
    Rejected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArchiveInspectionErrorV1 {
    NotSafe,
    NotZip,
    PolicyRejected,
    CorruptArchive,
    Unavailable,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArchiveInspectionStatusV1 {
    pub state: ArchiveInspectionStateV1,
    pub state_revision: u64,
    pub report: Option<ArchiveInspectionReportV1>,
    pub error: Option<ArchiveInspectionErrorV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ArchiveInspectionTransitionV1 {
    AwaitEvidence,
    BeginInspection,
    Complete(ArchiveInspectionReportV1),
    Reject(ArchiveInspectionErrorV1),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArchiveInspectionTransitionErrorV1 {
    InvalidCurrentStatus,
    InvalidTransition,
    InvalidReport,
}

#[must_use]
pub const fn accepted_archive_inspection_status_v1() -> ArchiveInspectionStatusV1 {
    ArchiveInspectionStatusV1 {
        state: ArchiveInspectionStateV1::Accepted,
        state_revision: 1,
        report: None,
        error: None,
    }
}

pub fn transition_archive_inspection_status_v1(
    current: &ArchiveInspectionStatusV1,
    transition: ArchiveInspectionTransitionV1,
) -> Result<ArchiveInspectionStatusV1, ArchiveInspectionTransitionErrorV1> {
    if !validate_archive_inspection_status_v1(current) {
        return Err(ArchiveInspectionTransitionErrorV1::InvalidCurrentStatus);
    }
    let (state, report, error) = match (current.state, transition) {
        (ArchiveInspectionStateV1::Accepted, ArchiveInspectionTransitionV1::AwaitEvidence) => {
            (ArchiveInspectionStateV1::AwaitingEvidence, None, None)
        }
        (
            ArchiveInspectionStateV1::Accepted | ArchiveInspectionStateV1::AwaitingEvidence,
            ArchiveInspectionTransitionV1::BeginInspection,
        ) => (ArchiveInspectionStateV1::Inspecting, None, None),
        (ArchiveInspectionStateV1::Inspecting, ArchiveInspectionTransitionV1::Complete(report))
            if valid_report(&report) =>
        {
            (ArchiveInspectionStateV1::Ready, Some(report), None)
        }
        (
            ArchiveInspectionStateV1::Accepted
            | ArchiveInspectionStateV1::AwaitingEvidence
            | ArchiveInspectionStateV1::Inspecting,
            ArchiveInspectionTransitionV1::Reject(error),
        ) => (ArchiveInspectionStateV1::Rejected, None, Some(error)),
        (ArchiveInspectionStateV1::Inspecting, ArchiveInspectionTransitionV1::Complete(_)) => {
            return Err(ArchiveInspectionTransitionErrorV1::InvalidReport);
        }
        _ => return Err(ArchiveInspectionTransitionErrorV1::InvalidTransition),
    };
    let next = ArchiveInspectionStatusV1 {
        state,
        state_revision: current
            .state_revision
            .checked_add(1)
            .ok_or(ArchiveInspectionTransitionErrorV1::InvalidTransition)?,
        report,
        error,
    };
    if !validate_archive_inspection_status_v1(&next) {
        return Err(ArchiveInspectionTransitionErrorV1::InvalidTransition);
    }
    Ok(next)
}

#[must_use]
pub fn validate_archive_inspection_status_v1(status: &ArchiveInspectionStatusV1) -> bool {
    status.state_revision > 0
        && match status.state {
            ArchiveInspectionStateV1::Accepted
            | ArchiveInspectionStateV1::AwaitingEvidence
            | ArchiveInspectionStateV1::Inspecting => {
                status.report.is_none() && status.error.is_none()
            }
            ArchiveInspectionStateV1::Ready => {
                status.report.as_ref().is_some_and(valid_report) && status.error.is_none()
            }
            ArchiveInspectionStateV1::Rejected => status.report.is_none() && status.error.is_some(),
        }
}

fn valid_report(report: &ArchiveInspectionReportV1) -> bool {
    report.entry_count == report.entries.len()
        && report.total_uncompressed_bytes
            == report.entries.iter().fold(0_u64, |total, entry| {
                total.saturating_add(entry.uncompressed_size)
            })
}

#[cfg(test)]
mod tests {
    use crate::{ArchiveEntryInspectionV1, ArchiveEntryKindV1};

    use super::*;

    #[test]
    fn state_machine_requires_evidence_before_work_and_report_before_ready() {
        let accepted = accepted_archive_inspection_status_v1();
        let awaiting = transition_archive_inspection_status_v1(
            &accepted,
            ArchiveInspectionTransitionV1::AwaitEvidence,
        )
        .expect("awaiting");
        let inspecting = transition_archive_inspection_status_v1(
            &awaiting,
            ArchiveInspectionTransitionV1::BeginInspection,
        )
        .expect("inspecting");
        let ready = transition_archive_inspection_status_v1(
            &inspecting,
            ArchiveInspectionTransitionV1::Complete(report()),
        )
        .expect("ready");
        assert_eq!(ready.state, ArchiveInspectionStateV1::Ready);
        assert_eq!(ready.state_revision, 4);
        assert!(ready.report.is_some());
    }

    #[test]
    fn unsafe_terminal_and_invalid_reentry_fail_closed() {
        let rejected = transition_archive_inspection_status_v1(
            &accepted_archive_inspection_status_v1(),
            ArchiveInspectionTransitionV1::Reject(ArchiveInspectionErrorV1::NotSafe),
        )
        .expect("reject");
        assert_eq!(rejected.state, ArchiveInspectionStateV1::Rejected);
        assert_eq!(rejected.error, Some(ArchiveInspectionErrorV1::NotSafe));
        assert_eq!(
            transition_archive_inspection_status_v1(
                &rejected,
                ArchiveInspectionTransitionV1::BeginInspection,
            ),
            Err(ArchiveInspectionTransitionErrorV1::InvalidTransition)
        );

        let inspecting = transition_archive_inspection_status_v1(
            &accepted_archive_inspection_status_v1(),
            ArchiveInspectionTransitionV1::BeginInspection,
        )
        .expect("inspecting");
        let mut invalid = report();
        invalid.entry_count = 2;
        assert_eq!(
            transition_archive_inspection_status_v1(
                &inspecting,
                ArchiveInspectionTransitionV1::Complete(invalid),
            ),
            Err(ArchiveInspectionTransitionErrorV1::InvalidReport)
        );
    }

    fn report() -> ArchiveInspectionReportV1 {
        ArchiveInspectionReportV1 {
            entry_count: 1,
            total_uncompressed_bytes: 42,
            entries: vec![ArchiveEntryInspectionV1 {
                normalized_path: "message.txt".to_owned(),
                compressed_size: 24,
                uncompressed_size: 42,
                kind: ArchiveEntryKindV1::File,
            }],
        }
    }
}
