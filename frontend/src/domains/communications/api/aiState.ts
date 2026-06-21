import { ApiClient } from '../../../platform/api/ApiClient'
import type {
  CommunicationAiStateRecord,
  CommunicationAiStateTransitionRequest
} from '../types/aiState'

export async function fetchMessageAiState(messageId: string): Promise<CommunicationAiStateRecord> {
  return ApiClient.instance.get<CommunicationAiStateRecord>(
    `/api/v1/communications/messages/${encodeURIComponent(messageId)}/ai-state`,
    'Message AI state request failed'
  )
}

export async function updateMessageAiState(
  messageId: string,
  request: CommunicationAiStateTransitionRequest
): Promise<CommunicationAiStateRecord> {
  return ApiClient.instance.put<CommunicationAiStateRecord>(
    `/api/v1/communications/messages/${encodeURIComponent(messageId)}/ai-state`,
    request,
    'Message AI state update failed'
  )
}
