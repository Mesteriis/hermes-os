import { beforeEach, describe, expect, it, vi } from 'vitest'

import {
	deleteTelegramMessage,
	editTelegramMessage,
	forwardTelegramMessage,
	replyToTelegramMessage,
	restoreTelegramMessageVisibility,
	setTelegramMessagePinned,
	setTelegramMessageReaction,
} from './telegramMessageCommandGateway'
import { getTelegramOperationalConnectClient } from './telegramOperationalClient'

vi.mock('./telegramOperationalClient', () => ({
	getTelegramOperationalConnectClient: vi.fn(),
}))

const executeCommand = vi.fn()
const target = {
	accountId: 'account-1',
	providerChatId: 'chat-1',
	providerMessageId: 'message-1',
	operationId: 'operation-1',
}

describe('Telegram message command adapter', () => {
	beforeEach(() => {
		executeCommand.mockReset()
		executeCommand.mockResolvedValue({ operationId: 'operation-1', state: 'accepted' })
		vi.mocked(getTelegramOperationalConnectClient).mockReturnValue({ executeCommand } as never)
	})

	it('uses one exact generated variant per message action', async () => {
		await replyToTelegramMessage(target, ' Reply ')
		await forwardTelegramMessage({ ...target, targetProviderChatId: 'chat-2' })
		await editTelegramMessage(target, ' Edit ')
		await deleteTelegramMessage(target, true)
		await restoreTelegramMessageVisibility(target, ' owner_restore ')
		await setTelegramMessageReaction(target, ' 👍 ', true)
		await setTelegramMessagePinned(target, true)

		expect(executeCommand.mock.calls.map(([request]) => request.command.case)).toEqual([
			'reply',
			'forward',
			'edit',
			'delete',
			'restoreVisibility',
			'reaction',
			'pin',
		])
		expect(executeCommand).toHaveBeenNthCalledWith(1, {
			command: {
				case: 'reply',
				value: {
					accountId: 'account-1',
					providerChatId: 'chat-1',
					replyToProviderMessageId: 'message-1',
					operationId: 'operation-1',
					text: 'Reply',
				},
			},
		})
	})

	it('rejects missing message input before transport', async () => {
		await expect(replyToTelegramMessage(target, ' ')).rejects.toThrow(
			'Telegram message text is required',
		)
		await expect(setTelegramMessageReaction(target, '', true)).rejects.toThrow(
			'reaction is required',
		)
		expect(executeCommand).not.toHaveBeenCalled()
	})
})
