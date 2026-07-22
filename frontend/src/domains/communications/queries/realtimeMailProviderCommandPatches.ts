import type { MailProviderCommandDiagnostics } from '../../../shared/mailSync/providerCommandDiagnostics'
import {
	nullableStringValue,
	numberValue,
	isRecord,
	storedEventEnvelope,
	stringValue,
	type MailRealtimePatchQueryClient,
} from './realtimePatchShared'

type AvailableQueryClient = Required<
	Pick<MailRealtimePatchQueryClient, 'getQueriesData' | 'setQueryData'>
>

export function applyMailProviderCommandDiagnosticsRealtimePatch(
	eventData: string,
	queryClient: AvailableQueryClient
): boolean {
	const envelope = storedEventEnvelope(eventData)
	const event = envelope?.event
	const eventType = stringValue(event?.event_type)
	if (!eventType?.startsWith('communication.provider_command.') || !eventType.endsWith('.v1')) {
		return false
	}

	const payload = isRecord(event?.payload) ? event.payload : undefined
	const commandId = stringValue(payload?.command_id)
	const accountId = stringValue(payload?.account_id)
	const status = stringValue(payload?.status)
	if (!commandId || !accountId || !status) return false

	let patched = false
	for (const [queryKey, data] of queryClient.getQueriesData<MailProviderCommandDiagnostics | null>({
		queryKey: ['communications', 'mail', 'provider-command-diagnostics']
	})) {
		if (!data || queryKey[3] !== accountId) continue
		const itemIndex = data.items.findIndex((item) => item.command_id === commandId)
		if (itemIndex < 0) continue

		const current = data.items[itemIndex]
		if (!current) continue
		const statusFilter = stringValue(queryKey[4])
		const updatedAt = stringValue(event?.occurred_at) ?? current.updated_at
		const next = {
			...current,
			status,
			retry_count: numberValue(payload?.retry_count) ?? current.retry_count,
			max_retries: numberValue(payload?.max_retries) ?? current.max_retries,
			reconciliation_status:
				stringValue(payload?.reconciliation_status) ?? current.reconciliation_status,
			next_attempt_at: nullableStringValue(payload?.next_attempt_at),
			dead_lettered_at: nullableStringValue(payload?.dead_lettered_at),
			last_attempt_at: status === 'executing' ? updatedAt : current.last_attempt_at,
			last_error: clearsLastError(status) ? null : current.last_error,
			updated_at: updatedAt
		}
		const items = data.items.slice()
		if (statusFilter && statusFilter !== status) items.splice(itemIndex, 1)
		else items[itemIndex] = next

		queryClient.setQueryData(queryKey, {
			...data,
			items,
			counts: patchCounts(data.counts, current.status, status)
		})
		patched = true
	}

	return patched
}

function clearsLastError(status: string): boolean {
	return status === 'queued' || status === 'executing' || status === 'retrying' || status === 'completed'
}

function patchCounts(
	counts: MailProviderCommandDiagnostics['counts'],
	previousStatus: string,
	nextStatus: string
): MailProviderCommandDiagnostics['counts'] {
	if (previousStatus === nextStatus) return counts
	const next = counts.map((item) => ({ ...item }))
	const previous = next.find((item) => item.status === previousStatus)
	if (previous) previous.count = Math.max(0, previous.count - 1)
	const current = next.find((item) => item.status === nextStatus)
	if (current) current.count += 1
	else next.push({ status: nextStatus, count: 1 })
	return next
}
