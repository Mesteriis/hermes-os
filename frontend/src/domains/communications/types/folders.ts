import type { LocalMessageState, WorkflowState } from './communications'

export type CommunicationFolder = {
  folder_id: string
  account_id: string | null
  name: string
  description: string | null
  color: string | null
  sort_order: number
  message_count: number
  created_at: string
  updated_at: string
}

export type CommunicationFolderListResponse = {
  items: CommunicationFolder[]
  next_cursor: string | null
  has_more: boolean
}

export type CommunicationFolderInput = {
  folder_id?: string
  account_id?: string | null
  name: string
  description?: string | null
  color?: string | null
  sort_order?: number
}

export type CommunicationFolderUpdate = Partial<CommunicationFolderInput>

export type FolderDeleteResponse = {
  deleted: boolean
}

export type FolderMessageOperation = 'copy' | 'move'

export type FolderMessageActionResponse = {
  operation: FolderMessageOperation
  folder_id: string
  message_id: string
  message: FolderMessage
}

export type FolderMessage = {
  folder_id: string
  message_id: string
  account_id: string
  subject: string
  sender: string
  occurred_at: string | null
  projected_at: string
  workflow_state: WorkflowState
  local_state: LocalMessageState
  added_at: string
  attachment_count: number
}

export type FolderMessageListResponse = {
  items: FolderMessage[]
  next_cursor: string | null
  has_more: boolean
}
