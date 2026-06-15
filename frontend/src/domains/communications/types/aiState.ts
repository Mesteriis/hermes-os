export type MailAiState =
  | 'NEW'
  | 'PROCESSING'
  | 'PROCESSED'
  | 'REVIEW_REQUIRED'
  | 'FAILED'
  | 'ARCHIVED'

export type MailAiStateRecord = {
  message_id: string
  ai_state: MailAiState
  review_reason: string | null
  last_error: string | null
  created_at: string
  updated_at: string
}

export type MailAiStateTransitionRequest = {
  ai_state: MailAiState
  review_reason?: string | null
  last_error?: string | null
}
