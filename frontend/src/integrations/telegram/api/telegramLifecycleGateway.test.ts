import { beforeEach, describe, expect, it, vi } from 'vitest'

import {
	listTelegramAccounts,
	provisionTelegramAccount,
	replayTelegramAccount,
	retireTelegramAccount,
	startTelegramAccount,
	stopTelegramAccount,
} from './telegramLifecycleGateway'
import { getTelegramLifecycleConnectClient } from './telegramLifecycleClient'

vi.mock('./telegramLifecycleClient', () => ({
	getTelegramLifecycleConnectClient: vi.fn(),
}))

const execute = vi.fn()

describe('Telegram lifecycle Gateway adapter', () => {
	beforeEach(() => {
		execute.mockReset()
		vi.mocked(getTelegramLifecycleConnectClient).mockReturnValue({ execute } as never)
	})

	it('lists and provisions owner-local accounts', async () => {
		execute
			.mockResolvedValueOnce({
				response: { case: 'accounts', value: { account: [{ accountId: 'account-1' }] } },
			})
			.mockResolvedValueOnce({
				response: { case: 'account', value: { accountId: 'account-2' } },
			})

		await expect(listTelegramAccounts()).resolves.toHaveLength(1)
		await expect(provisionTelegramAccount({
			accountId: ' account-2 ',
			providerKind: ' telegram ',
			displayName: ' Personal ',
			externalAccountId: ' @owner ',
			credentials: [],
			qrAuthorized: true,
		})).resolves.toMatchObject({ accountId: 'account-2' })

		expect(execute).toHaveBeenNthCalledWith(2, {
			request: {
				case: 'provision',
				value: {
					accountId: 'account-2',
					providerKind: 'telegram',
					displayName: 'Personal',
					externalAccountId: '@owner',
					credential: [],
					qrAuthorized: true,
				},
			},
		})
	})

	it('keeps start, stop, replay and retire as exact lifecycle actions', async () => {
		execute
			.mockResolvedValueOnce({ response: { case: 'accepted', value: { operationId: 'start-1' } } })
			.mockResolvedValueOnce({ response: { case: 'accepted', value: { operationId: 'stop-1' } } })
			.mockResolvedValueOnce({
				response: {
					case: 'operation',
					value: { operationId: 'replay-1', state: 'accepted' },
				},
			})
			.mockResolvedValueOnce({ response: { case: 'accepted', value: { operationId: 'retire-1' } } })

		await expect(startTelegramAccount('account-1', 'desktop', 100n)).resolves.toBe('start-1')
		await expect(stopTelegramAccount('account-1')).resolves.toBe('stop-1')
		await expect(replayTelegramAccount('account-1', 8n)).resolves.toMatchObject({
			operationId: 'replay-1',
		})
		await expect(retireTelegramAccount('account-1')).resolves.toBe('retire-1')

		expect(execute).toHaveBeenNthCalledWith(1, {
			request: {
				case: 'startAccount',
				value: {
					accountId: 'account-1',
					topology: 'managed',
					holder: 'desktop',
					nowUnixSeconds: 100n,
					expiresAtUnixSeconds: 160n,
				},
			},
		})
		expect(execute).toHaveBeenNthCalledWith(2, {
			request: { case: 'stopAccount', value: { accountId: 'account-1' } },
		})
		expect(execute).toHaveBeenNthCalledWith(3, {
			request: {
				case: 'replay',
				value: { accountId: 'account-1', afterSequence: 8n, limit: 100 },
			},
		})
		expect(execute).toHaveBeenNthCalledWith(4, {
			request: { case: 'retireAccount', value: { accountId: 'account-1' } },
		})
	})

	it('rejects missing lifecycle identifiers before transport', async () => {
		await expect(stopTelegramAccount(' ')).rejects.toThrow('account ID is required')
		await expect(startTelegramAccount('account-1', '', 100n)).rejects.toThrow(
			'runtime holder is required',
		)
		expect(execute).not.toHaveBeenCalled()
	})
})
