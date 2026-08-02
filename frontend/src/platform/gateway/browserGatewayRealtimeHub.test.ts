import { create } from '@bufbuild/protobuf'
import { describe, expect, it, vi } from 'vitest'

import {
	ClientRealtimeEventV1Schema,
	ClientRealtimeStreamStateV1Schema,
	ClientReplayGapV1Schema,
} from '../../gen/hermes/gateway/v1/client_realtime_pb'
import type {
	BrowserGatewayRealtimeObserver,
	BrowserGatewayRealtimeSubscription,
} from './browserGatewayRealtime'
import { BrowserGatewayRealtimeHub } from './browserGatewayRealtimeHub'

describe('BrowserGatewayRealtimeHub', () => {
	it('shares one browser EventSource subscription across independent consumers', () => {
		let sourceObserver: BrowserGatewayRealtimeObserver | undefined
		const closeSource = vi.fn()
		const subscribe = vi.fn((observer: BrowserGatewayRealtimeObserver): BrowserGatewayRealtimeSubscription => {
			sourceObserver = observer
			return { close: closeSource }
		})
		const hub = new BrowserGatewayRealtimeHub({ subscribe })
		const first = observerFixture()
		const second = observerFixture()
		const firstSubscription = hub.subscribe(first)
		const secondSubscription = hub.subscribe(second)

		expect(subscribe).toHaveBeenCalledTimes(1)
		const event = create(ClientRealtimeEventV1Schema, { cursor: 'cursor-1' })
		sourceObserver?.onEvent(event)
		expect(first.onEvent).toHaveBeenCalledWith(event)
		expect(second.onEvent).toHaveBeenCalledWith(event)

		firstSubscription.close()
		expect(closeSource).not.toHaveBeenCalled()
		secondSubscription.close()
		expect(closeSource).toHaveBeenCalledTimes(1)
	})

	it('broadcasts stream state and fail-closed replay signals', () => {
		let sourceObserver: BrowserGatewayRealtimeObserver | undefined
		const closeSource = vi.fn()
		const subscribe = vi.fn((observer: BrowserGatewayRealtimeObserver) => {
			sourceObserver = observer
			return { close: closeSource }
		})
		const hub = new BrowserGatewayRealtimeHub({ subscribe })
		const observer = observerFixture()
		hub.subscribe(observer)

		const state = create(ClientRealtimeStreamStateV1Schema, { cursor: 'cursor-2' })
		const gap = create(ClientReplayGapV1Schema, { reasonCode: 'history_gap' })
		sourceObserver?.onStreamState(state)
		sourceObserver?.onReplayGap(gap)
		sourceObserver?.onProtocolError()

		expect(observer.onStreamState).toHaveBeenCalledWith(state)
		expect(observer.onReplayGap).toHaveBeenCalledWith(gap)
		expect(observer.onProtocolError).toHaveBeenCalledTimes(1)
		expect(closeSource).toHaveBeenCalledTimes(1)
	})
})

function observerFixture(): BrowserGatewayRealtimeObserver {
	return {
		onEvent: vi.fn(),
		onStreamState: vi.fn(),
		onReplayGap: vi.fn(),
		onProtocolError: vi.fn(),
	}
}
