export type SseEventHandler = (event: MessageEvent) => void
export type SseErrorHandler = (error: Event) => void

export interface SseClientOptions {
	url: string
	onMessage?: SseEventHandler
	onError?: SseErrorHandler
	reconnectDelay?: number
	maxReconnectAttempts?: number
}

/**
 * EventSource wrapper with automatic reconnect logic.
 */
export class SseClient {
	private url: string
	private eventSource: EventSource | null = null
	private onMessage?: SseEventHandler
	private onError?: SseErrorHandler
	private reconnectDelay: number
	private maxReconnectAttempts: number
	private reconnectAttempts = 0
	private shouldReconnect = true

	constructor(options: SseClientOptions) {
		this.url = options.url
		this.onMessage = options.onMessage
		this.onError = options.onError
		this.reconnectDelay = options.reconnectDelay ?? 3000
		this.maxReconnectAttempts = options.maxReconnectAttempts ?? 10
	}

	connect(): void {
		if (this.eventSource) {
			this.disconnect()
		}

		this.shouldReconnect = true
		this.reconnectAttempts = 0
		this._connect()
	}

	private _connect(): void {
		try {
			this.eventSource = new EventSource(this.url)
		} catch (err) {
			console.error('[SseClient] Failed to create EventSource:', err)
			this._scheduleReconnect()
			return
		}

		this.eventSource.onmessage = (event: MessageEvent) => {
			this.reconnectAttempts = 0
			this.onMessage?.(event)
		}

		this.eventSource.onerror = (event: Event) => {
			console.warn('[SseClient] Connection error:', event)
			this.onError?.(event)
			this.eventSource?.close()
			this.eventSource = null
			this._scheduleReconnect()
		}
	}

	private _scheduleReconnect(): void {
		if (!this.shouldReconnect) return
		if (this.reconnectAttempts >= this.maxReconnectAttempts) {
			console.error('[SseClient] Max reconnect attempts reached')
			return
		}

		this.reconnectAttempts++
		const delay = this.reconnectDelay * Math.min(this.reconnectAttempts, 5)
		console.debug(`[SseClient] Reconnecting in ${delay}ms (attempt ${this.reconnectAttempts}/${this.maxReconnectAttempts})`)
		setTimeout(() => this._connect(), delay)
	}

	disconnect(): void {
		this.shouldReconnect = false
		this.eventSource?.close()
		this.eventSource = null
	}
}
