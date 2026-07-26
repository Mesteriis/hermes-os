import { beforeEach, describe, expect, it, vi } from 'vitest'

import { getWhatsAppCommandConnectClient } from './whatsappCommandClient'
import {
	getWhatsAppOperationStatus,
	sendWhatsAppText,
} from './whatsappOperationalGateway'
import { getWhatsAppQueryConnectClient } from './whatsappQueryClient'

vi.mock('./whatsappCommandClient', () => ({ getWhatsAppCommandConnectClient: vi.fn() }))
vi.mock('./whatsappQueryClient', () => ({ getWhatsAppQueryConnectClient: vi.fn() }))

const executeCommand = vi.fn()
const getOperationStatus = vi.fn()

describe('WhatsApp operational Gateway adapter', () => {
	beforeEach(() => {
		executeCommand.mockReset()
		getOperationStatus.mockReset()
		vi.mocked(getWhatsAppCommandConnectClient).mockReturnValue({ executeCommand } as never)
		vi.mocked(getWhatsAppQueryConnectClient).mockReturnValue({ getOperationStatus } as never)
	})

	it('dispatches the exact generated send command', async () => {
		executeCommand.mockResolvedValue({ operationId: 'operation-1', contractName: 'whatsapp.command.v1' })

		await expect(sendWhatsAppText({
			accountId: ' account-1 ',
			providerChatId: ' chat-1 ',
			text: ' Hello ',
			operationId: 'operation-1',
		})).resolves.toMatchObject({ operationId: 'operation-1' })

		expect(executeCommand).toHaveBeenCalledWith({
			command: {
				case: 'sendText',
				value: {
					accountId: 'account-1',
					providerChatId: 'chat-1',
					text: 'Hello',
					operationId: 'operation-1',
				},
			},
		})
	})

	it('reads terminal status through the exact generated query', async () => {
		getOperationStatus.mockResolvedValue({
			status: { operationId: 'operation-1', accountId: 'account-1', state: 'completed' },
		})

		await expect(getWhatsAppOperationStatus(' operation-1 ')).resolves.toMatchObject({
			state: 'completed',
		})
		expect(getOperationStatus).toHaveBeenCalledWith({ operationId: 'operation-1' })
	})

	it('rejects missing identifiers before transport', async () => {
		await expect(getWhatsAppOperationStatus(' ')).rejects.toThrow('operation ID is required')
		await expect(sendWhatsAppText({
			accountId: '',
			providerChatId: 'chat-1',
			text: 'Hello',
			operationId: 'operation-1',
		})).rejects.toThrow('account ID is required')
		expect(executeCommand).not.toHaveBeenCalled()
		expect(getOperationStatus).not.toHaveBeenCalled()
	})
})
