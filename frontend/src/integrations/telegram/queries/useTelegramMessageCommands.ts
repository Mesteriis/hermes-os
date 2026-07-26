import { computed, ref } from 'vue'

import {
	deleteTelegramMessage,
	editTelegramMessage,
	forwardTelegramMessage,
	replyToTelegramMessage,
	restoreTelegramMessageVisibility,
	setTelegramMessagePinned,
	setTelegramMessageReaction,
} from '../api/telegramMessageCommandGateway'
import type { TelegramMessageTarget } from '../api/telegramMessageCommandGateway'
import { useTelegramCommandFeedback } from './useTelegramCommandFeedback'

export type TelegramMessageCommandModel = {
	text: string
	emoji: string
	targetChatId: string
	restoreReason: string
	selectedMessageId: string
	pending: boolean
	statusMessage: string
	canCommand: boolean
}

export function useTelegramMessageCommands(input: {
	accountId: () => string
	canCommand: () => boolean
	providerChatId: () => string
	providerMessageId: () => string
}) {
	const text = ref('')
	const emoji = ref('👍')
	const targetChatId = ref('')
	const restoreReason = ref('owner_restore')
	const feedback = useTelegramCommandFeedback(input.canCommand)

	const model = computed<TelegramMessageCommandModel>(() => ({
		text: text.value,
		emoji: emoji.value,
		targetChatId: targetChatId.value,
		restoreReason: restoreReason.value,
		selectedMessageId: input.providerMessageId(),
		pending: feedback.pending.value,
		statusMessage: feedback.statusMessage.value,
		canCommand: input.canCommand(),
	}))

	async function reply(): Promise<void> {
		await feedback.run(() => replyToTelegramMessage(target(), text.value))
	}

	async function forward(): Promise<void> {
		await feedback.run(() => forwardTelegramMessage({
			...target(),
			targetProviderChatId: targetChatId.value,
		}))
	}

	async function edit(): Promise<void> {
		await feedback.run(() => editTelegramMessage(target(), text.value))
	}

	async function remove(): Promise<void> {
		const messageId = input.providerMessageId()
		if (!window.confirm(`Delete Telegram message ${messageId} for all participants?`)) {
			return
		}
		await feedback.run(() => deleteTelegramMessage(target(), true))
	}

	async function restore(): Promise<void> {
		await feedback.run(() => restoreTelegramMessageVisibility(target(), restoreReason.value))
	}

	async function react(active: boolean): Promise<void> {
		await feedback.run(() => setTelegramMessageReaction(target(), emoji.value, active))
	}

	async function pin(active: boolean): Promise<void> {
		await feedback.run(() => setTelegramMessagePinned(target(), active))
	}

	function updateText(value: string): void {
		text.value = value
	}

	function updateEmoji(value: string): void {
		emoji.value = value
	}

	function updateTargetChatId(value: string): void {
		targetChatId.value = value
	}

	function updateRestoreReason(value: string): void {
		restoreReason.value = value
	}

	function target(): TelegramMessageTarget {
		return {
			accountId: input.accountId(),
			providerChatId: input.providerChatId(),
			providerMessageId: input.providerMessageId(),
			operationId: crypto.randomUUID(),
		}
	}

	return {
		model,
		reply,
		forward,
		edit,
		remove,
		restore,
		react,
		pin,
		updateText,
		updateEmoji,
		updateTargetChatId,
		updateRestoreReason,
	}
}
