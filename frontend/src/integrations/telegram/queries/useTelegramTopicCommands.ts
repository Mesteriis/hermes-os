import { computed, ref } from 'vue'

import {
	createTelegramTopic,
	requestTelegramMessageSearch,
	requestTelegramParticipants,
	requestTelegramTopics,
	setTelegramTopicClosed,
} from '../api/telegramTopicCommandGateway'
import type { TelegramTopicTarget } from '../api/telegramTopicCommandGateway'
import { useTelegramCommandFeedback } from './useTelegramCommandFeedback'

export type TelegramTopicCommandModel = {
	providerSearchQuery: string
	topicId: string
	topicTitle: string
	pending: boolean
	statusMessage: string
	canCommand: boolean
	hasChat: boolean
}

export function useTelegramTopicCommands(input: {
	accountId: () => string
	canCommand: () => boolean
	providerChatId: () => string
}) {
	const providerSearchQuery = ref('')
	const topicId = ref('')
	const topicTitle = ref('')
	const feedback = useTelegramCommandFeedback(input.canCommand)
	const model = computed<TelegramTopicCommandModel>(() => ({
		providerSearchQuery: providerSearchQuery.value,
		topicId: topicId.value,
		topicTitle: topicTitle.value,
		pending: feedback.pending.value,
		statusMessage: feedback.statusMessage.value,
		canCommand: input.canCommand(),
		hasChat: Boolean(input.providerChatId()),
	}))

	async function searchMessages(): Promise<void> {
		await feedback.run(() => requestTelegramMessageSearch(target(), providerSearchQuery.value))
	}

	async function refreshParticipants(): Promise<void> {
		await feedback.run(() => requestTelegramParticipants(target()))
	}

	async function refreshTopics(): Promise<void> {
		await feedback.run(() => requestTelegramTopics(target()))
	}

	async function createTopic(): Promise<void> {
		await feedback.run(() => createTelegramTopic(target(), topicTitle.value))
	}

	async function closeTopic(isClosed: boolean): Promise<void> {
		await feedback.run(() => setTelegramTopicClosed(target(), topicId.value, isClosed))
	}

	function updateProviderSearchQuery(value: string): void {
		providerSearchQuery.value = value
	}

	function updateTopicId(value: string): void {
		topicId.value = value
	}

	function updateTopicTitle(value: string): void {
		topicTitle.value = value
	}

	function target(): TelegramTopicTarget {
		return {
			accountId: input.accountId(),
			providerChatId: input.providerChatId(),
			operationId: crypto.randomUUID(),
		}
	}

	return {
		model,
		searchMessages,
		refreshParticipants,
		refreshTopics,
		createTopic,
		closeTopic,
		updateProviderSearchQuery,
		updateTopicId,
		updateTopicTitle,
	}
}
