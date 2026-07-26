import { beforeEach, describe, expect, it, vi } from 'vitest'

import {
	createTelegramTopic,
	requestTelegramMessageSearch,
	requestTelegramParticipants,
	requestTelegramTopics,
	setTelegramTopicClosed,
} from './telegramTopicCommandGateway'
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

describe('Telegram topic and provider-fetch command adapter', () => {
	beforeEach(() => {
		executeCommand.mockReset()
		executeCommand.mockResolvedValue({ operationId: 'operation-1', state: 'accepted' })
		vi.mocked(getTelegramOperationalConnectClient).mockReturnValue({ executeCommand } as never)
	})

	it('uses distinct search, participant and topic variants', async () => {
		await requestTelegramMessageSearch(target, 'architecture')
		await requestTelegramParticipants(target)
		await requestTelegramTopics(target)
		await createTelegramTopic(target, 'Decisions')
		await setTelegramTopicClosed(target, 'topic-1', true)

		expect(executeCommand.mock.calls.map(([request]) => request.command.case)).toEqual([
			'searchMessages',
			'listParticipants',
			'listTopics',
			'createTopic',
			'setTopicClosed',
		])
	})

	it('rejects missing topic semantics before transport', async () => {
		await expect(createTelegramTopic(target, ' ')).rejects.toThrow('topic title is required')
		await expect(setTelegramTopicClosed(target, '', true)).rejects.toThrow('topic ID is required')
		expect(executeCommand).not.toHaveBeenCalled()
	})
})
