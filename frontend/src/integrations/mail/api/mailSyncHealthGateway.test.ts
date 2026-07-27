import { beforeEach, describe, expect, it, vi } from 'vitest'

import { getMailSyncHealthConnectClient } from './mailSyncHealthClient'
import {
	getMailSyncRun,
	getMailSyncStatus,
	listMailSyncRuns,
} from './mailSyncHealthGateway'

vi.mock('./mailSyncHealthClient', () => ({
	getMailSyncHealthConnectClient: vi.fn(),
}))

const query = vi.fn()

describe('Mail sync health Gateway adapter', () => {
	beforeEach(() => {
		query.mockReset()
		vi.mocked(getMailSyncHealthConnectClient).mockReturnValue({ query } as never)
	})

	it('uses each exact generated health query with bounded normalized input', async () => {
		query
			.mockResolvedValueOnce({
				response: { case: 'status', value: { connectionId: 'primary' } },
			})
			.mockResolvedValueOnce({
				response: { case: 'runs', value: { item: [], nextCursor: 'next' } },
			})
			.mockResolvedValueOnce({
				response: {
					case: 'run',
					value: { run: { operationId: 'operation-1' } },
				},
			})

		await getMailSyncStatus(' primary ')
		await listMailSyncRuns({
			connectionId: 'primary',
			cursor: ' cursor-1 ',
			limit: 25,
		})
		await getMailSyncRun({
			connectionId: 'primary',
			operationId: ' operation-1 ',
		})

		expect(query.mock.calls.map(([request]) => request.query.case)).toEqual([
			'getStatus',
			'listRuns',
			'getRun',
		])
		expect(query.mock.calls[0]![0].query.value.connectionId).toBe('primary')
		expect(query.mock.calls[1]![0].query.value).toMatchObject({
			connectionId: 'primary',
			cursor: 'cursor-1',
			limit: 25,
		})
		expect(query.mock.calls[2]![0].query.value.operationId).toBe('operation-1')
	})

	it('returns a closed absence for an unknown run', async () => {
		query.mockResolvedValueOnce({
			response: { case: 'run', value: {} },
		})

		await expect(getMailSyncRun({
			connectionId: 'primary',
			operationId: 'missing',
		})).resolves.toBeNull()
	})

	it('fails closed before transport for invalid input and mismatched responses', async () => {
		await expect(listMailSyncRuns({
			connectionId: 'primary',
			limit: 201,
		})).rejects.toThrow('page limit')
		await expect(getMailSyncRun({
			connectionId: 'primary',
			operationId: 'bad\noperation',
		})).rejects.toThrow('operation ID is invalid')
		expect(query).not.toHaveBeenCalled()

		query.mockResolvedValueOnce({
			response: { case: 'runs', value: { item: [] } },
		})
		await expect(getMailSyncStatus('primary')).rejects.toThrow(
			'status response is unavailable',
		)

		query.mockResolvedValueOnce({
			response: { case: 'status', value: { connectionId: 'primary' } },
		})
		await expect(listMailSyncRuns({
			connectionId: 'primary',
		})).rejects.toThrow('runs response is unavailable')
	})
})
