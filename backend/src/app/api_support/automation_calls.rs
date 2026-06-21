use super::*;

#[derive(Deserialize)]
pub(crate) struct PolicyTemplateApiRequest {
    pub(crate) template_id: String,
    pub(crate) name: String,
    pub(crate) body_template: String,
    #[serde(default)]
    pub(crate) required_variables: Vec<String>,
}

impl PolicyTemplateApiRequest {
    pub(crate) fn into_template(self) -> NewAutomationTemplate {
        NewAutomationTemplate {
            template_id: self.template_id,
            name: self.name,
            body_template: self.body_template,
            required_variables: self.required_variables,
        }
    }
}

#[derive(Serialize)]
pub(crate) struct PolicyTemplateListResponse {
    pub(crate) items: Vec<AutomationTemplate>,
}

#[derive(Deserialize)]
pub(crate) struct PolicyApiRequest {
    pub(crate) policy_id: String,
    pub(crate) template_id: String,
    pub(crate) name: String,
    pub(crate) enabled: bool,
    pub(crate) account_id: String,
    pub(crate) allowed_chat_ids: Vec<String>,
    pub(crate) trigger_kind: String,
    pub(crate) max_sends_per_hour: i32,
    #[serde(default = "empty_json_object")]
    pub(crate) quiet_hours: Value,
    pub(crate) expires_at: Option<DateTime<Utc>>,
    #[serde(default = "empty_json_object")]
    pub(crate) conditions: Value,
}

impl PolicyApiRequest {
    pub(crate) fn into_policy(self) -> NewAutomationPolicy {
        NewAutomationPolicy {
            policy_id: self.policy_id,
            template_id: self.template_id,
            name: self.name,
            enabled: self.enabled,
            account_id: self.account_id,
            allowed_chat_ids: self.allowed_chat_ids,
            trigger_kind: self.trigger_kind,
            max_sends_per_hour: self.max_sends_per_hour,
            quiet_hours: self.quiet_hours,
            expires_at: self.expires_at,
            conditions: self.conditions,
        }
    }
}

#[derive(Serialize)]
pub(crate) struct PolicyListResponse {
    pub(crate) items: Vec<AutomationPolicy>,
}

#[derive(Deserialize)]
pub(crate) struct CallApiRequest {
    pub(crate) call_id: String,
    pub(crate) account_id: String,
    pub(crate) provider_call_id: String,
    pub(crate) provider_chat_id: String,
    pub(crate) direction: CallDirection,
    pub(crate) call_state: CallState,
    pub(crate) started_at: Option<DateTime<Utc>>,
    pub(crate) ended_at: Option<DateTime<Utc>>,
    pub(crate) transcription_policy_id: Option<String>,
    #[serde(default = "empty_json_object")]
    pub(crate) metadata: Value,
}

impl CallApiRequest {
    pub(crate) fn into_call(self) -> NewTelegramCall {
        NewTelegramCall {
            call_id: self.call_id,
            account_id: self.account_id,
            provider_call_id: self.provider_call_id,
            provider_chat_id: self.provider_chat_id,
            direction: self.direction,
            call_state: self.call_state,
            started_at: self.started_at,
            ended_at: self.ended_at,
            transcription_policy_id: self.transcription_policy_id,
            metadata: self.metadata,
        }
    }
}

#[derive(Serialize)]
pub(crate) struct CallListResponse {
    pub(crate) items: Vec<TelegramCall>,
}

#[derive(Deserialize)]
pub(crate) struct CallTranscriptFixtureApiRequest {
    pub(crate) transcript_id: String,
    pub(crate) account_id: String,
    pub(crate) provider_chat_id: String,
    pub(crate) source_audio_ref: String,
    pub(crate) language_code: Option<String>,
    #[serde(default)]
    pub(crate) always_on_policy: bool,
}

#[derive(Serialize)]
pub(crate) struct CallTranscriptResponse {
    pub(crate) transcript: Option<CallTranscript>,
}
