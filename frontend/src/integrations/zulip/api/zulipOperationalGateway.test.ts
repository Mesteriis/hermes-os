import { beforeEach, describe, expect, it, vi } from 'vitest'

import { getZulipCommandConnectClient } from './zulipCommandClient'
import {
	getZulipOperationStatus,
	sendZulipDirectMessage,
	sendZulipStreamMessage,
} from './zulipOperationalGateway'
import { getZulipQueryConnectClient } from './zulipQueryClient'

vi.mock('./zulipCommandClient', () => ({ getZulipCommandConnectClient: vi.fn() }))
vi.mock('./zulipQueryClient', () => ({ getZulipQueryConnectClient: vi.fn() }))

const executeCommand = vi.fn()
const getOperationStatus = vi.fn()

describe('Zulip operational Gateway adapter', () => {
	beforeEach(() => {
		executeCommand.mockReset()
		getOperationStatus.mockReset()
		vi.mocked(getZulipCommandConnectClient).mockReturnValue({ executeCommand } as never)
		vi.mocked(getZulipQueryConnectClient).mockReturnValue({ getOperationStatus } as never)
	})

	it('dispatches exact stream and direct generated commands', async () => {
		executeCommand.mockResolvedValue({ operationId: 'operation-1', accountId: 'account-1' })

		await sendZulipStreamMessage({
			accountId: ' account-1 ',
			stream: ' engineering ',
			topic: ' clean-room ',
			content: ' Ready ',
			operationId: 'operation-1',
		})
		expect(executeCommand).toHaveBeenLastCalledWith({
			command: {
				case: 'sendStream',
				value: {
					accountId: 'account-1',
					stream: 'engineering',
					topic: 'clean-room',
					content: 'Ready',
					operationId: 'operation-1',
				},
			},
		})

		await sendZulipDirectMessage({
			accountId: 'account-1',
			recipients: [' owner@example.com ', '', 'team@example.com'],
			content: ' Ready ',
			operationId: 'operation-2',
		})
		expect(executeCommand).toHaveBeenLastCalledWith({
			command: {
				case: 'sendDirect',
				value: {
					accountId: 'account-1',
					recipient: ['owner@example.com', 'team@example.com'],
					content: 'Ready',
					operationId: 'operation-2',
				},
			},
		})
	})

	it('reads terminal status through the exact generated query', async () => {
		getOperationStatus.mockResolvedValue({
			status: { operationId: 'operation-1', accountId: 'account-1', outcome: 'completed' },
		})

		await expect(getZulipOperationStatus(' operation-1 ')).resolves.toMatchObject({
			outcome: 'completed',
		})
		expect(getOperationStatus).toHaveBeenCalledWith({ operationId: 'operation-1' })
	})

	it('rejects incomplete commands before transport', async () => {
		await expect(sendZulipDirectMessage({
			accountId: 'account-1',
			recipients: [],
			content: 'Ready',
			operationId: 'operation-1',
		})).rejects.toThrow('recipient is required')
		await expect(getZulipOperationStatus(' ')).rejects.toThrow('operation ID is required')
		expect(executeCommand).not.toHaveBeenCalled()
		expect(getOperationStatus).not.toHaveBeenCalled()
	})
})
