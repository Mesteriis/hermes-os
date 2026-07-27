import { beforeEach, describe, expect, it, vi } from 'vitest'

import { getMailOperationalQueryConnectClient } from './mailOperationalQueryClient'
import {
	getMailOperationalMessage,
	listMailOperationalFolders,
	listMailOperationalMessages,
	listMailOperationalThreads,
} from './mailOperationalReadGateway'

vi.mock('./mailOperationalQueryClient', () => ({
	getMailOperationalQueryConnectClient: vi.fn(),
}))

const query = vi.fn()

describe('Mail operational read Gateway adapter', () => {
	beforeEach(() => {
		query.mockReset()
		vi.mocked(getMailOperationalQueryConnectClient).mockReturnValue({ query } as never)
	})

	it('uses every exact generated operational query and trims scoped identifiers', async () => {
		query
			.mockResolvedValueOnce({ response: { case: 'folders', value: { item: [] } } })
			.mockResolvedValueOnce({ response: { case: 'threads', value: { item: [] } } })
			.mockResolvedValueOnce({ response: { case: 'messages', value: { item: [] } } })
			.mockResolvedValueOnce({
				response: { case: 'message', value: { summary: { providerMessageId: 'message-1' } } },
			})

		await listMailOperationalFolders({ connectionId: ' primary ', limit: 25 })
		await listMailOperationalThreads({
			connectionId: 'primary',
			folderId: ' inbox ',
			cursor: ' cursor-1 ',
		})
		await listMailOperationalMessages({
			connectionId: 'primary',
			folderId: 'inbox',
			providerThreadId: ' thread-1 ',
		})
		await getMailOperationalMessage({
			connectionId: 'primary',
			providerMessageId: ' message-1 ',
		})

		expect(query.mock.calls.map(([request]) => request.query.case)).toEqual([
			'listFolders',
			'listThreads',
			'listMessages',
			'getMessage',
		])
		expect(query.mock.calls[0]![0].query.value).toMatchObject({
			connectionId: 'primary',
			limit: 25,
		})
		expect(query.mock.calls[1]![0].query.value).toMatchObject({
			folderId: 'inbox',
			cursor: 'cursor-1',
			limit: 50,
		})
		expect(query.mock.calls[2]![0].query.value.providerThreadId).toBe('thread-1')
		expect(query.mock.calls[3]![0].query.value.providerMessageId).toBe('message-1')
	})

	it('fails closed before transport for invalid input and mismatched responses', async () => {
		await expect(listMailOperationalFolders({
			connectionId: 'primary',
			limit: 201,
		})).rejects.toThrow('page limit')
		await expect(getMailOperationalMessage({
			connectionId: 'primary',
			providerMessageId: 'bad\nidentifier',
		})).rejects.toThrow('provider message ID is invalid')
		expect(query).not.toHaveBeenCalled()

		query.mockResolvedValueOnce({
			response: { case: 'messages', value: { item: [] } },
		})
		await expect(listMailOperationalFolders({
			connectionId: 'primary',
		})).rejects.toThrow('folders response is unavailable')

		query.mockResolvedValueOnce({
			response: { case: 'message', value: {} },
		})
		await expect(getMailOperationalMessage({
			connectionId: 'primary',
			providerMessageId: 'message-1',
		})).rejects.toThrow('message response is unavailable')
	})
})
