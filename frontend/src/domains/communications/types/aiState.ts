export type CommunicationAiState =
  | 'NEW'
  | 'PROCESSING'
  | 'PROCESSED'
  | 'REVIEW_REQUIRED'
  | 'FAILED'
  | 'ARCHIVED'

export type CommunicationAiStateRecord = {
  message_id: string
  ai_state: CommunicationAiState
  review_reason: string | null
  last_error: string | null
  retry_count: number
  next_attempt_at: string | null
  processing_lease_expires_at: string | null
  created_at: string
  updated_at: string
}

export type CommunicationAiStateTransitionRequest = {
  ai_state: CommunicationAiState
  review_reason?: string | null
  last_error?: string | null
}
