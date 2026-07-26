import { computed, ref } from 'vue'

import {
	addTelegramChatToFolder,
	joinTelegramChat,
	leaveTelegramChat,
	reassignTelegramChatFolders,
	removeTelegramChatFromFolder,
	setTelegramChatArchived,
	setTelegramChatMuted,
	setTelegramChatUnread,
} from '../api/telegramChatCommandGateway'
import type { TelegramChatTarget } from '../api/telegramChatCommandGateway'
import type { TelegramChatCommandModel } from '../presentation/telegramChatCommandModel'
import { useTelegramCommandFeedback } from './useTelegramCommandFeedback'

export function useTelegramChatCommands(input: {
	accountId: () => string
	canCommand: () => boolean
	providerChatId: () => string
}) {
	const folderId = ref('')
	const targetFolderIds = ref('')
	const feedback = useTelegramCommandFeedback(input.canCommand)
	const model = computed<TelegramChatCommandModel>(() => ({
		folderId: folderId.value,
		targetFolderIds: targetFolderIds.value,
		pending: feedback.pending.value,
		statusMessage: feedback.statusMessage.value,
		canCommand: input.canCommand(),
		hasChat: Boolean(input.providerChatId()),
	}))

	async function markUnread(unread: boolean): Promise<void> {
		await feedback.run(() => setTelegramChatUnread(target(), unread))
	}

	async function archive(archived: boolean): Promise<void> {
		await feedback.run(() => setTelegramChatArchived(target(), archived))
	}

	async function mute(muted: boolean): Promise<void> {
		await feedback.run(() => setTelegramChatMuted(target(), muted))
	}

	async function join(): Promise<void> {
		await feedback.run(() => joinTelegramChat(target()))
	}

	async function leave(): Promise<void> {
		if (!window.confirm(`Leave Telegram chat ${input.providerChatId()}?`)) {
			return
		}
		await feedback.run(() => leaveTelegramChat(target()))
	}

	async function addToFolder(): Promise<void> {
		await feedback.run(() => addTelegramChatToFolder(target(), parseFolderId()))
	}

	async function removeFromFolder(): Promise<void> {
		await feedback.run(() => removeTelegramChatFromFolder(target(), parseFolderId()))
	}

	async function reassignFolders(): Promise<void> {
		await feedback.run(() => reassignTelegramChatFolders(target(), parseTargetFolderIds()))
	}

	function updateFolderId(value: string): void {
		folderId.value = value
	}

	function updateTargetFolderIds(value: string): void {
		targetFolderIds.value = value
	}

	function target(): TelegramChatTarget {
		return {
			accountId: input.accountId(),
			providerChatId: input.providerChatId(),
			operationId: crypto.randomUUID(),
		}
	}

	function parseFolderId(): bigint {
		if (!/^\d+$/.test(folderId.value.trim())) {
			throw new RangeError('Telegram folder ID must be a non-negative integer')
		}
		return BigInt(folderId.value.trim())
	}

	function parseTargetFolderIds(): bigint[] {
		const values = targetFolderIds.value
			.trim()
			.split(/[\s,]+/)
			.filter(Boolean)
		if (values.length === 0 || values.some((value) => !/^\d+$/.test(value))) {
			throw new RangeError('Telegram target folder IDs must be positive integers')
		}
		return values.map(BigInt)
	}

	return {
		model,
		markUnread,
		archive,
		mute,
		join,
		leave,
		addToFolder,
		removeFromFolder,
		reassignFolders,
		updateFolderId,
		updateTargetFolderIds,
	}
}
