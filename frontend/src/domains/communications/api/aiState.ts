import { ApiClient } from '../../../platform/api/ApiClient'
import type {
  MailAiStateRecord,
  MailAiStateTransitionRequest
} from '../types/aiState'

export async function fetchMessageAiState(messageId: string): Promise<MailAiStateRecord> {
  return ApiClient.instance.get<MailAiStateRecord>(
    `/api/v1/communications/messages/${encodeURIComponent(messageId)}/ai-state`,
    'Message AI state request failed'
  )
}

export async function updateMessageAiState(
  messageId: string,
  request: MailAiStateTransitionRequest
): Promise<MailAiStateRecord> {
  return ApiClient.instance.put<MailAiStateRecord>(
    `/api/v1/communications/messages/${encodeURIComponent(messageId)}/ai-state`,
    request,
    'Message AI state update failed'
  )
}
