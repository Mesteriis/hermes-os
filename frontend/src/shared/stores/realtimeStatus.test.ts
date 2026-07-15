import { beforeEach, describe, expect, it, vi } from 'vitest'
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

		store.setRealtimeStatus({ transport: 'sse', state: 'reconnecting' })
		expect(store.realtimeStatusLabel).toBe('Realtime reconnecting')
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

	it('tracks replay cursor progress for offline recovery diagnostics', () => {
		const store = useRealtimeStatusStore()

		expect(store.realtimeRecoveryDetail).toBe('Waiting for first replay cursor')

		store.observeRealtimeEvent('51')
		expect(store.status.lastEventId).toBe('51')
		expect(store.status.lastEventAt).toBeTruthy()
		expect(store.realtimeRecoveryDetail).toContain('Replay cursor 51.')

		store.setRealtimeStatus({
			transport: 'sse',
			state: 'disconnected',
			error: 'stream closed'
		})
		expect(store.realtimeRecoveryDetail).toContain(
			'Offline recovery will resume from cursor 51.'
		)
		expect(store.canTriggerReconnect).toBe(true)
	})

	it('surfaces replay-gap diagnostics when the transport reports skipped realtime events', () => {
		const store = useRealtimeStatusStore()

		store.observeRealtimeEvent('51')
		store.observeRealtimeLag(7)

		expect(store.isRealtimeDegraded).toBe(true)
		expect(store.canTriggerReconnect).toBe(true)
		expect(store.realtimeRecoveryDetail).toContain('Replay gap detected after cursor 51.')
		expect(store.realtimeRecoveryDetail).toContain('Skipped 7 events.')

		store.setRealtimeStatus({
			transport: 'sse',
			state: 'connected'
		})
		expect(store.isRealtimeDegraded).toBe(false)
		expect(store.realtimeRecoveryDetail).toContain('Replay cursor 51.')
	})

	it('exposes a manual reconnect control only for degraded or disconnected transports', () => {
		const store = useRealtimeStatusStore()
		const reconnect = vi.fn()

		store.setReconnectHandler(reconnect)
		expect(store.canTriggerReconnect).toBe(false)

		store.requestReconnect()
		expect(reconnect).toHaveBeenCalledTimes(1)

		store.setRealtimeStatus({
			transport: 'sse',
			state: 'reconnecting'
		})
		expect(store.canTriggerReconnect).toBe(true)
	})
})
