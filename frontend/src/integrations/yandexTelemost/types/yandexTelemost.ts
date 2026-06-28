export interface YandexTelemostCapabilityState {
  capability: string
  status: string
  source: string
  confidence: number
  evidence: Record<string, unknown>
}

export interface YandexTelemostLocalRecordingPolicy {
  macos: string
  linux: string
  windows: string
  ffmpeg_path_env: string
  ffmpeg_input_env: string
}

export interface YandexTelemostLocalRecordingManifest {
  state: string
  audio_format: 'mp3'
  recorder_boundary: string
  consent_required: boolean
  default_output_policy: string
  audio_device_policy: YandexTelemostLocalRecordingPolicy
}

export interface YandexTelemostSpeakerTimelinePolicy {
  state: string
  source: string
  reliability: string
  output_files: string[]
  role_in_transcription: string
}

export interface YandexTelemostCapabilitiesResponse {
  provider_kind: 'yandex_telemost_user'
  api_base_url: string
  web_origin: string
  capabilities: YandexTelemostCapabilityState[]
  recording_policy: YandexTelemostLocalRecordingManifest
  speaker_timeline_policy: YandexTelemostSpeakerTimelinePolicy
}

export interface YandexTelemostAccount {
  account_id: string
  provider_kind: 'yandex_telemost_user'
  display_name: string
  external_account_id: string
  lifecycle_state: string
  runtime_kind: string
  api_base_url: string
  token_secret_ref?: string | null
  join_webview_available: boolean
  local_recorder_available: boolean
  config: Record<string, unknown>
  created_at: string
  updated_at: string
}

export interface YandexTelemostAccountListResponse {
  items: YandexTelemostAccount[]
}

export interface YandexTelemostAccountSetupRequest {
  account_id: string
  display_name: string
  external_account_id: string
  oauth_token?: string
  oauth_token_ref?: string
  api_base_url?: string
  metadata?: Record<string, unknown>
}

export interface YandexTelemostAccountSetupResponse {
  account: YandexTelemostAccount
}

export interface YandexTelemostRuntimeStatus {
  account_id: string
  provider_kind: string
  lifecycle_state: string
  runtime_kind: string
  checked_at: string
  api_base_url: string
  authorized: boolean
  blockers: string[]
  capabilities: YandexTelemostCapabilityState[]
}

export interface YandexTelemostCohost {
  email: string
}

export interface YandexTelemostLiveStreamRequest {
  access_level?: string
  title?: string
  description?: string
}

export interface YandexTelemostLiveStreamResponse {
  watch_url?: string | null
  access_level?: string | null
  title?: string | null
  description?: string | null
}

export interface YandexTelemostConferenceCreateRequest {
  account_id: string
  waiting_room_level?: string
  live_stream?: YandexTelemostLiveStreamRequest
  cohosts?: YandexTelemostCohost[]
  is_auto_summarization_enabled?: boolean
  metadata?: Record<string, unknown>
}

export interface YandexTelemostConferenceUpdateRequest {
  waiting_room_level?: string
  live_stream?: YandexTelemostLiveStreamRequest
  cohosts?: YandexTelemostCohost[]
  is_auto_summarization_enabled?: boolean
  metadata?: Record<string, unknown>
}

export interface YandexTelemostConference {
  id: string
  join_url: string
  access_level?: string | null
  waiting_room_level?: string | null
  live_stream?: YandexTelemostLiveStreamResponse | null
  sip_uri_meeting?: string | null
  sip_uri_telemost?: string | null
  sip_id?: string | null
}

export interface YandexTelemostConferenceOperationResponse {
  account_id: string
  conference: YandexTelemostConference
  status: 'created' | 'observed' | 'updated'
}

export interface YandexTelemostCohostPage {
  cohosts: YandexTelemostCohost[]
}

export interface YandexTelemostWebviewManifestRequest {
  account_id: string
  conference_id?: string | null
  join_url: string
  display_name?: string | null
}

export interface YandexTelemostConferenceWebviewManifest {
  account_id: string
  conference_id?: string | null
  join_url: string
  target_origin: string
  provider_shape: string
  runtime_kind: string
  window_label: string
  opened_window: boolean
  focused_existing_window: boolean
  owner_visible: boolean
  hidden_headless_mode: string
  local_recording: YandexTelemostLocalRecordingManifest
  speaker_timeline: YandexTelemostSpeakerTimelinePolicy
}

export interface YandexTelemostRecordingIntentResponse {
  account_id: string
  conference_id?: string | null
  join_url: string
  consent_required: boolean
  source_of_truth: false
  local_recording: YandexTelemostLocalRecordingManifest
  speaker_timeline: YandexTelemostSpeakerTimelinePolicy
  tauri_commands: Record<string, string>
}

export interface YandexTelemostCompanionOpenRequest {
  account_id: string
  join_url: string
  conference_id?: string | null
  display_name?: string | null
}

export interface YandexTelemostCompanionManifest {
  account_id: string
  conference_id?: string | null
  join_url: string
  provider_shape: string
  runtime_kind: string
  window_label: string
  opened_window: boolean
  focused_existing_window: boolean
  owner_visible: boolean
  hidden_headless_mode: string
  allowed_hosts: string[]
  speaker_timeline: Record<string, unknown>
  recorder: Record<string, unknown>
}

export interface YandexTelemostRecordingSession {
  recording_session_id: string
  account_id: string
  conference_id?: string | null
  join_url: string
  window_label: string
  output_dir: string
  audio_path: string
  speaker_jsonl_path: string
  speaker_txt_path: string
  ffmpeg_pid?: number | null
  started_at_epoch_ms: number
  consent_attested: boolean
}

export interface YandexTelemostRecordingStopReceipt {
  recording_session_id: string
  account_id: string
  conference_id?: string | null
  audio_path: string
  speaker_jsonl_path: string
  speaker_txt_path: string
  stopped_at_epoch_ms: number
  state: string
}

export interface YandexTelemostRecordingBridgeRequest {
  account_id: string
  conference_id?: string | null
  join_url: string
  recording_session_id: string
  output_dir: string
  audio_path: string
  speaker_jsonl_path: string
  speaker_txt_path: string
  started_at_epoch_ms: number
  stopped_at_epoch_ms: number
  consent_attested: boolean
}

export interface YandexTelemostRecordingBridgeResponse {
  account_id: string
  conference_id?: string | null
  recording_session_id: string
  bundle_id: string
  bundle_root: string
  manifest_path: string
  follow_up_events: string[]
  radar_signal_kinds: string[]
}
