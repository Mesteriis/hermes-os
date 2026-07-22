export const DEFAULT_MAIL_BATCH_SIZE = 50_000
export const MAX_MAIL_BATCH_SIZE = 1_000_000
export const DEFAULT_MAIL_POLL_INTERVAL_SECONDS = 300
export const MIN_MAIL_POLL_INTERVAL_SECONDS = 60
export const MAX_MAIL_POLL_INTERVAL_SECONDS = 86_400
export const DEFAULT_MAIL_SYNC_WINDOWS = 5_000
export const MAX_MAIL_SYNC_WINDOWS = 1_000_000

export type MailSyncSettings = {
  account_id: string
  sync_enabled: boolean
  batch_size: number
  windows: number
  poll_interval_seconds: number
  failure_threshold?: number
  updated_at: string
}

export type MailContentEgressSettings = {
  body: boolean
  attachments: boolean
  extracted_text: boolean
}

export type MailSensitiveForwardingPolicy = {
  policy_id: string
  source_account_id: string
  delivery_account_id: string
  name: string
  enabled: boolean
  include_message_body: boolean
  include_attachments: boolean
  fixed_recipients: string[]
  minimum_severity: 'low' | 'medium' | 'high' | 'critical'
  subject_template: string
  body_template: string
  max_sends_per_hour: number
  quiet_hours: Record<string, unknown>
  expires_at: string | null
  updated_at: string
}

export type MailSensitiveForwardingPolicyInput = Omit<
  MailSensitiveForwardingPolicy,
  'policy_id' | 'source_account_id' | 'updated_at'
> & {
  policy_id?: string
}

export type MailSensitiveForwardingPolicyListResponse = {
  items: MailSensitiveForwardingPolicy[]
}

export type MailSyncSettingsUpdate = {
  sync_enabled: boolean
  batch_size: number
  windows: number
  poll_interval_seconds: number
  failure_threshold?: number
}

export type MailSyncRunRequest = {
  full_resync?: boolean
  emit_observations?: boolean
}

export type MailSyncStatus = {
  account_id: string
  status: string
  phase: string
  progress_mode: 'none' | 'determinate' | 'indeterminate' | string
  progress_percent: number | null
  processed_messages: number
  estimated_total_messages: number | null
  current_batch_size: number
  failure_threshold?: number
  last_started_at: string | null
  last_updated_at: string | null
  last_completed_at: string | null
  next_run_at: string | null
  last_error_code: string | null
  last_error_message: string | null
  consecutive_failures: number
  last_fetched_messages: number
  last_projected_messages: number
  last_upserted_personas: number
  last_upserted_organizations: number
}

export type MailSyncStatusListResponse = {
  items: MailSyncStatus[]
}

export type MailSyncRunResponse = {
  run_id: string
  account_id: string
  trigger: string
  status: string
  phase: string
  progress_mode: 'none' | 'determinate' | 'indeterminate' | string
  progress_percent: number | null
  processed_messages: number
  estimated_total_messages: number | null
  current_batch_size: number
  fetched_messages: number
  projected_messages: number
  upserted_personas: number
  upserted_organizations: number
  checkpoint_before_present: boolean
  checkpoint_after_present: boolean
  checkpoint_saved: boolean
  failure_reason: { code: string; message: string } | null
  started_at: string
  completed_at: string | null
  next_run_at: string | null
}
