import { beforeEach, describe, expect, it, vi } from 'vitest'

import { getZulipOperationalRealtimeConnectClient } from './zulipOperationalRealtimeClient'
import { replayZulipOperationalEvents } from './zulipOperationalReplayGateway'

vi.mock('./zulipOperationalRealtimeClient', () => ({
	getZulipOperationalRealtimeConnectClient: vi.fn(),
}))

const replay = vi.fn()

describe('Zulip operational replay Gateway adapter', () => {
	beforeEach(() => {
		replay.mockReset()
		vi.mocked(getZulipOperationalRealtimeConnectClient).mockReturnValue({ replay } as never)
	})

	it('uses the exact generated replay contract through the uint64 cursor range', async () => {
		replay.mockResolvedValue({
			accountId: 'account-1',
			frame: [],
			nextSequence: 18_446_744_073_709_551_615n,
			resetRequired: false,
		})

		await replayZulipOperationalEvents({
			accountId: ' account-1 ',
			afterSequence: 18_446_744_073_709_551_615n,
			limit: 200,
		})

		expect(replay.mock.calls[0]![0]).toMatchObject({
			accountId: 'account-1',
			afterSequence: 18_446_744_073_709_551_615n,
			limit: 200,
		})
	})

	it('rejects invalid bounds and a cross-account response', async () => {
		await expect(replayZulipOperationalEvents({
			accountId: 'account-1',
			afterSequence: 18_446_744_073_709_551_616n,
		})).rejects.toThrow('sequence is invalid')
		await expect(replayZulipOperationalEvents({
			accountId: 'bad\naccount',
		})).rejects.toThrow('account ID is invalid')
		expect(replay).not.toHaveBeenCalled()

		replay.mockResolvedValue({
			accountId: 'account-2',
			frame: [],
			nextSequence: 0n,
			resetRequired: false,
		})
		await expect(replayZulipOperationalEvents({
			accountId: 'account-1',
		})).rejects.toThrow('account response is invalid')
	})
})
