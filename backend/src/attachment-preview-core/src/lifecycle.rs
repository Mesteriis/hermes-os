use hermes_attachment_preview_api::wire::AttachmentPreviewStateV1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentPreviewTransitionErrorV1 {
    InvalidState,
    InvalidTransition,
}

pub fn transition_attachment_preview_v1(
    current: AttachmentPreviewStateV1,
    next: AttachmentPreviewStateV1,
) -> Result<AttachmentPreviewStateV1, AttachmentPreviewTransitionErrorV1> {
    use AttachmentPreviewStateV1::{
        Accepted, AwaitingEvidence, Ready, Rejected, Rendering, Unsupported,
    };
    if current == next {
        return Ok(current);
    }
    let allowed = matches!(
        (current, next),
        (Accepted, AwaitingEvidence)
            | (Accepted, Rejected)
            | (AwaitingEvidence, Rendering)
            | (AwaitingEvidence, Unsupported)
            | (AwaitingEvidence, Rejected)
            | (Rendering, Ready)
            | (Rendering, Unsupported)
            | (Rendering, Rejected)
    );
    if allowed {
        Ok(next)
    } else if current == AttachmentPreviewStateV1::Unspecified
        || next == AttachmentPreviewStateV1::Unspecified
    {
        Err(AttachmentPreviewTransitionErrorV1::InvalidState)
    } else {
        Err(AttachmentPreviewTransitionErrorV1::InvalidTransition)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_states_cannot_be_reopened() {
        assert_eq!(
            transition_attachment_preview_v1(
                AttachmentPreviewStateV1::Ready,
                AttachmentPreviewStateV1::Rendering,
            ),
            Err(AttachmentPreviewTransitionErrorV1::InvalidTransition)
        );
        assert_eq!(
            transition_attachment_preview_v1(
                AttachmentPreviewStateV1::Rendering,
                AttachmentPreviewStateV1::Ready,
            ),
            Ok(AttachmentPreviewStateV1::Ready)
        );
    }
}
