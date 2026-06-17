export type WebSocketEventHandler = (event: {
	id: string
	event: string
	data: string
}) => void
export type WebSocketErrorHandler = (error: unknown) => void
export type WebSocketTransportState =
	| 'connecting'
	| 'connected'
	| 'reconnecting'
	| 'disconnected'

export type WebSocketStatusEvent = {
	transport: 'websocket'
	state: WebSocketTransportState
	attempt?: number
	maxAttempts?: number
	error?: string
}

export type WebSocketStatusHandler = (status: WebSocketStatusEvent) => void

export interface WebSocketClientOptions {
	url: string
	secret: string
	lastEventId?: string
	onMessage?: WebSocketEventHandler
	onError?: WebSocketErrorHandler
	onStatus?: WebSocketStatusHandler
	reconnectDelay?: number
	maxReconnectAttempts?: number
}

type WebSocketPayload = {
	position?: number | string
	type?: string
}

type WebSocketEnvelope = {
	type?: string
	data?: WebSocketPayload | string
}

type WebSocketLaggedPayload = {
	skipped?: number
}

/**
 * Browser WebSocket event client with replay cursor persistence and reconnect loop.
 * Uses query parameters for authentication and replay because browsers cannot set
 * custom headers in native WebSocket requests.
 */
export class WebSocketClient {
	private url: string
	private secret: string
	private lastEventId: string
	private onMessage?: WebSocketEventHandler
	private onError?: WebSocketErrorHandler
	private onStatus?: WebSocketStatusHandler
	private reconnectDelay: number
	private maxReconnectAttempts: number
	private reconnectAttempts = 0
	private shouldReconnect = true
	private socket: WebSocket | null = null
	private reconnectTimer: ReturnType<typeof setTimeout> | null = null

	constructor(options: WebSocketClientOptions) {
		this.url = options.url
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
	}

	connect(): void {
		this.stop()
		this.shouldReconnect = true
		this.reconnectAttempts = 0
		this.connectOnce()
	}

	disconnect(): void {
		this.shouldReconnect = false
		this.stop()
		this.reportStatus({ transport: 'websocket', state: 'disconnected' })
	}

	private connectOnce(): void {
		if (typeof globalThis.WebSocket !== 'function') {
			const error = new Error('WebSocket is not supported in this runtime')
			this.onError?.(error)
			this.reportStatus({
				transport: 'websocket',
				state: 'disconnected',
				error: error.message
			})
			return
		}

		this.reportStatus({ transport: 'websocket', state: 'connecting' })
		this.socket = new WebSocket(this.replayUrl())

		this.socket.addEventListener('open', () => {
			this.reconnectAttempts = 0
			this.reportStatus({ transport: 'websocket', state: 'connected' })
		})

		this.socket.addEventListener('message', (event) => {
			this.handleMessage(event.data)
		})

		this.socket.addEventListener('error', () => {
			this.handleSocketError(new Error('WebSocket transport error'))
		})

		this.socket.addEventListener('close', () => {
			if (!this.shouldReconnect) {
				return
			}
			this.handleSocketClose()
		})
	}

	private handleMessage(rawMessage: unknown): void {
		if (typeof rawMessage !== 'string') {
			return
		}

		let envelope: WebSocketEnvelope
		try {
			envelope = JSON.parse(rawMessage) as WebSocketEnvelope
		} catch (error) {
			this.onError?.(error)
			return
		}

		if (envelope.type === 'heartbeat') {
			return
		}

		if (envelope.type === 'lagged') {
			const skipped = parseLaggedSkipped(envelope.data)
			if (skipped === null) {
				this.onError?.(new Error('WebSocket lagged payload missing skipped count'))
				return
			}

			this.onMessage?.({
				id: this.lastEventId,
				event: 'lagged',
				data: JSON.stringify({ skipped })
			})
			return
		}

		if (envelope.type !== 'event') {
			const message = `Unknown WebSocket message type ${String(envelope.type ?? 'unknown')}`
			this.onError?.(new Error(message))
			return
		}

		if (envelope.data === undefined) {
			this.onError?.(new Error('WebSocket message missing event payload'))
			return
		}

		const position = parseEnvelopePosition(envelope.data)
		if (!position) {
			this.onError?.(new Error('WebSocket message missing event position'))
			return
		}

		const data =
			typeof envelope.data === 'string' ? envelope.data : JSON.stringify(envelope.data)
		this.lastEventId = position
		this.reconnectAttempts = 0
		this.onMessage?.({
			id: this.lastEventId,
			event: 'event',
			data
		})
	}

	private handleSocketError(error: unknown): void {
		this.onError?.(error)
		if (!this.shouldReconnect) return
		this.scheduleReconnect(error)
	}

	private handleSocketClose(): void {
		this.scheduleReconnect(new Error('WebSocket connection closed'))
	}

	private scheduleReconnect(error: unknown): void {
		if (!this.shouldReconnect) {
			return
		}
		if (this.reconnectTimer) {
			return
		}

		if (this.reconnectAttempts >= this.maxReconnectAttempts) {
			this.reportStatus({
				transport: 'websocket',
				state: 'disconnected',
				error: statusErrorMessage(error)
			})
			this.onError?.(new Error('WebSocket reconnect attempts exhausted'))
			return
		}

		this.reconnectAttempts += 1
		this.reportStatus({
			transport: 'websocket',
			state: 'reconnecting',
			attempt: this.reconnectAttempts,
			maxAttempts: this.maxReconnectAttempts,
			error: statusErrorMessage(error)
		})

		const delay = this.reconnectDelay * Math.min(this.reconnectAttempts, 5)
		this.reconnectTimer = setTimeout(() => {
			this.reconnectTimer = null
			this.connectOnce()
		}, delay)
	}

	private replayUrl(): string {
		const parsed = new URL(this.url, globalThis.location?.origin ?? 'http://localhost')
		if (this.lastEventId) {
			parsed.searchParams.set('after_position', this.lastEventId)
		}
		parsed.searchParams.set('hermes_secret', this.secret)
		return parsed.toString()
	}

	private stop(): void {
		if (this.reconnectTimer) {
			clearTimeout(this.reconnectTimer)
			this.reconnectTimer = null
		}
		if (this.socket) {
			this.socket.close()
			this.socket = null
		}
	}

	private reportStatus(status: WebSocketStatusEvent): void {
		this.onStatus?.(status)
	}
}

function parseEnvelopePosition(data: WebSocketPayload | string): string | null {
	if (typeof data === 'string') return null
	if (typeof data.position === 'undefined') return null

	const position = Number(data.position)
	if (!Number.isFinite(position) || position < 0) {
		return null
	}

	return String(Math.floor(position))
}

function parseLaggedSkipped(data: WebSocketPayload | string | undefined): number | null {
	if (!data || typeof data === 'string') return null

	const payload = data as WebSocketLaggedPayload
	return typeof payload.skipped === 'number' && payload.skipped > 0 ? payload.skipped : null
}

function statusErrorMessage(error: unknown): string {
	if (error instanceof Error) return error.message
	if (typeof error === 'string') return error
	return 'Unknown realtime transport error'
}
