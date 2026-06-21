import type { LocalMessageState, WorkflowState } from './communications'

export type CommunicationSavedSearch = {
  saved_search_id: string
  name: string
  description: string | null
  account_id: string | null
  query: string
  workflow_state: WorkflowState | null
  local_state: LocalMessageState
  channel_kind: string | null
  is_smart_folder: boolean
  sort_order: number
  message_count: number
  created_at: string
  updated_at: string
}

export type SavedSearchListResponse = {
  items: CommunicationSavedSearch[]
  next_cursor: string | null
  has_more: boolean
}

export type SavedSearchInput = {
  name: string
  description?: string | null
  account_id?: string | null
  query?: string
  workflow_state?: WorkflowState | null
  local_state?: LocalMessageState
  channel_kind?: string | null
  is_smart_folder?: boolean
  sort_order?: number
}

export type SavedSearchUpdate = Partial<SavedSearchInput>

export type SavedSearchDeleteResponse = {
  deleted: boolean
}
