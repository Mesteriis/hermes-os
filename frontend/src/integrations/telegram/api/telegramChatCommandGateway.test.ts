import { beforeEach, describe, expect, it, vi } from 'vitest'

import {
	addTelegramChatToFolder,
	joinTelegramChat,
	leaveTelegramChat,
	removeTelegramChatFromFolder,
	setTelegramChatArchived,
	setTelegramChatMuted,
	setTelegramChatUnread,
} from './telegramChatCommandGateway'
import { getTelegramOperationalConnectClient } from './telegramOperationalClient'

vi.mock('./telegramOperationalClient', () => ({
	getTelegramOperationalConnectClient: vi.fn(),
}))

const executeCommand = vi.fn()
const target = {
	accountId: 'account-1',
	providerChatId: 'chat-1',
	operationId: 'operation-1',
}

describe('Telegram chat command adapter', () => {
	beforeEach(() => {
		executeCommand.mockReset()
		executeCommand.mockResolvedValue({ operationId: 'operation-1', state: 'accepted' })
		vi.mocked(getTelegramOperationalConnectClient).mockReturnValue({ executeCommand } as never)
	})

	it('keeps chat state, membership and folders as distinct generated variants', async () => {
		await setTelegramChatUnread(target, true)
		await setTelegramChatArchived(target, true)
		await setTelegramChatMuted(target, true)
		await joinTelegramChat(target)
		await leaveTelegramChat(target)
		await addTelegramChatToFolder(target, 4n)
		await removeTelegramChatFromFolder(target, 4n)

		expect(executeCommand.mock.calls.map(([request]) => request.command.case)).toEqual([
			'markUnread',
			'archive',
			'mute',
			'join',
			'leave',
			'addChatToFolder',
			'removeChatFromFolder',
		])
	})

	it('rejects invalid folder IDs before transport', async () => {
		await expect(addTelegramChatToFolder(target, -1n)).rejects.toThrow(
			'Telegram folder ID must be non-negative',
		)
		expect(executeCommand).not.toHaveBeenCalled()
	})
})
