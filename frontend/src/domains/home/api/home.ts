import { ApiClient } from '../../../platform/api/ApiClient'
import type { CommunicationMessagesResponse } from '../types/api'
import type { MailboxHealth } from '../types/api'

export async function fetchCommunicationMessages(limit = 50): Promise<CommunicationMessagesResponse> {
  const params = new URLSearchParams({ limit: String(Math.trunc(limit)) })
  return ApiClient.instance.get<CommunicationMessagesResponse>(
    `/api/v1/communications/messages?${params.toString()}`,
    'Communication messages request failed'
  )
}

export async function fetchMailboxHealth(): Promise<MailboxHealth> {
  return ApiClient.instance.get<MailboxHealth>(
    '/api/v1/communications/analytics/health',
    'Health request failed'
  )
}
