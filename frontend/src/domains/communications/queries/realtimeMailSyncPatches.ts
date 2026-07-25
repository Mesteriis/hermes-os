import type { MailSyncStatus } from '../types/communications'
import {
  isRecord,
  nullableNumberValue,
  nullableStringValue,
  numberValue,
  storedEventEnvelope,
	stringValue,
	type MailRealtimePatchQueryClient,
	type SyncPatchPayload,
} from './realtimePatchShared'

type AvailableMailRealtimePatchQueryClient = Required<
  Pick<MailRealtimePatchQueryClient, 'getQueriesData' | 'setQueryData'>
>

export function applySyncRealtimePatch(
  eventData: string,
  queryClient: AvailableMailRealtimePatchQueryClient
): boolean {
  const envelope = storedEventEnvelope(eventData)
  const event = envelope?.event
  const eventType = event?.event_type
  if (
    eventType !== 'mail.sync.started' &&
    eventType !== 'mail.sync.progress' &&
    eventType !== 'mail.sync.completed' &&
    eventType !== 'mail.sync.failed' &&
    eventType !== 'mail.sync.skipped'
  ) {
    return false
  }

	const payload = isRecord(event?.payload) ? event.payload : undefined
  const accountId = stringValue(payload?.account_id)
  if (!accountId) return false

  let patched = false
  for (const [queryKey, data] of queryClient.getQueriesData<MailSyncStatus[]>({
    queryKey: ['communications', 'mail', 'sync-statuses'],
  })) {
    const updated = patchSyncStatuses(data, accountId, payload)
    if (updated !== data) {
      queryClient.setQueryData(queryKey, updated)
      patched = true
    }
  }

  return patched
}

function patchSyncStatuses(
  items: MailSyncStatus[] | undefined,
  accountId: string,
  payload: SyncPatchPayload | undefined
): MailSyncStatus[] | undefined {
  if (!items || !payload) return items

  let changed = false
  const patched = items.map((item) => {
    if (item.account_id !== accountId) return item
    changed = true
    const patchedAt = new Date().toISOString()
    return {
      ...item,
      status: stringValue(payload.status) ?? item.status,
      phase: stringValue(payload.phase) ?? item.phase,
      progress_mode: stringValue(payload.progress_mode) ?? item.progress_mode,
      progress_percent:
        typeof payload.progress_percent === 'undefined'
          ? item.progress_percent
          : nullableNumberValue(payload.progress_percent),
      processed_messages: numberValue(payload.processed_messages) ?? item.processed_messages,
      estimated_total_messages:
        typeof payload.estimated_total_messages === 'undefined'
          ? item.estimated_total_messages
          : nullableNumberValue(payload.estimated_total_messages),
      current_batch_size: numberValue(payload.current_batch_size) ?? item.current_batch_size,
      last_updated_at: patchedAt,
      next_run_at:
        typeof payload.next_run_at === 'undefined'
          ? item.next_run_at
          : nullableStringValue(payload.next_run_at),
      last_error_code:
        typeof payload.error_code === 'undefined'
          ? item.last_error_code
          : nullableStringValue(payload.error_code),
      last_fetched_messages: numberValue(payload.fetched_messages) ?? item.last_fetched_messages,
      last_projected_messages:
        numberValue(payload.projected_messages) ?? item.last_projected_messages,
      last_upserted_personas:
        numberValue(payload.upserted_personas) ?? item.last_upserted_personas,
      last_upserted_organizations:
        numberValue(payload.upserted_organizations) ?? item.last_upserted_organizations,
    }
  })

  return changed ? patched : items
}
