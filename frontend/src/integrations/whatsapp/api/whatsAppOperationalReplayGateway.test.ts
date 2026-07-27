import { beforeEach, describe, expect, it, vi } from 'vitest'

import { getWhatsAppOperationalRealtimeConnectClient } from './whatsAppOperationalRealtimeClient'
import { replayWhatsAppOperationalEvents } from './whatsAppOperationalReplayGateway'

vi.mock('./whatsAppOperationalRealtimeClient', () => ({
	getWhatsAppOperationalRealtimeConnectClient: vi.fn(),
}))

const replay = vi.fn()

describe('WhatsApp operational replay Gateway adapter', () => {
	beforeEach(() => {
		replay.mockReset()
		vi.mocked(getWhatsAppOperationalRealtimeConnectClient).mockReturnValue({ replay } as never)
	})

	it('uses the exact generated replay contract through the signed i64 cursor range', async () => {
		replay.mockResolvedValue({
			accountId: 'account-1',
			frame: [],
			nextSequence: 9_223_372_036_854_775_807n,
			resetRequired: false,
		})

		await replayWhatsAppOperationalEvents({
			accountId: ' account-1 ',
			afterSequence: 9_223_372_036_854_775_807n,
			limit: 500,
		})

		expect(replay.mock.calls[0]![0]).toMatchObject({
			accountId: 'account-1',
			afterSequence: 9_223_372_036_854_775_807n,
			limit: 500,
		})
	})

	it('rejects invalid bounds and a cross-account response', async () => {
		await expect(replayWhatsAppOperationalEvents({
			accountId: 'account-1',
			afterSequence: 9_223_372_036_854_775_808n,
		})).rejects.toThrow('sequence is invalid')
		await expect(replayWhatsAppOperationalEvents({
			accountId: 'bad\naccount',
		})).rejects.toThrow('account ID is invalid')
		expect(replay).not.toHaveBeenCalled()

		replay.mockResolvedValue({
			accountId: 'account-2',
			frame: [],
			nextSequence: 0n,
			resetRequired: false,
		})
		await expect(replayWhatsAppOperationalEvents({
			accountId: 'account-1',
		})).rejects.toThrow('account response is invalid')
	})
})
