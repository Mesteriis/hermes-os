import {
  fetchCommunicationThreadMessagesConnect,
  fetchCommunicationThreadsConnect,
  translateCommunicationThreadConnect
} from './connectCommunications'
import type { ThreadListResponse, ThreadMessagesResponse } from '../types/communications'
import type { ThreadTranslationResponse } from '../types/multilingual'

export async function fetchThreads(
  accountId?: string,
  limit = 50,
  cursor?: string | null
): Promise<ThreadListResponse> {
  return fetchCommunicationThreadsConnect(accountId, limit, cursor ?? undefined)
}

export async function fetchThreadMessages(
  accountId: string,
  subject: string,
  limit = 50
): Promise<ThreadMessagesResponse> {
  return fetchCommunicationThreadMessagesConnect(accountId, subject, limit)
}

export async function translateThread(
  accountId: string,
  subject: string,
  targetLanguage: string,
  limit = 50
): Promise<ThreadTranslationResponse> {
  return translateCommunicationThreadConnect(accountId, subject, targetLanguage, limit)
}
