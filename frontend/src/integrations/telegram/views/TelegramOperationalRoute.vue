<script setup lang="ts">
import TelegramOperationalPage from '../presentation/TelegramOperationalPage.vue'
import TelegramCommandWorkbench from '../presentation/TelegramCommandWorkbench.vue'
import TelegramMessageInspector from '../presentation/TelegramMessageInspector.vue'
import TelegramOperationRetryPanel from '../presentation/TelegramOperationRetryPanel.vue'
import { useTelegramOperationalPage } from '../queries/useTelegramOperationalPage'
import { useTelegramAccountAccess } from '../queries/useTelegramAccountAccess'
import { useTelegramChatCommands } from '../queries/useTelegramChatCommands'
import { useTelegramDiscovery } from '../queries/useTelegramDiscovery'
import { useTelegramMediaCommands } from '../queries/useTelegramMediaCommands'
import { useTelegramMessageCommands } from '../queries/useTelegramMessageCommands'
import { useTelegramMessageInspector } from '../queries/useTelegramMessageInspector'
import { useTelegramOperationRetry } from '../queries/useTelegramOperationRetry'
import { useTelegramTopicCommands } from '../queries/useTelegramTopicCommands'

const props = defineProps<{
	canAuthorize: boolean
	canManageLifecycle: boolean
	canQuery: boolean
	canSend: boolean
}>()
const surface = useTelegramOperationalPage(() => props.canSend)
const accountAccess = useTelegramAccountAccess({
	canAuthorize: () => props.canAuthorize,
	canManageLifecycle: () => props.canManageLifecycle,
})
const discovery = useTelegramDiscovery({
	accountId: () => accountAccess.selectedAccountId.value,
	canQuery: () => props.canQuery,
	selectedChatId: () => surface.model.value.selectedChatId,
})
const commandTarget = {
	accountId: () => accountAccess.selectedAccountId.value,
	canCommand: () => props.canSend,
	providerChatId: () => surface.model.value.selectedChatId,
}
const messageCommands = useTelegramMessageCommands({
	...commandTarget,
	providerMessageId: () => surface.model.value.selectedProviderMessageId,
})
const chatCommands = useTelegramChatCommands(commandTarget)
const topicCommands = useTelegramTopicCommands(commandTarget)
const mediaCommands = useTelegramMediaCommands(commandTarget)
const messageInspector = useTelegramMessageInspector({
	accountId: () => accountAccess.selectedAccountId.value,
	canQuery: () => props.canQuery,
	messageId: () => surface.model.value.selectedMessageId,
	providerChatId: () => surface.model.value.selectedChatId,
	providerMessageId: () => surface.model.value.selectedProviderMessageId,
})
const operationRetry = useTelegramOperationRetry(() => props.canManageLifecycle)

async function refreshAccounts(): Promise<void> {
	await accountAccess.refresh()
	updateAccountId(accountAccess.selectedAccountId.value)
}

async function selectAccount(accountId: string): Promise<void> {
	accountAccess.selectAccount(accountId)
	updateAccountId(accountId)
	await surface.loadChats()
}

async function selectChat(providerChatId: string): Promise<void> {
	await surface.selectChat(providerChatId)
}

function updateAccountId(accountId: string): void {
	accountAccess.selectAccount(accountId)
	surface.updateAccountId(accountId)
}
</script>

<template>
	<TelegramOperationalPage
		:account-access="accountAccess.model.value"
		:discovery="discovery.model.value"
		:model="surface.model.value"
		@provision-account="accountAccess.provision"
		@refresh-accounts="refreshAccounts"
		@refresh-chat-context="discovery.refreshChatContext"
		@replay-account="accountAccess.replay"
		@restart-account="accountAccess.restart"
		@retire-account="accountAccess.retire"
		@select-account="selectAccount"
		@start-account="accountAccess.start"
		@stop-account="accountAccess.stop"
		@submit-authorization-password="accountAccess.submitPassword"
		@load="surface.loadChats"
		@search="discovery.search"
		@select-chat="selectChat"
		@select-message="surface.selectMessage"
		@send="surface.send"
		@update-account-id="updateAccountId"
		@update-authorization-password="accountAccess.updatePassword"
		@update-draft="surface.updateDraft"
		@update-search-query="discovery.updateQuery"
		@update-provision-account-id="accountAccess.updateProvisionAccountId"
		@update-provision-display-name="accountAccess.updateProvisionDisplayName"
		@update-provision-external-account-id="accountAccess.updateProvisionExternalAccountId"
	/>
	<TelegramMessageInspector
		:model="messageInspector.model.value"
		@inspect="messageInspector.inspect"
	/>
	<TelegramOperationRetryPanel
		:model="operationRetry.model.value"
		@retry="operationRetry.retry"
		@update-operation-id="operationRetry.updateOperationId"
	/>
	<TelegramCommandWorkbench
		:chat="chatCommands.model.value"
		:media="mediaCommands.model.value"
		:message="messageCommands.model.value"
		:topic="topicCommands.model.value"
		@chat-add-to-folder="chatCommands.addToFolder"
		@chat-archive="chatCommands.archive"
		@chat-join="chatCommands.join"
		@chat-leave="chatCommands.leave"
		@chat-mark-unread="chatCommands.markUnread"
		@chat-mute="chatCommands.mute"
		@chat-remove-from-folder="chatCommands.removeFromFolder"
		@media-download="mediaCommands.downloadFile"
		@media-send="mediaCommands.sendMedia"
		@message-delete="messageCommands.remove"
		@message-edit="messageCommands.edit"
		@message-forward="messageCommands.forward"
		@message-pin="messageCommands.pin"
		@message-react="messageCommands.react"
		@message-reply="messageCommands.reply"
		@message-restore="messageCommands.restore"
		@topic-close="topicCommands.closeTopic"
		@topic-create="topicCommands.createTopic"
		@topic-participants="topicCommands.refreshParticipants"
		@topic-refresh="topicCommands.refreshTopics"
		@topic-search="topicCommands.searchMessages"
		@update-chat-folder-id="chatCommands.updateFolderId"
		@update-media-blob-ref="mediaCommands.updateBlobRef"
		@update-media-backup-class="mediaCommands.updateBackupClass"
		@update-media-caption="mediaCommands.updateCaption"
		@update-media-declared-size="mediaCommands.updateDeclaredSize"
		@update-media-filename="mediaCommands.updateFilename"
		@update-media-kind="mediaCommands.updateMediaKind"
		@update-media-provider-file-id="mediaCommands.updateProviderFileId"
		@update-media-reference-id-hex="mediaCommands.updateReferenceIdHex"
		@update-message-emoji="messageCommands.updateEmoji"
		@update-message-restore-reason="messageCommands.updateRestoreReason"
		@update-message-target-chat-id="messageCommands.updateTargetChatId"
		@update-message-text="messageCommands.updateText"
		@update-topic-id="topicCommands.updateTopicId"
		@update-topic-search-query="topicCommands.updateProviderSearchQuery"
		@update-topic-title="topicCommands.updateTopicTitle"
	/>
</template>
