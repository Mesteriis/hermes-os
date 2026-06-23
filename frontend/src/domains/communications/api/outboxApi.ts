import { fetchCommunicationOutboxConnect, undoCommunicationOutboxItemConnect } from './connectCommunications'
import type { CommunicationOutboxItem, CommunicationOutboxStatus, OutboxListResponse } from '../types/communications'

export async function fetchOutboxItems(
  accountId?: string,
  status?: CommunicationOutboxStatus,
  limit = 100,
  cursor?: string | null
): Promise<OutboxListResponse> {
  return fetchCommunicationOutboxConnect(accountId, status, limit, cursor ?? undefined)
}

export async function undoOutboxItem(outboxId: string): Promise<CommunicationOutboxItem> {
  return undoCommunicationOutboxItemConnect(outboxId)
}
