export type MailSyncSettings = {
  account_id: string
  sync_enabled: boolean
  batch_size: number
  poll_interval_seconds: number
  failure_threshold?: number
  updated_at: string
}

export type MailReadSyncStatus =
  | 'queued'
  | 'syncing'
  | 'retrying'
  | 'failed'
  | 'awaiting_provider'
  | 'synced'
  | 'superseded'

export type MailSyncSettingsUpdate = {
  sync_enabled: boolean
  batch_size: number
  poll_interval_seconds: number
  failure_threshold?: number
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
