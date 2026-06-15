import { beforeEach, describe, expect, it } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useRealtimeStatusStore } from './realtimeStatus'

beforeEach(() => {
	setActivePinia(createPinia())
})

describe('realtime status store', () => {
	it('summarizes live and degraded realtime transport states for shell UI', () => {
		const store = useRealtimeStatusStore()

		expect(store.realtimeStatusLabel).toBe('Realtime starting')
		expect(store.realtimeStatusTone).toBe('neutral')
		expect(store.isRealtimeDegraded).toBe(false)

		store.setRealtimeStatus({ transport: 'sse', state: 'connected' })
		expect(store.realtimeStatusLabel).toBe('Realtime live')
		expect(store.realtimeStatusTone).toBe('success')
		expect(store.isRealtimeDegraded).toBe(false)

		store.setRealtimeStatus({ transport: 'long_poll', state: 'fallback' })
		expect(store.realtimeStatusLabel).toBe('Realtime fallback')
		expect(store.realtimeStatusTone).toBe('warning')
		expect(store.isRealtimeDegraded).toBe(true)
	})

	it('keeps sanitized error context for reconnecting transports', () => {
		const store = useRealtimeStatusStore()

		store.setRealtimeStatus({
			transport: 'sse',
			state: 'reconnecting',
			attempt: 2,
			maxAttempts: 10,
			error: 'SSE connection failed with HTTP 503'
		})

		expect(store.status.error).toBe('SSE connection failed with HTTP 503')
		expect(store.realtimeStatusDetail).toBe(
			'Realtime reconnecting: SSE connection failed with HTTP 503'
		)
		expect(store.realtimeStatusTone).toBe('warning')
	})
})
