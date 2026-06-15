import { ApiClient } from '../../../platform/api/ApiClient'
import type { ThreadListResponse, ThreadMessagesResponse } from '../types/communications'
import type { ThreadTranslationResponse } from '../types/multilingual'

export async function fetchThreads(
  accountId?: string,
  limit = 50,
  cursor?: string | null
): Promise<ThreadListResponse> {
  const params = new URLSearchParams({ limit: String(Math.trunc(limit)) })
  if (accountId?.trim()) params.set('account_id', accountId.trim())
  if (cursor?.trim()) params.set('cursor', cursor.trim())
  return ApiClient.instance.get<ThreadListResponse>(
    `/api/v1/communications/threads?${params.toString()}`,
    'Threads request failed'
  )
}

export async function fetchThreadMessages(
  accountId: string,
  subject: string,
  limit = 50
): Promise<ThreadMessagesResponse> {
  const params = new URLSearchParams({ account_id: accountId, subject, limit: String(Math.trunc(limit)) })
  return ApiClient.instance.get<ThreadMessagesResponse>(
    `/api/v1/communications/threads/messages?${params.toString()}`,
    'Thread messages failed'
  )
}

export async function translateThread(
  accountId: string,
  subject: string,
  targetLanguage: string,
  limit = 50
): Promise<ThreadTranslationResponse> {
  const params = new URLSearchParams({
    account_id: accountId,
    subject,
    limit: String(Math.trunc(limit))
  })
  return ApiClient.instance.post<ThreadTranslationResponse>(
    `/api/v1/communications/threads/translate?${params.toString()}`,
    { target_language: targetLanguage },
    'Thread translation failed'
  )
}
