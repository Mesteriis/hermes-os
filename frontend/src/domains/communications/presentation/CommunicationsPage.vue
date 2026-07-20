<script setup lang="ts">
import MailWorkspace from '../components/mail/MailWorkspace.vue'
import MessengerWorkspace from '../components/messengers/MessengerWorkspace.vue'
import type { CommunicationsPageActions, CommunicationsPageModel } from './communicationsPageModel'

defineProps<{
	model: CommunicationsPageModel
	actions: CommunicationsPageActions
}>()
</script>

<template>
	<section class="communications-workspace-view" aria-label="Communications workspace">
		<MailWorkspace
			v-if="model.channel === 'mail'"
			:items="model.items"
			:conversation="model.conversation"
			:has-more-items="model.hasMoreItems"
			:inspector="model.inspector"
			:is-importing="model.isImporting"
			:compose-error="model.composeError"
			:compose-account-options="model.composeAccountOptions"
			:compose-form="model.composeForm"
			:compose-open="model.composeOpen"
			:compose-status="model.composeStatus"
			:is-action-running="model.isActionRunning"
			:is-loading-more="model.isLoadingMore"
			:is-sending="model.isSending"
			:search-query="model.searchQuery"
			:sync-status="model.syncStatus"
			@attach-compose-files="actions.attachComposeFiles"
			@close-compose="actions.closeCompose"
			@import-mail-file="actions.importMailFile"
			@load-more="actions.loadMoreMail"
			@new-message="actions.newMailMessage"
			@refresh="actions.refreshMail"
			@remove-compose-attachment="actions.removeComposeAttachment"
			@save-compose="actions.saveCompose"
			@select-action="actions.selectMailAction"
			@select-message="actions.selectMailMessage"
			@send-compose="actions.sendCompose"
			@update-search-query="actions.updateMailSearch"
			@update-compose="actions.updateCompose"
			@visible-items-change="actions.setVisibleMailItemIds"
		/>

		<MessengerWorkspace
			v-else-if="model.channel === 'telegram'"
			:items="model.items"
			:conversation="model.conversation"
			:inspector="model.inspector"
			:is-action-running="model.isActionRunning"
			:is-list-loading="model.isListLoading"
			:is-list-refreshing="model.isListRefreshing"
			:is-loading-older="model.isLoadingOlder"
			:list-error="model.listError"
			:selected-message-id="model.selectedMessageId"
			:telegram-chat="model.telegramChat"
			:telegram-message="model.telegramMessage"
			:runtime-action-runner="model.runtimeActionRunner"
			@conversation-action="actions.runTelegramAction($event)"
			@refresh="actions.refreshTelegram"
			@select-conversation="actions.selectTelegramConversation"
			@select-message="actions.selectTelegramMessage"
			@submit="actions.submitTelegram"
			@upload-file="(file, caption) => actions.runTelegramAction('upload_media', file, caption)"
			@download-attachment="actions.downloadTelegramAttachment"
			@load-older="actions.loadOlderTelegram"
			@messages-visible="actions.markTelegramMessagesVisible"
		/>

		<MessengerWorkspace
			v-else
			:items="model.items"
			:conversation="model.conversation"
			:inspector="model.inspector"
			@select-conversation="actions.selectWhatsappConversation"
		/>
	</section>
</template>
