import { beforeEach, describe, expect, it, vi } from 'vitest'

import {
	downloadTelegramFile,
	sendTelegramMedia,
} from './telegramMediaCommandGateway'
import { getTelegramOperationalConnectClient } from './telegramOperationalClient'

vi.mock('./telegramOperationalClient', () => ({
	getTelegramOperationalConnectClient: vi.fn(),
}))

const executeCommand = vi.fn()

describe('Telegram media command adapter', () => {
	beforeEach(() => {
		executeCommand.mockReset()
		executeCommand.mockResolvedValue({ operationId: 'operation-1', state: 'accepted' })
		vi.mocked(getTelegramOperationalConnectClient).mockReturnValue({ executeCommand } as never)
	})

	it('sends admitted Blob intent and provider file download through exact variants', async () => {
		await sendTelegramMedia({
			accountId: 'account-1',
			providerChatId: 'chat-1',
			operationId: 'operation-1',
			mediaKind: 'document',
			blobRef: 'blob-ref-1',
			referenceIdHex: '0a0b',
			declaredSize: 42n,
			backupClass: 1,
			caption: ' Evidence ',
			filename: 'adr.pdf',
		})
		await downloadTelegramFile({
			accountId: 'account-1',
			providerFileId: 'file-1',
			operationId: 'operation-2',
			priority: 1,
		})

		expect(executeCommand.mock.calls.map(([request]) => request.command.case)).toEqual([
			'sendMedia',
			'downloadFile',
		])
		expect(executeCommand).toHaveBeenNthCalledWith(1, {
			command: {
				case: 'sendMedia',
				value: {
					accountId: 'account-1',
					providerChatId: 'chat-1',
					operationId: 'operation-1',
					mediaKind: 'document',
					blob: {
						blobRef: 'blob-ref-1',
						referenceId: new Uint8Array([10, 11]),
						declaredSize: 42n,
						backupClass: 1,
					},
					caption: 'Evidence',
					filename: 'adr.pdf',
				},
			},
		})
	})

	it('rejects invalid Blob reference inputs before transport', async () => {
		await expect(sendTelegramMedia({
			accountId: 'account-1',
			providerChatId: 'chat-1',
			operationId: 'operation-1',
			mediaKind: 'document',
			blobRef: 'blob-ref-1',
			referenceIdHex: 'xyz',
			declaredSize: 42n,
			backupClass: 1,
		})).rejects.toThrow('Telegram Blob reference ID must be even-length hexadecimal')
		expect(executeCommand).not.toHaveBeenCalled()
	})
})
