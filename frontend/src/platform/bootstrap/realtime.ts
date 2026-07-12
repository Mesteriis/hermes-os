import { SseClient, WebSocketClient } from '../sse'
import type {
	SseClientOptions,
	SseMessageEvent,
	SseStatusEvent,
	WebSocketClientOptions,
	WebSocketStatusEvent
} from '../sse'
import type { FrontendConfig } from '../config/env'
import { applyMailRealtimePatch } from '../../domains/communications/queries/realtimeMailPatches'
import { applyWhatsAppRealtimePatch } from '../../domains/communications/queries/realtimeWhatsAppPatches'
import { applyTelegramParticipantRealtimePatch } from '../../domains/communications/queries/realtimeTelegramParticipantPatches'
import { applyTelegramRealtimePatch } from '../../domains/communications/queries/realtimeTelegramPatches'
import { applyTelegramCommandRealtimePatch } from '../../integrations/telegram/queries/realtimeTelegramCommandPatches'
import { applyWhatsAppRuntimeRealtimePatch } from '../../integrations/whatsapp/queries/realtimeWhatsAppRuntimePatches'
import { zoomQueryKeys } from '../../integrations/zoom/queries/zoomQueryKeys'

export type RealtimeClient = {
	connect: () => void
	disconnect: () => void
	reconnect: () => void
}

export type RealtimeQueryClient = {
	invalidateQueries: (filters: { queryKey: readonly unknown[] }) => unknown
	getQueriesData?: <TData>(filters: { queryKey: readonly unknown[] }) => Array<[
		readonly unknown[],
		TData | undefined
	]>
	setQueryData?: <TData>(
		queryKey: readonly unknown[],
		updater: TData | ((data: TData | undefined) => TData | undefined)
	) => unknown
}

export type RealtimeClientOptions = SseClientOptions | WebSocketClientOptions
export type RealtimeClientFactory = (options: RealtimeClientOptions) => RealtimeClient
export type RealtimeStatusHandler = (status: SseStatusEvent | WebSocketStatusEvent) => void

export type RealtimeBootstrapOptions = {
	createClient?: RealtimeClientFactory
	onEventObserved?: (eventId: string) => void
	onLaggedObserved?: (skipped: number) => void
	onStatus?: RealtimeStatusHandler
}

const REALTIME_CURSOR_STORAGE_KEY = 'hermes.realtime.lastEventId'

const REALTIME_QUERY_KEYS: readonly (readonly unknown[])[] = [
	['communications-list'],
	['communications-state-counts'],
	['communications-drafts'],
	['communications-outbox'],
	['communications-threads'],
	['communications-message'],
	['communications-ai-state'],
	['communications-saved-searches'],
	['communications-folders'],
	['communications-folder-messages'],
	['communications-attachment-search']
]

const MAIL_RUNTIME_QUERY_KEYS: readonly (readonly unknown[])[] = [
	['communications', 'mail', 'sync-statuses'],
	['communications', 'mail', 'mailbox-health']
]

const TELEGRAM_QUERY_KEYS: readonly (readonly unknown[])[] = [
	['integrations', 'telegram', 'capabilities'],
	['integrations', 'telegram', 'accounts'],
	['communications', 'telegram', 'chats'],
	['communications', 'telegram', 'folders'],
	['communications', 'telegram', 'messages'],
	['integrations', 'telegram', 'runtime'],
	['communications', 'telegram', 'calls']
]

const WHATSAPP_QUERY_KEYS: readonly (readonly unknown[])[] = [
	['integrations', 'whatsapp', 'capabilities'],
	['integrations', 'whatsapp', 'account-capabilities'],
	['integrations', 'whatsapp', 'sessions'],
	['integrations', 'whatsapp', 'runtime', 'status'],
	['integrations', 'whatsapp', 'runtime', 'health'],
	['integrations', 'whatsapp', 'commands'],
	['integrations', 'whatsapp', 'runtime', 'sync-chats'],
	['integrations', 'whatsapp', 'runtime', 'sync-history'],
	['integrations', 'whatsapp', 'runtime', 'sync-members'],
	['integrations', 'whatsapp', 'runtime', 'sync-statuses'],
	['integrations', 'whatsapp', 'runtime', 'sync-presence'],
	['integrations', 'whatsapp', 'runtime', 'sync-calls'],
	['integrations', 'whatsapp', 'runtime', 'sync-contacts'],
	['integrations', 'whatsapp', 'runtime', 'sync-media'],
	['communications', 'whatsapp', 'conversations'],
	['communications', 'whatsapp', 'conversation-detail'],
	['communications', 'whatsapp', 'chat-members'],
	['communications', 'whatsapp', 'messages']
]

const ZOOM_QUERY_KEYS: readonly (readonly unknown[])[] = [
	zoomQueryKeys.accounts,
	zoomQueryKeys.capabilities,
	zoomQueryKeys.runtimeStatus,
	zoomQueryKeys.webhookSubscriptions,
	zoomQueryKeys.providerCalls,
	zoomQueryKeys.callTranscript,
	zoomQueryKeys.recordingImports,
	zoomQueryKeys.auditEvents
]

const SIGNAL_HUB_QUERY_KEYS: readonly (readonly unknown[])[] = [
	['signal-hub']
]

export function initializeRealtime(
	config: FrontendConfig,
	queryClient: RealtimeQueryClient,
	options: RealtimeClientFactory | RealtimeBootstrapOptions = {}
): RealtimeClient {
	const bootstrapOptions = normalizeRealtimeBootstrapOptions(options)
	const createClient: RealtimeClientFactory =
		bootstrapOptions.createClient ??
		((clientOptions) =>
			adaptRealtimeClient(
				isWebSocketClientOptions(clientOptions)
					? new WebSocketClient(clientOptions)
					: new SseClient(clientOptions)
			))

	const clientOptions = realtimeClientOptions(
		config,
		queryClient,
		bootstrapOptions.onEventObserved,
		bootstrapOptions.onLaggedObserved,
		bootstrapOptions.onStatus
	)
	const createSseClient = (): RealtimeClient => createClient(clientOptions.sse)

	if (config.realtimeTransport !== 'websocket') {
		const client = createSseClient()
		client.connect()
		return {
			connect: () => client.connect(),
			disconnect: () => client.disconnect(),
			reconnect: () => {
				client.disconnect()
				client.connect()
			}
		}
	}

	let sseFallbackClient: RealtimeClient | null = null
	let disconnected = false
	let reconnecting = false
	const webSocketClient = createClient({
		...clientOptions.webSocket,
		onStatus: (status: WebSocketStatusEvent) => {
			bootstrapOptions.onStatus?.(status)
			if (
				status.state === 'disconnected' &&
				!disconnected &&
				!reconnecting &&
				!sseFallbackClient
			) {
				sseFallbackClient = createSseClient()
				sseFallbackClient.connect()
			}
		}
	})
	webSocketClient.connect()

	return {
		connect: () => {
			disconnected = false
			if (sseFallbackClient) {
				sseFallbackClient.connect()
				return
			}
			webSocketClient.connect()
		},
		disconnect: () => {
			disconnected = true
			webSocketClient.disconnect()
			sseFallbackClient?.disconnect()
		},
		reconnect: () => {
			reconnecting = true
			disconnected = true
			sseFallbackClient?.disconnect()
			sseFallbackClient = null
			webSocketClient.disconnect()
			disconnected = false
			reconnecting = false
			webSocketClient.connect()
		}
	}
}

function realtimeClientOptions(
	config: FrontendConfig,
	queryClient: RealtimeQueryClient,
	onEventObserved?: (eventId: string) => void,
	onLaggedObserved?: (skipped: number) => void,
	onStatus?: RealtimeStatusHandler
): { sse: SseClientOptions; webSocket: WebSocketClientOptions } {
	const realtimeQueryClient = bindRealtimeQueryClient(queryClient)
	const common = {
		secret: config.apiSecret,
		lastEventId: readRealtimeCursor(),
		onMessage: (event: SseMessageEvent) => {
			if (event.event === 'lagged') {
				onLaggedObserved?.(laggedSkippedCount(event.data))
				handleRealtimeEvent(event, realtimeQueryClient)
				return
			}

			persistRealtimeCursor(event.id)
			onEventObserved?.(event.id)
			handleRealtimeEvent(event, realtimeQueryClient)
		}
	}

	return {
		sse: {
			...common,
			url: config.sseUrl,
			longPollUrl: `${config.apiBaseUrl}/api/v1/events`,
			onError: (error) => {
				console.warn('[Realtime] SSE stream unavailable', error)
			},
			onStatus
		},
		webSocket: {
			...common,
			url: config.webSocketUrl,
			onError: (error) => {
				console.warn('[Realtime] WebSocket stream unavailable', error)
			},
			onStatus
		}
	}
}

function bindRealtimeQueryClient(queryClient: RealtimeQueryClient): RealtimeQueryClient {
	const boundClient: RealtimeQueryClient = {
		invalidateQueries: (filters) => queryClient.invalidateQueries(filters)
	}

	if (queryClient.getQueriesData) {
		boundClient.getQueriesData = <TData>(filters: { queryKey: readonly unknown[] }) =>
			queryClient.getQueriesData?.<TData>(filters) ?? []
	}

	if (queryClient.setQueryData) {
		boundClient.setQueryData = <TData>(
			queryKey: readonly unknown[],
			updater: TData | ((data: TData | undefined) => TData | undefined)
		) => queryClient.setQueryData?.<TData>(queryKey, updater)
	}

	return boundClient
}

function normalizeRealtimeBootstrapOptions(
	options: RealtimeClientFactory | RealtimeBootstrapOptions
): RealtimeBootstrapOptions {
	if (typeof options === 'function') {
		return { createClient: options }
	}

	return options
}

function isWebSocketClientOptions(
	options: RealtimeClientOptions
): options is WebSocketClientOptions {
	return options.url.includes('/api/events/ws')
}

function adaptRealtimeClient(client: { connect: () => void; disconnect: () => void }): RealtimeClient {
	return {
		connect: () => client.connect(),
		disconnect: () => client.disconnect(),
		reconnect: () => {
			client.disconnect()
			client.connect()
		}
	}
}

export function handleRealtimeEvent(
	event: SseMessageEvent,
	queryClient: RealtimeQueryClient
): void {
	if (event.event === 'heartbeat') return
	if (event.event === 'error') return
	if (event.event === 'lagged') {
		for (const queryKey of laggedRealtimeQueryKeys()) {
			void queryClient.invalidateQueries({ queryKey })
		}
		return
	}

	applyMailRealtimePatch(event.data, queryClient)
	applyWhatsAppRealtimePatch(event.data, queryClient)
	applyWhatsAppRuntimeRealtimePatch(event.data, queryClient)
	applyTelegramRealtimePatch(event.data, queryClient)
	applyTelegramParticipantRealtimePatch(event.data, queryClient)
	applyTelegramCommandRealtimePatch(event.data, queryClient)

	for (const queryKey of queryKeysForRealtimeEvent(event)) {
		void queryClient.invalidateQueries({ queryKey })
	}
}

function laggedRealtimeQueryKeys(): readonly (readonly unknown[])[] {
	return [
		...REALTIME_QUERY_KEYS,
		...MAIL_RUNTIME_QUERY_KEYS,
		...TELEGRAM_QUERY_KEYS,
		...WHATSAPP_QUERY_KEYS,
		...ZOOM_QUERY_KEYS,
		...SIGNAL_HUB_QUERY_KEYS
	]
}

function queryKeysForRealtimeEvent(event: SseMessageEvent): readonly (readonly unknown[])[] {
	const eventType = canonicalEventType(event.data)
	if (!eventType) return REALTIME_QUERY_KEYS

	if (eventType.startsWith('signal.')) {
		return SIGNAL_HUB_QUERY_KEYS
	}
	if (eventType.startsWith('communication.provider_command.')) {
		return [['communications', 'mail', 'provider-command-diagnostics']]
	}

	if (eventType === 'mail.ai_state.changed') {
		return [['communications-ai-state'], ['communications-message'], ['communications-list']]
	}
	if (eventType === 'mail.read_receipt.recorded') {
		return [['communications-outbox'], ['communications-message'], ['communications-list']]
	}
	if (eventType.startsWith('mail.outbox.')) {
		return [['communications-outbox'], ['communications-list']]
	}
	if (eventType.startsWith('mail.sync.')) {
		return [['communications', 'mail', 'sync-statuses']]
	}
	if (eventType.startsWith('mail.message.')) {
		return [
			['communications-message'],
			['communications-list'],
			['communications-state-counts'],
			['communications-threads'],
			['communications-saved-searches'],
			['communications-folders'],
			['communications-folder-messages']
		]
	}
	if (eventType.startsWith('mail.draft.')) {
		return [['communications-drafts']]
	}
	if (eventType.startsWith('mail.saved_search.')) {
		return [['communications-saved-searches']]
	}
	if (eventType.startsWith('mail.folder_message.')) {
		return [
			['communications-folders'],
			['communications-folder-messages'],
			['communications-list']
		]
	}
	if (eventType.startsWith('mail.folder.')) {
		return [['communications-folders'], ['communications-folder-messages']]
	}
	if (eventType.startsWith('telegram.sync.')) {
		return [['communications', 'telegram', 'chats'], ['communications', 'telegram', 'messages'], ['integrations', 'telegram', 'runtime']]
	}
	if (eventType.startsWith('telegram.message.')) {
		return [['communications', 'telegram', 'messages'], ['communications', 'telegram', 'chats']]
	}
	if (eventType.startsWith('telegram.typing.')) {
		return [['communications', 'telegram', 'chats'], ['integrations', 'telegram', 'runtime']]
	}
	if (eventType.startsWith('telegram.topic.')) {
		return [['communications', 'telegram', 'topics'], ['communications', 'telegram', 'topic-search'], ['communications', 'telegram', 'topic-messages']]
	}
	if (eventType.startsWith('telegram.participant.')) {
		return [['communications', 'telegram', 'chat-members'], ['communications', 'telegram', 'chats']]
	}
	if (eventType.startsWith('telegram.folders.')) {
		return [['communications', 'telegram', 'folders'], ['communications', 'telegram', 'chats']]
	}
	if (eventType.startsWith('telegram.media.upload.')) {
		return [['integrations', 'telegram', 'commands'], ['integrations', 'telegram', 'runtime']]
	}
	if (eventType.startsWith('telegram.media.download.')) {
		return [['communications', 'telegram', 'messages'], ['communications', 'telegram', 'search', 'media']]
	}
	if (eventType.startsWith('telegram.reaction.')) {
		return [['communications', 'telegram', 'messages']]
	}
	if (eventType.startsWith('telegram.command.')) {
		return [['communications', 'telegram', 'messages'], ['integrations', 'telegram', 'runtime'], ['integrations', 'telegram', 'commands']]
	}
	if (eventType.startsWith('telegram.')) {
		return TELEGRAM_QUERY_KEYS
	}
	if (eventType.startsWith('zoom.')) {
		return ZOOM_QUERY_KEYS
	}
	if (eventType.startsWith('whatsapp.sync.')) {
		return [
			['integrations', 'whatsapp', 'runtime', 'sync-chats'],
			['integrations', 'whatsapp', 'runtime', 'sync-history'],
			['integrations', 'whatsapp', 'runtime', 'sync-members'],
			['integrations', 'whatsapp', 'runtime', 'sync-statuses'],
			['integrations', 'whatsapp', 'runtime', 'sync-presence'],
			['integrations', 'whatsapp', 'runtime', 'sync-calls'],
			['integrations', 'whatsapp', 'runtime', 'sync-contacts'],
			['integrations', 'whatsapp', 'runtime', 'sync-media'],
			['communications', 'whatsapp', 'conversations'],
			['communications', 'whatsapp', 'messages'],
			['integrations', 'whatsapp', 'sessions'],
			['integrations', 'whatsapp', 'runtime', 'status'],
			['integrations', 'whatsapp', 'runtime', 'health'],
		]
	}
	if (eventType === 'whatsapp.dialog.updated') {
		return [
			['communications', 'whatsapp', 'conversations'],
			['communications', 'whatsapp', 'conversation-detail'],
			['communications', 'whatsapp', 'messages']
		]
	}
	if (
		eventType.startsWith('whatsapp.message.') ||
		eventType === 'whatsapp.reaction.changed' ||
		eventType === 'whatsapp.receipt.changed'
	) {
		return [['communications', 'whatsapp', 'messages']]
	}
	if (
		eventType === 'whatsapp.participant.changed' ||
		eventType === 'whatsapp.presence.changed' ||
		eventType === 'whatsapp.call.updated' ||
		eventType === 'whatsapp.status.updated' ||
		eventType === 'whatsapp.status.deleted'
	) {
		return [
			['communications', 'whatsapp', 'conversations'],
			['communications', 'whatsapp', 'conversation-detail'],
			['communications', 'whatsapp', 'chat-members'],
				...(eventType === 'whatsapp.participant.changed'
					? [[
							'integrations',
							'whatsapp',
							'runtime',
							'sync-contacts',
					  ] as const]
					: []),
				...(eventType === 'whatsapp.presence.changed'
					? [[
							'integrations',
							'whatsapp',
							'runtime',
							'sync-presence',
					  ] as const]
					: []),
				...(eventType === 'whatsapp.call.updated'
					? [[
							'integrations',
							'whatsapp',
							'runtime',
							'sync-calls',
					  ] as const]
					: []),
				...(eventType === 'whatsapp.status.updated' || eventType === 'whatsapp.status.deleted'
					? [[
							'integrations',
							'whatsapp',
							'runtime',
							'sync-statuses',
					  ] as const]
					: []),
		]
	}
	if (
		eventType === 'whatsapp.runtime.status_changed' ||
		eventType === 'whatsapp.session.link_state_changed' ||
		eventType === 'whatsapp.runtime.event'
	) {
		return [
			['integrations', 'whatsapp', 'sessions'],
			['integrations', 'whatsapp', 'capabilities'],
			['integrations', 'whatsapp', 'account-capabilities'],
			['integrations', 'whatsapp', 'runtime', 'status'],
			['integrations', 'whatsapp', 'runtime', 'health'],
		]
	}
	if (
		eventType.startsWith('whatsapp.command.') ||
		eventType.startsWith('whatsapp.media.upload.') ||
		eventType.startsWith('whatsapp.media.download.')
	) {
		return [
			['integrations', 'whatsapp', 'commands'],
			['integrations', 'whatsapp', 'sessions'],
			['integrations', 'whatsapp', 'capabilities'],
			['integrations', 'whatsapp', 'account-capabilities'],
			['integrations', 'whatsapp', 'runtime', 'status'],
			['integrations', 'whatsapp', 'runtime', 'health'],
				...(eventType.startsWith('whatsapp.media.')
					? [[
							'integrations',
							'whatsapp',
							'runtime',
							'sync-media',
					  ] as const]
					: []),
		]
	}
	if (eventType.startsWith('whatsapp.')) {
		return WHATSAPP_QUERY_KEYS
	}

	return [...REALTIME_QUERY_KEYS, ...MAIL_RUNTIME_QUERY_KEYS]
}

function canonicalEventType(data: string): string | null {
	try {
		const parsed = JSON.parse(data) as { event?: { event_type?: unknown } }
		const eventType = parsed.event?.event_type
		return typeof eventType === 'string' && eventType.trim() ? eventType : null
	} catch {
		return null
	}
}

function laggedSkippedCount(data: string): number {
	try {
		const parsed = JSON.parse(data) as { skipped?: unknown }
		return typeof parsed.skipped === 'number' && parsed.skipped > 0
			? Math.floor(parsed.skipped)
			: 0
	} catch {
		return 0
	}
}

function readRealtimeCursor(): string | undefined {
	try {
		return globalThis.localStorage?.getItem(REALTIME_CURSOR_STORAGE_KEY)?.trim() || undefined
	} catch {
		return undefined
	}
}

function persistRealtimeCursor(eventId: string): void {
	const cursor = eventId.trim()
	if (!cursor) return

	try {
		const currentCursor =
			globalThis.localStorage?.getItem(REALTIME_CURSOR_STORAGE_KEY)?.trim() || ''
		if (isOlderNumericCursor(cursor, currentCursor)) return

		globalThis.localStorage?.setItem(REALTIME_CURSOR_STORAGE_KEY, cursor)
	} catch {
		// Cursor persistence is an offline recovery aid; stream delivery remains authoritative.
	}
}

function isOlderNumericCursor(candidate: string, current: string): boolean {
	if (!isUnsignedIntegerString(candidate) || !isUnsignedIntegerString(current)) {
		return false
	}

	return compareUnsignedIntegerStrings(candidate, current) < 0
}

function isUnsignedIntegerString(value: string): boolean {
	return /^\d+$/.test(value)
}

function compareUnsignedIntegerStrings(left: string, right: string): number {
	const normalizedLeft = left.replace(/^0+/, '') || '0'
	const normalizedRight = right.replace(/^0+/, '') || '0'

	if (normalizedLeft.length !== normalizedRight.length) {
		return normalizedLeft.length - normalizedRight.length
	}

	return normalizedLeft.localeCompare(normalizedRight)
}
