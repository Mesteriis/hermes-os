import { ApiClient } from '../../../platform/api/ApiClient'
import type { EmailOutboxItem, EmailOutboxStatus, OutboxListResponse } from '../types/communications'

export async function fetchOutboxItems(
  accountId?: string,
  status?: EmailOutboxStatus,
  limit = 100,
  cursor?: string | null
): Promise<OutboxListResponse> {
  const params = new URLSearchParams({ limit: String(Math.trunc(limit)) })
  if (accountId?.trim()) params.set('account_id', accountId.trim())
  if (status?.trim()) params.set('status', status.trim())
  if (cursor?.trim()) params.set('cursor', cursor.trim())
  return ApiClient.instance.get<OutboxListResponse>(
    `/api/v1/communications/outbox?${params.toString()}`,
    'Outbox request failed'
  )
}

export async function undoOutboxItem(outboxId: string): Promise<EmailOutboxItem> {
  return ApiClient.instance.post<EmailOutboxItem>(
    `/api/v1/communications/outbox/${encodeURIComponent(outboxId)}/undo`,
    {},
    'Undo send failed'
  )
}
