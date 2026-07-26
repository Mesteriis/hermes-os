import { beforeEach, describe, expect, it, vi } from 'vitest'

import { getMailDeliveryCommandConnectClient } from './mailDeliveryCommandClient'
import { getMailDeliveryQueryConnectClient } from './mailDeliveryQueryClient'
import {
	getMailDeliveryStatus,
	sendMailMessage,
	syncMailInbox,
} from './mailOperationalGateway'
import { getMailSyncConnectClient } from './mailSyncClient'

vi.mock('./mailDeliveryCommandClient', () => ({ getMailDeliveryCommandConnectClient: vi.fn() }))
vi.mock('./mailDeliveryQueryClient', () => ({ getMailDeliveryQueryConnectClient: vi.fn() }))
vi.mock('./mailSyncClient', () => ({ getMailSyncConnectClient: vi.fn() }))

const send = vi.fn()
const sync = vi.fn()
const getOperationStatus = vi.fn()

describe('Mail operational Gateway adapter', () => {
	beforeEach(() => {
		send.mockReset()
		sync.mockReset()
		getOperationStatus.mockReset()
		vi.mocked(getMailDeliveryCommandConnectClient).mockReturnValue({ send } as never)
		vi.mocked(getMailDeliveryQueryConnectClient).mockReturnValue({ getOperationStatus } as never)
		vi.mocked(getMailSyncConnectClient).mockReturnValue({ sync } as never)
	})

	it('runs bounded sync through the generated Mail sync contract', async () => {
		sync.mockResolvedValue({ operationId: 'sync-1', observedMessages: 12 })
		await expect(syncMailInbox(' sync-1 ')).resolves.toMatchObject({ observedMessages: 12 })
		expect(sync).toHaveBeenCalledWith({ operationId: 'sync-1' })
	})

	it('dispatches generated delivery without invented attachment payloads', async () => {
		send.mockResolvedValue({ operationId: 'delivery-1' })
		await expect(sendMailMessage({
			operationId: 'delivery-1',
			providerConversationId: '',
			recipients: [' owner@example.com ', ''],
			subject: ' Clean room ',
			textBody: ' Ready ',
		})).resolves.toBe('delivery-1')
		expect(send).toHaveBeenCalledWith({
			operationId: 'delivery-1',
			providerConversationId: '',
			recipient: ['owner@example.com'],
			subject: 'Clean room',
			textBody: 'Ready',
			attachmentAnchorId: [],
		})
	})

	it('reads delivery status and validates input before transport', async () => {
		getOperationStatus.mockResolvedValue({ status: { operationId: 'delivery-1' } })
		await expect(getMailDeliveryStatus('delivery-1')).resolves.toMatchObject({
			operationId: 'delivery-1',
		})
		await expect(sendMailMessage({
			operationId: 'delivery-2',
			providerConversationId: '',
			recipients: [],
			subject: '',
			textBody: 'body',
		})).rejects.toThrow('recipient is required')
		expect(getOperationStatus).toHaveBeenCalledWith({ operationId: 'delivery-1' })
		expect(send).not.toHaveBeenCalled()
	})
})
