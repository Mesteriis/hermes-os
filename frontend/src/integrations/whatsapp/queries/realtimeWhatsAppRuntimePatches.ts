import type {
	WhatsAppCallSyncItem,
	WhatsAppChatSyncItem,
	WhatsAppContactSyncItem,
	WhatsAppMembersSyncItem,
	WhatsAppPresenceSyncItem,
	WhatsAppProviderCommand,
	WhatsAppRuntimeStatus,
	WhatsappWebMessage,
	WhatsappWebSession,
} from '../types/whatsapp'
import {
	isRecord,
	storedEventEnvelope,
	stringValue,
} from '../../../shared/communications/queries/realtimePatchShared'
import {
	booleanValue,
	integerValue,
	nullableStringValue,
	stringArray,
	type WhatsAppRuntimeEventPayload,
} from './realtimeWhatsAppRuntimePatchValues'
import {
	patchCallList,
	patchChatsList,
	patchContactsList,
	patchMembersList,
	patchPresenceList,
	patchStatusesList,
} from './realtimeWhatsAppRuntimeSyncPatches'

export type WhatsAppRuntimePatchQueryClient = {
	getQueriesData?: <TData>(filters: { queryKey: readonly unknown[] }) => Array<
		[readonly unknown[], TData | undefined]
	>
	setQueryData?: <TData>(
		queryKey: readonly unknown[],
		updater: TData | ((data: TData | undefined) => TData | undefined)
	) => unknown
}

export function applyWhatsAppRuntimeRealtimePatch(
	eventData: string,
	queryClient: WhatsAppRuntimePatchQueryClient
): boolean {
	const { getQueriesData, setQueryData } = queryClient
	if (!getQueriesData || !setQueryData) return false

	const envelope = storedEventEnvelope(eventData)
	const eventType = stringValue(envelope?.event?.event_type)
	if (!eventType || !eventType.startsWith('whatsapp.')) return false

	const payload = isRecord(envelope?.event?.payload)
		? (envelope.event?.payload as WhatsAppRuntimeEventPayload)
		: undefined
	if (!payload) return false

	let patched = false
	for (const [queryKey, data] of getQueriesData<WhatsappWebSession[]>({
		queryKey: ['integrations', 'whatsapp', 'sessions'],
	})) {
		const updated = patchSessionList(queryKey, data, eventType, payload)
		if (updated !== data) {
			setQueryData(queryKey, updated)
			patched = true
		}
	}

	for (const [queryKey, data] of getQueriesData<WhatsAppRuntimeStatus | null>({
		queryKey: ['integrations', 'whatsapp', 'runtime', 'status'],
	})) {
		const updated = patchRuntimeStatus(queryKey, data, eventType, payload)
		if (updated !== data) {
			setQueryData(queryKey, updated)
			patched = true
		}
	}

	for (const [queryKey, data] of getQueriesData<WhatsAppProviderCommand[]>({
		queryKey: ['integrations', 'whatsapp', 'commands'],
	})) {
		const updated = patchCommandList(queryKey, data, eventType, payload)
		if (updated !== data) {
			setQueryData(queryKey, updated)
			patched = true
		}
	}

	for (const [queryKey, data] of getQueriesData<WhatsAppPresenceSyncItem[]>({
		queryKey: ['integrations', 'whatsapp', 'runtime', 'sync-presence'],
	})) {
		const updated = patchPresenceList(queryKey, data, eventType, payload)
		if (updated !== data) {
			setQueryData(queryKey, updated)
			patched = true
		}
	}

	for (const [queryKey, data] of getQueriesData<WhatsAppCallSyncItem[]>({
		queryKey: ['integrations', 'whatsapp', 'runtime', 'sync-calls'],
	})) {
		const updated = patchCallList(queryKey, data, eventType, payload)
		if (updated !== data) {
			setQueryData(queryKey, updated)
			patched = true
		}
	}

	for (const [queryKey, data] of getQueriesData<WhatsappWebMessage[]>({
		queryKey: ['integrations', 'whatsapp', 'runtime', 'sync-statuses'],
	})) {
		const updated = patchStatusesList(queryKey, data, eventType, payload)
		if (updated !== data) {
			setQueryData(queryKey, updated)
			patched = true
		}
	}

	for (const [queryKey, data] of getQueriesData<WhatsAppChatSyncItem[]>({
		queryKey: ['integrations', 'whatsapp', 'runtime', 'sync-chats'],
	})) {
		const updated = patchChatsList(queryKey, data, eventType, payload)
		if (updated !== data) {
			setQueryData(queryKey, updated)
			patched = true
		}
	}

	for (const [queryKey, data] of getQueriesData<WhatsAppMembersSyncItem[]>({
		queryKey: ['integrations', 'whatsapp', 'runtime', 'sync-members'],
	})) {
		const updated = patchMembersList(queryKey, data, eventType, payload)
		if (updated !== data) {
			setQueryData(queryKey, updated)
			patched = true
		}
	}

	for (const [queryKey, data] of getQueriesData<WhatsAppContactSyncItem[]>({
		queryKey: ['integrations', 'whatsapp', 'runtime', 'sync-contacts'],
	})) {
		const updated = patchContactsList(queryKey, data, eventType, payload)
		if (updated !== data) {
			setQueryData(queryKey, updated)
			patched = true
		}
	}

	return patched
}

function patchSessionList(
	queryKey: readonly unknown[],
	sessions: WhatsappWebSession[] | undefined,
	eventType: string,
	payload: WhatsAppRuntimeEventPayload
): WhatsappWebSession[] | undefined {
	if (!sessions) return sessions
	if (
		eventType !== 'whatsapp.runtime.status_changed' &&
		eventType !== 'whatsapp.session.link_state_changed' &&
		eventType !== 'whatsapp.runtime.event'
	) {
		return sessions
	}

	const payloadAccountId = stringValue(payload.account_id)
	if (!payloadAccountId) return sessions
	const queryAccountId = typeof queryKey[3] === 'string' && queryKey[3] !== 'all' ? queryKey[3] : null
	if (queryAccountId && queryAccountId !== payloadAccountId) return sessions

	let changed = false
	const updatedSessions = sessions.map((session) => {
		if (session.account_id !== payloadAccountId) return session
		const updated = patchSession(session, eventType, payload)
		if (updated !== session) changed = true
		return updated
	})

	return changed ? updatedSessions : sessions
}

function patchSession(
	session: WhatsappWebSession,
	eventType: string,
	payload: WhatsAppRuntimeEventPayload
): WhatsappWebSession {
	if (eventType === 'whatsapp.session.link_state_changed') {
		const linkState = linkStateValue(payload.link_state)
		if (!linkState) return session
		return {
			...session,
			link_state: linkState,
			updated_at: stringValue(payload.occurred_at) ?? session.updated_at,
		}
	}

	if (eventType === 'whatsapp.runtime.status_changed') {
		const status = stringValue(payload.status)
		if (!status) return session
		return {
			...session,
			metadata: {
				...session.metadata,
				runtime_status: status,
				runtime_status_source: stringValue(payload.source),
			},
			updated_at: stringValue(payload.occurred_at) ?? session.updated_at,
		}
	}

	const runtimeStatus = stringValue(payload.runtime_status)
	const lifecycleState = stringValue(payload.lifecycle_state)
	if (!runtimeStatus && !lifecycleState) return session

	return {
		...session,
		link_state: linkStateValue(lifecycleState) ?? session.link_state,
		metadata: {
			...session.metadata,
			...(runtimeStatus ? { runtime_status: runtimeStatus } : {}),
			...(lifecycleState ? { lifecycle_state: lifecycleState } : {}),
			...(stringValue(payload.provider_shape)
				? { provider_shape: stringValue(payload.provider_shape) }
				: {}),
			...(stringValue(payload.runtime_kind)
				? { runtime_kind: stringValue(payload.runtime_kind) }
				: {}),
			...(stringValue(payload.provider_event_id)
				? { provider_event_id: stringValue(payload.provider_event_id) }
				: {}),
		},
		updated_at:
			stringValue(payload.occurred_at) ??
			stringValue(payload.observed_at) ??
			session.updated_at,
	}
}

function patchRuntimeStatus(
	queryKey: readonly unknown[],
	status: WhatsAppRuntimeStatus | null | undefined,
	eventType: string,
	payload: WhatsAppRuntimeEventPayload
): WhatsAppRuntimeStatus | null | undefined {
	if (!status) return status

	const payloadAccountId = stringValue(payload.account_id)
	if (!payloadAccountId) return status
	const queryAccountId =
		typeof queryKey[4] === 'string' && queryKey[4] !== 'none' ? queryKey[4] : null
	if (queryAccountId && queryAccountId !== payloadAccountId) return status
	if (status.account_id !== payloadAccountId) return status

	if (eventType === 'whatsapp.runtime.status_changed') {
		const nextStatus = stringValue(payload.status)
		if (!nextStatus) return status
		return {
			...status,
			provider_kind: stringValue(payload.provider_kind) ?? status.provider_kind,
			provider_shape: stringValue(payload.provider_shape) ?? status.provider_shape,
			runtime_kind: stringValue(payload.runtime_kind) ?? status.runtime_kind,
			status: nextStatus,
			live_runtime_available:
				booleanValue(payload.live_runtime_available) ?? status.live_runtime_available,
			live_send_available:
				booleanValue(payload.live_send_available) ?? status.live_send_available,
			qr_pairing_available:
				booleanValue(payload.qr_pairing_available) ?? status.qr_pairing_available,
			pair_code_available:
				booleanValue(payload.pair_code_available) ?? status.pair_code_available,
			media_download_available:
				booleanValue(payload.media_download_available) ?? status.media_download_available,
			media_upload_available:
				booleanValue(payload.media_upload_available) ?? status.media_upload_available,
			session_restore_available:
				booleanValue(payload.session_restore_available) ?? status.session_restore_available,
			runtime_blockers: stringArray(payload.runtime_blockers) ?? status.runtime_blockers,
			last_error: nullableStringValue(payload.last_error, status.last_error),
			updated_at: status.updated_at,
		}
	}

	if (eventType === 'whatsapp.session.link_state_changed') {
		const linkState = stringValue(payload.link_state)
		if (!linkState) return status
		return {
			...status,
			provider_shape: stringValue(payload.provider_shape) ?? status.provider_shape,
			runtime_kind: stringValue(payload.runtime_kind) ?? status.runtime_kind,
			status: linkState,
		}
	}

	const runtimeStatus = stringValue(payload.runtime_status)
	const lifecycleState = stringValue(payload.lifecycle_state)
	if (!runtimeStatus && !lifecycleState) return status

	return {
		...status,
		status: lifecycleState ?? runtimeStatus ?? status.status,
	}
}

function patchCommandList(
	queryKey: readonly unknown[],
	commands: WhatsAppProviderCommand[] | undefined,
	eventType: string,
	payload: WhatsAppRuntimeEventPayload
): WhatsAppProviderCommand[] | undefined {
	if (!commands || eventType !== 'whatsapp.command.status_changed') return commands

	const payloadAccountId = stringValue(payload.account_id)
	if (!payloadAccountId) return commands
	const queryAccountId =
		typeof queryKey[3] === 'string' && queryKey[3] !== 'none' ? queryKey[3] : null
	if (queryAccountId && queryAccountId !== payloadAccountId) return commands

	const commandId = stringValue(payload.command_id)
	if (!commandId) return commands

	let changed = false
	const updatedCommands = commands.map((command) => {
		if (command.command_id !== commandId) return command
		changed = true
		return {
			...command,
			account_id: payloadAccountId,
			command_kind: stringValue(payload.command_kind) ?? command.command_kind,
			provider_chat_id: stringValue(payload.provider_chat_id) ?? command.provider_chat_id,
			provider_message_id: nullableStringValue(
				payload.provider_message_id,
				command.provider_message_id
			),
			status: stringValue(payload.status) ?? command.status,
			last_error: nullableStringValue(payload.last_error, command.last_error),
			retry_count: integerValue(payload.retry_count) ?? command.retry_count,
			max_retries: integerValue(payload.max_retries) ?? command.max_retries,
			reconciliation_status:
				stringValue(payload.reconciliation_status) ?? command.reconciliation_status,
			next_attempt_at: nullableStringValue(payload.next_attempt_at, command.next_attempt_at),
			last_attempt_at: nullableStringValue(payload.last_attempt_at, command.last_attempt_at),
			provider_observed_at: nullableStringValue(
				payload.provider_observed_at,
				command.provider_observed_at
			),
			reconciled_at: nullableStringValue(payload.reconciled_at, command.reconciled_at),
			dead_lettered_at: nullableStringValue(
				payload.dead_lettered_at,
				command.dead_lettered_at
			),
			completed_at: nullableStringValue(payload.completed_at, command.completed_at),
			updated_at:
				nullableStringValue(payload.completed_at, null) ??
				nullableStringValue(payload.reconciled_at, null) ??
				nullableStringValue(payload.provider_observed_at, null) ??
				nullableStringValue(payload.last_attempt_at, null) ??
				command.updated_at,
		}
	})

	return changed ? updatedCommands : commands
}

function linkStateValue(value: unknown): WhatsappWebSession['link_state'] | null {
	const normalized = stringValue(value)
	if (
		normalized === 'qr_pending' ||
		normalized === 'pair_code_pending' ||
		normalized === 'link_required' ||
		normalized === 'linked' ||
		normalized === 'degraded' ||
		normalized === 'revoked' ||
		normalized === 'removed' ||
		normalized === 'blocked'
	) {
		return normalized
	}
	return null
}
