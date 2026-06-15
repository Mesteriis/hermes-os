import { describe, expect, it } from 'vitest'
import { loadFrontendConfig } from './env'

describe('frontend env config', () => {
	it('uses Hermes env names and default backend URL', () => {
		const config = loadFrontendConfig({
			VITE_HERMES_LOCAL_API_SECRET: 'dev-secret'
		})

		expect(config.apiBaseUrl).toBe('http://127.0.0.1:8080')
		expect(config.apiSecret).toBe('dev-secret')
		expect(config.sseUrl).toBe('http://127.0.0.1:8080/api/events/stream')
		expect(config.webSocketUrl).toBe('ws://127.0.0.1:8080/api/events/ws')
		expect(config.realtimeTransport).toBe('sse')
	})

	it('rejects missing local API secret', () => {
		expect(() => loadFrontendConfig({})).toThrow('VITE_HERMES_LOCAL_API_SECRET is required')
	})

	it('accepts explicit Hermes backend URL', () => {
		const config = loadFrontendConfig({
			VITE_HERMES_API_BASE_URL: 'http://127.0.0.1:9090/',
			VITE_HERMES_LOCAL_API_SECRET: 'dev-secret'
		})

		expect(config.apiBaseUrl).toBe('http://127.0.0.1:9090')
		expect(config.sseUrl).toBe('http://127.0.0.1:9090/api/events/stream')
		expect(config.webSocketUrl).toBe('ws://127.0.0.1:9090/api/events/ws')
	})

	it('can opt back to WebSocket transport selection', () => {
		const config = loadFrontendConfig({
			VITE_HERMES_LOCAL_API_SECRET: 'dev-secret',
			VITE_HERMES_REALTIME_TRANSPORT: 'websocket'
		})

		expect(config.realtimeTransport).toBe('websocket')
	})
})
