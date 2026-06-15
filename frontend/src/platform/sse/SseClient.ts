export type SseEventHandler = (event: SseMessageEvent) => void
export type SseErrorHandler = (error: unknown) => void
export type SseTransport = 'sse' | 'long_poll'
export type SseConnectionState =
	| 'connecting'
	| 'connected'
	| 'reconnecting'
	| 'fallback'
	| 'disconnected'
export type SseStatusEvent = {
	transport: SseTransport
	state: SseConnectionState
	attempt?: number
	maxAttempts?: number
	error?: string
}
export type SseStatusHandler = (status: SseStatusEvent) => void

export type SseMessageEvent = {
	id: string
	event: string
	data: string
}

export interface SseClientOptions {
	url: string
	longPollUrl?: string
	secret: string
	lastEventId?: string
	onMessage?: SseEventHandler
	onError?: SseErrorHandler
	onStatus?: SseStatusHandler
	reconnectDelay?: number
	maxReconnectAttempts?: number
	longPollDelay?: number
	longPollBatchSize?: number
	longPollWaitSeconds?: number
	fetchImpl?: typeof fetch
}

type LongPollEventItem = {
	position: number | string
	event: unknown
}

type LongPollResponse = {
	items?: LongPollEventItem[]
	next_after_position?: number
	has_more?: boolean
}

/**
 * Fetch-based SSE client. Browser EventSource cannot send X-Hermes-Secret,
 * so protected local streams must be consumed through a readable fetch body.
 */
export class SseClient {
	private url: string
	private longPollUrl?: string
	private secret: string
	private lastEventId: string
	private onMessage?: SseEventHandler
	private onError?: SseErrorHandler
	private reconnectDelay: number
	private maxReconnectAttempts: number
	private longPollDelay: number
	private longPollBatchSize: number
	private longPollWaitSeconds: number
	private currentTransport: SseTransport = 'sse'
	private reconnectAttempts = 0
	private shouldReconnect = true
	private longPollRunning = false
	private abortController: AbortController | null = null
	private reconnectTimer: ReturnType<typeof setTimeout> | null = null
	private buffer = ''
	private fetchImpl: typeof fetch
	private onStatus?: SseStatusHandler

	constructor(options: SseClientOptions) {
		this.url = options.url
		this.longPollUrl = options.longPollUrl
		this.secret = options.secret.trim()
		if (!this.secret) {
			throw new Error('X-Hermes-Secret cannot be empty')
		}
		this.lastEventId = options.lastEventId?.trim() ?? ''
		this.onMessage = options.onMessage
		this.onError = options.onError
		this.onStatus = options.onStatus
		this.reconnectDelay = options.reconnectDelay ?? 3000
		this.maxReconnectAttempts = options.maxReconnectAttempts ?? 10
		this.longPollDelay = options.longPollDelay ?? 3000
		this.longPollBatchSize = options.longPollBatchSize ?? 100
		this.longPollWaitSeconds = options.longPollWaitSeconds ?? 15
		this.fetchImpl = options.fetchImpl ?? globalThis.fetch.bind(globalThis)
	}

	connect(): void {
		this.stopCurrentConnection()
		this.currentTransport = 'sse'
		this.shouldReconnect = true
		this.reconnectAttempts = 0
		this.reportStatus({ transport: 'sse', state: 'connecting' })
		void this.connectOnce()
	}

	disconnect(): void {
		this.shouldReconnect = false
		this.stopCurrentConnection()
		this.reportStatus({ transport: this.currentTransport, state: 'disconnected' })
	}

	private async connectOnce(): Promise<void> {
		this.abortController = new AbortController()
		try {
			const response = await this.fetchImpl(this.replayUrl(), {
				method: 'GET',
				headers: this.sseHeaders(),
				cache: 'no-store',
				signal: this.abortController.signal
			})

			if (!response.ok) {
				throw new Error(`SSE connection failed with HTTP ${response.status}`)
			}
			if (!response.body) {
				throw new Error('SSE response body is unavailable')
			}

			this.currentTransport = 'sse'
			this.reportStatus({ transport: 'sse', state: 'connected' })
			await this.readSseBody(response.body)
			if (this.shouldReconnect) {
				this.scheduleReconnect()
			}
		} catch (error) {
			if (!this.shouldReconnect) return
			this.onError?.(error)
			this.scheduleReconnect(error)
		}
	}

	private async readSseBody(body: ReadableStream<Uint8Array>): Promise<void> {
		const reader = body.getReader()
		const decoder = new TextDecoder()

		try {
			while (this.shouldReconnect) {
				const { done, value } = await reader.read()
				if (done) break
				this.buffer += decoder.decode(value, { stream: true })
				this.dispatchBufferedEvents()
			}

			this.buffer += decoder.decode()
			this.dispatchBufferedEvents(true)
		} finally {
			reader.releaseLock()
		}
	}

	private dispatchBufferedEvents(final = false): void {
		this.buffer = this.buffer.replaceAll('\r\n', '\n').replaceAll('\r', '\n')
		let boundary = this.buffer.indexOf('\n\n')
		while (boundary !== -1) {
			const block = this.buffer.slice(0, boundary)
			this.buffer = this.buffer.slice(boundary + 2)
			this.dispatchEventBlock(block)
			boundary = this.buffer.indexOf('\n\n')
		}

		if (final && this.buffer.trim()) {
			this.dispatchEventBlock(this.buffer)
			this.buffer = ''
		}
	}

	private dispatchEventBlock(block: string): void {
		let id = ''
		let event = 'message'
		const data: string[] = []

		for (const line of block.split('\n')) {
			if (!line || line.startsWith(':')) continue
			const separator = line.indexOf(':')
			const field = separator === -1 ? line : line.slice(0, separator)
			let value = separator === -1 ? '' : line.slice(separator + 1)
			if (value.startsWith(' ')) value = value.slice(1)

			if (field === 'id') {
				id = value
			} else if (field === 'event') {
				event = value || 'message'
			} else if (field === 'data') {
				data.push(value)
			}
		}

		if (id) {
			this.lastEventId = id
		}
		if (!id && event === 'message' && data.length === 0) {
			return
		}

		this.reconnectAttempts = 0
		this.onMessage?.({
			id,
			event,
			data: data.join('\n')
		})
	}

	private replayUrl(): string {
		if (!this.lastEventId) return this.url
		const parsed = new URL(this.url, globalThis.location?.origin ?? 'http://localhost')
		parsed.searchParams.set('after_position', this.lastEventId)
		return parsed.toString()
	}

	private sseHeaders(): Record<string, string> {
		const headers: Record<string, string> = {
			Accept: 'text/event-stream',
			'X-Hermes-Secret': this.secret
		}
		if (this.lastEventId) {
			headers['Last-Event-ID'] = this.lastEventId
		}
		return headers
	}

	private scheduleReconnect(error?: unknown): void {
		if (!this.shouldReconnect) return
		if (this.reconnectAttempts >= this.maxReconnectAttempts) {
			if (this.longPollUrl) {
				this.startLongPolling()
			} else {
				this.reportStatus({
					transport: 'sse',
					state: 'disconnected',
					error: error ? statusErrorMessage(error) : undefined
				})
				this.onError?.(new Error('SSE reconnect attempts exhausted'))
			}
			return
		}

		this.reconnectAttempts += 1
		this.reportStatus({
			transport: 'sse',
			state: 'reconnecting',
			attempt: this.reconnectAttempts,
			maxAttempts: this.maxReconnectAttempts,
			error: error ? statusErrorMessage(error) : undefined
		})
		const delay = this.reconnectDelay * Math.min(this.reconnectAttempts, 5)
		this.reconnectTimer = setTimeout(() => {
			void this.connectOnce()
		}, delay)
	}

	private startLongPolling(): void {
		if (this.longPollRunning || !this.longPollUrl) return
		this.currentTransport = 'long_poll'
		this.reportStatus({ transport: 'long_poll', state: 'fallback' })
		void this.longPollLoop()
	}

	private async longPollLoop(): Promise<void> {
		this.longPollRunning = true
		try {
			while (this.shouldReconnect && this.longPollUrl) {
				try {
					await this.longPollOnce()
				} catch (error) {
					if (!this.shouldReconnect) return
					this.reportStatus({
						transport: 'long_poll',
						state: 'reconnecting',
						error: statusErrorMessage(error)
					})
					this.onError?.(error)
				}

				if (this.shouldReconnect) {
					await this.wait(this.longPollDelay)
				}
			}
		} finally {
			this.longPollRunning = false
		}
	}

	private async longPollOnce(): Promise<void> {
		this.abortController = new AbortController()
		const response = await this.fetchImpl(this.longPollReplayUrl(), {
			method: 'GET',
			headers: this.longPollHeaders(),
			cache: 'no-store',
			signal: this.abortController.signal
		})

		if (!response.ok) {
			throw new Error(`Long polling failed with HTTP ${response.status}`)
		}

		const payload = (await response.json()) as LongPollResponse
		if (!Array.isArray(payload.items)) {
			throw new Error('Long polling response missing items')
		}

		this.currentTransport = 'long_poll'
		this.reportStatus({ transport: 'long_poll', state: 'connected' })
		for (const item of payload.items) {
			this.dispatchLongPollItem(item)
			if (!this.shouldReconnect) break
		}
	}

	private dispatchLongPollItem(item: LongPollEventItem): void {
		const position = Number(item.position)
		if (!Number.isFinite(position) || position < 0) {
			throw new Error('Long polling event position is invalid')
		}

		const id = String(position)
		this.lastEventId = id
		this.reconnectAttempts = 0
		this.onMessage?.({
			id,
			event: 'event',
			data: JSON.stringify(item)
		})
	}

	private longPollReplayUrl(): string {
		const parsed = new URL(
			this.longPollUrl ?? '',
			globalThis.location?.origin ?? 'http://localhost'
		)
		parsed.searchParams.set('after_position', this.lastEventId || '0')
		parsed.searchParams.set('limit', String(this.longPollBatchSize))
		parsed.searchParams.set('wait_seconds', String(this.longPollWaitSeconds))
		return parsed.toString()
	}

	private longPollHeaders(): Record<string, string> {
		const headers: Record<string, string> = {
			Accept: 'application/json',
			'X-Hermes-Secret': this.secret
		}
		if (this.lastEventId) {
			headers['Last-Event-ID'] = this.lastEventId
		}
		return headers
	}

	private wait(delay: number): Promise<void> {
		return new Promise((resolve) => {
			this.reconnectTimer = setTimeout(resolve, delay)
		})
	}

	private stopCurrentConnection(): void {
		if (this.reconnectTimer) {
			clearTimeout(this.reconnectTimer)
			this.reconnectTimer = null
		}
		if (this.abortController) {
			this.abortController.abort()
			this.abortController = null
		}
	}

	private reportStatus(status: SseStatusEvent): void {
		this.onStatus?.(status)
	}
}

function statusErrorMessage(error: unknown): string {
	if (error instanceof Error) return error.message
	if (typeof error === 'string') return error
	return 'Unknown realtime transport error'
}
