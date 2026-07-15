use super::*;

#[derive(Clone, Debug, serde::Deserialize)]
pub(super) struct ZoomApiEventSubscriptionListResponse {
    #[serde(default, alias = "subscriptions")]
    pub(super) event_subscriptions: Vec<ZoomApiEventSubscription>,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub(super) struct ZoomApiEventSubscription {
    #[serde(alias = "id", alias = "event_subscription_id")]
    subscription_id: Option<String>,
    #[serde(alias = "subscription_name", alias = "event_subscription_name")]
    subscription_name: Option<String>,
    #[serde(alias = "event_webhook_url", alias = "webhook_url")]
    endpoint_url: Option<String>,
    #[serde(default, alias = "events")]
    event_types: Vec<String>,
}

impl ZoomApiEventSubscription {
    pub(super) fn into_public(self) -> Option<ZoomWebhookSubscription> {
        let subscription_id = self
            .subscription_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())?
            .to_owned();
        let subscription_name = self
            .subscription_name
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())?
            .to_owned();
        let endpoint_url = self
            .endpoint_url
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())?
            .to_owned();
        Some(ZoomWebhookSubscription {
            subscription_id,
            subscription_name,
            endpoint_url,
            event_types: canonical_zoom_webhook_event_types(&self.event_types),
        })
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
pub(super) struct ZoomApiRecordingListResponse {
    #[serde(default)]
    pub(super) meetings: Vec<ZoomApiRecordingMeeting>,
    #[serde(default)]
    pub(super) next_page_token: Option<String>,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub(super) struct ZoomApiRecordingMeeting {
    id: Value,
    pub(super) uuid: Option<String>,
    topic: Option<String>,
    host_email: Option<String>,
    join_url: Option<String>,
    start_time: Option<String>,
    end_time: Option<String>,
    duration: Option<i64>,
    #[serde(default)]
    pub(super) recording_files: Vec<ZoomApiRecordingFile>,
}

impl ZoomApiRecordingMeeting {
    pub(super) fn meeting_id(&self) -> Result<String, ZoomError> {
        json_string_or_number("meeting.id", &self.id)
    }

    pub(super) fn to_meeting_observation(
        &self,
        account_id: &str,
        meeting_id: &str,
    ) -> ZoomMeetingObservationRequest {
        let recording_refs = self
            .recording_files
            .iter()
            .filter_map(|item| item.to_recording_ref().ok())
            .collect::<Vec<ZoomRecordingRef>>();
        ZoomMeetingObservationRequest {
            observation_id: Some(format!("zoom-provider-sync-meeting:{meeting_id}")),
            account_id: account_id.to_owned(),
            meeting_id: meeting_id.to_owned(),
            meeting_uuid: self.uuid.clone(),
            topic: self.topic.clone(),
            host_email: self.host_email.clone(),
            join_url: self.join_url.clone(),
            started_at: zoom_api_datetime(self.start_time.as_deref()),
            ended_at: zoom_api_datetime(self.end_time.as_deref()),
            duration_seconds: self.duration.map(|minutes| minutes.saturating_mul(60)),
            participants: Vec::new(),
            recording_refs,
            transcript_ref: None,
            metadata: json!({
                "source": "zoom_provider_sync",
            }),
            causation_id: Some(format!("zoom-provider-sync:{account_id}")),
            correlation_id: Some(format!(
                "zoom-provider-sync-meeting:{account_id}:{meeting_id}"
            )),
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
pub(super) struct ZoomApiRecordingFile {
    id: Value,
    pub(super) file_type: Option<String>,
    pub(super) recording_type: Option<String>,
    download_url: Option<String>,
    pub(super) file_extension: Option<String>,
    file_size: Option<i64>,
    recording_start: Option<String>,
}

impl ZoomApiRecordingFile {
    pub(super) fn recording_id(&self) -> Result<String, ZoomError> {
        json_string_or_number("recording_file.id", &self.id)
    }

    pub(super) fn to_recording_ref(&self) -> Result<ZoomRecordingRef, ZoomError> {
        Ok(ZoomRecordingRef {
            recording_id: self.recording_id()?,
            recording_type: self.recording_type.clone(),
            download_ref: self.download_url(),
            file_extension: self
                .file_extension
                .clone()
                .or_else(|| self.file_type.clone()),
            file_size_bytes: self.file_size,
            recorded_at: zoom_api_datetime(self.recording_start.as_deref()),
            metadata: json!({
                "file_type": self.file_type,
                "source": "zoom_provider_sync",
            }),
        })
    }

    pub(super) fn download_url(&self) -> Option<String> {
        self.download_url
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
    }

    pub(super) fn transcript_file_name(&self, meeting_id: &str) -> Option<String> {
        let extension = self
            .file_extension
            .as_deref()
            .or(self.file_type.as_deref())
            .map(str::trim)
            .filter(|value| !value.is_empty())?
            .to_ascii_lowercase();
        Some(format!(
            "{}-{}.{}",
            meeting_id.trim(),
            self.recording_id().ok()?,
            extension
        ))
    }

    pub(super) fn transcript_content_type(&self) -> Option<String> {
        zoom_transcript_content_type(self.file_extension.as_deref().or(self.file_type.as_deref()))
    }

    pub(super) fn file_name(&self, meeting_id: &str) -> Option<String> {
        let extension = self
            .file_extension
            .as_deref()
            .or(self.file_type.as_deref())
            .map(str::trim)
            .filter(|value| !value.is_empty())?
            .to_ascii_lowercase();
        Some(format!(
            "{}-{}.{}",
            meeting_id.trim(),
            self.recording_id().ok()?,
            extension
        ))
    }

    pub(super) fn content_type(&self) -> Option<String> {
        zoom_recording_content_type(self.file_extension.as_deref().or(self.file_type.as_deref()))
    }
}
