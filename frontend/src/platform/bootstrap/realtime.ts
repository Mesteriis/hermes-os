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
import { applyTelegramParticipantRealtimePatch } from '../../domains/telegram/queries/realtimeTelegramParticipantPatches'
import { applyTelegramRealtimePatch } from '../../domains/telegram/queries/realtimeTelegramPatches'

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
	['communications-mail-list'],
	['communications-state-counts'],
	['communications-sync-statuses'],
	['communications-drafts'],
	['communications-outbox'],
	['communications-threads'],
	['communications-mailbox-health'],
	['communications-message'],
	['communications-ai-state'],
	['communications-saved-searches'],
	['communications-folders'],
	['communications-folder-messages'],
	['communications-attachment-search']
]

const TELEGRAM_QUERY_KEYS: readonly (readonly unknown[])[] = [
	['telegram', 'capabilities'],
	['telegram', 'accounts'],
	['telegram', 'chats'],
	['telegram', 'folders'],
	['telegram', 'messages'],
	['telegram', 'runtime'],
	['telegram', 'calls']
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
	const common = {
		secret: config.apiSecret,
		lastEventId: readRealtimeCursor(),
		onMessage: (event: SseMessageEvent) => {
			if (event.event === 'lagged') {
				onLaggedObserved?.(laggedSkippedCount(event.data))
				handleRealtimeEvent(event, queryClient)
				return
			}

			persistRealtimeCursor(event.id)
			onEventObserved?.(event.id)
			handleRealtimeEvent(event, queryClient)
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
	applyTelegramRealtimePatch(event.data, queryClient)
	applyTelegramParticipantRealtimePatch(event.data, queryClient)

	for (const queryKey of queryKeysForRealtimeEvent(event)) {
		void queryClient.invalidateQueries({ queryKey })
	}
}

function laggedRealtimeQueryKeys(): readonly (readonly unknown[])[] {
	return [...REALTIME_QUERY_KEYS, ...TELEGRAM_QUERY_KEYS]
}

function queryKeysForRealtimeEvent(event: SseMessageEvent): readonly (readonly unknown[])[] {
	const eventType = canonicalEventType(event.data)
	if (!eventType) return REALTIME_QUERY_KEYS

	if (eventType === 'mail.ai_state.changed') {
		return [['communications-ai-state'], ['communications-message'], ['communications-mail-list']]
	}
	if (eventType === 'mail.read_receipt.recorded') {
		return [['communications-outbox'], ['communications-message'], ['communications-mail-list']]
	}
	if (eventType.startsWith('mail.outbox.')) {
		return [['communications-outbox'], ['communications-mail-list']]
	}
	if (eventType.startsWith('mail.sync.')) {
		return [['communications-sync-statuses']]
	}
	if (eventType.startsWith('mail.message.')) {
		return [
			['communications-message'],
			['communications-mail-list'],
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
			['communications-mail-list']
		]
	}
	if (eventType.startsWith('mail.folder.')) {
		return [['communications-folders'], ['communications-folder-messages']]
	}
	if (eventType.startsWith('telegram.sync.')) {
		return [['telegram', 'chats'], ['telegram', 'messages'], ['telegram', 'runtime']]
	}
	if (eventType.startsWith('telegram.message.')) {
		return [['telegram', 'messages'], ['telegram', 'chats']]
	}
	if (eventType.startsWith('telegram.typing.')) {
		return [['telegram', 'chats'], ['telegram', 'runtime']]
	}
	if (eventType.startsWith('telegram.topic.')) {
		return [['telegram', 'topics'], ['telegram', 'topic-search'], ['telegram', 'topic-messages']]
	}
	if (eventType.startsWith('telegram.participant.')) {
		return [['telegram', 'chat-members'], ['telegram', 'chats']]
	}
	if (eventType.startsWith('telegram.folders.')) {
		return [['telegram', 'folders'], ['telegram', 'chats']]
	}
	if (eventType.startsWith('telegram.media.upload.')) {
		return [['telegram', 'commands'], ['telegram', 'runtime']]
	}
	if (eventType.startsWith('telegram.media.download.')) {
		return [['telegram', 'messages'], ['telegram', 'search', 'media']]
	}
	if (eventType.startsWith('telegram.reaction.')) {
		return [['telegram', 'messages']]
	}
	if (eventType.startsWith('telegram.command.')) {
		return [['telegram', 'messages'], ['telegram', 'runtime'], ['telegram', 'commands']]
	}
	if (eventType.startsWith('telegram.')) {
		return TELEGRAM_QUERY_KEYS
	}

	return REALTIME_QUERY_KEYS
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
