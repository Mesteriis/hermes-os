import { afterEach, describe, expect, it, vi } from 'vitest'
import { handleRealtimeEvent, initializeRealtime } from './realtime'
import type { RealtimeClientOptions } from './realtime'
import type { SseClientOptions } from '../sse'

const config = {
	apiBaseUrl: 'http://127.0.0.1:8080',
	apiSecret: 'test-secret',
	sseUrl: 'http://127.0.0.1:8080/api/realtime/v2/events'
}

describe('realtime bootstrap', () => {
	afterEach(() => vi.unstubAllGlobals())

	it('creates one protected SSE client and connects it', () => {
		const connect = vi.fn()
		const queryClient = { invalidateQueries: vi.fn() }
		let captured: RealtimeClientOptions | undefined

		initializeRealtime(config, queryClient, {
			createClient: (options) => {
				captured = options
				return { connect, disconnect: vi.fn(), reconnect: vi.fn() }
			}
		})

		expect(connect).toHaveBeenCalledOnce()
		expect(captured).toMatchObject({
			url: config.sseUrl,
			secret: config.apiSecret
		})
	})

	it('persists event cursors and preserves them across SSE reconnects', () => {
		const storage = { getItem: vi.fn().mockReturnValue('41'), setItem: vi.fn() }
		vi.stubGlobal('localStorage', storage)
		let captured: SseClientOptions | undefined

		initializeRealtime(config, { invalidateQueries: vi.fn() }, (options) => {
			captured = options
			return { connect: vi.fn(), disconnect: vi.fn(), reconnect: vi.fn() }
		})

		expect(captured?.lastEventId).toBe('41')
		captured?.onMessage?.({ id: '42', event: 'event', data: '{}' })
		expect(storage.setItem).toHaveBeenCalledWith('hermes.realtime.lastEventId', '42')
	})

	it('invalidates affected queries after a replay gap', () => {
		const queryClient = { invalidateQueries: vi.fn() }
		handleRealtimeEvent({ id: '52', event: 'lagged', data: JSON.stringify({ skipped: 4 }) }, queryClient)

		expect(queryClient.invalidateQueries).toHaveBeenCalledWith({ queryKey: ['communications-list'] })
		expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
			queryKey: ['integrations', 'telegram', 'runtime']
		})
	})
})
