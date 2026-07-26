<script setup lang="ts">
import type { TelegramChatCommandModel } from './telegramChatCommandModel'
import type { TelegramMediaCommandModel } from '../queries/useTelegramMediaCommands'
import type { TelegramMessageCommandModel } from '../queries/useTelegramMessageCommands'
import type { TelegramTopicCommandModel } from '../queries/useTelegramTopicCommands'
import TelegramChatCommandPanel from './TelegramChatCommandPanel.vue'
import TelegramMediaCommandPanel from './TelegramMediaCommandPanel.vue'
import TelegramMessageCommandPanel from './TelegramMessageCommandPanel.vue'
import TelegramTopicCommandPanel from './TelegramTopicCommandPanel.vue'
import './telegramCommandPanels.css'

defineProps<{
	chat: TelegramChatCommandModel
	media: TelegramMediaCommandModel
	message: TelegramMessageCommandModel
	topic: TelegramTopicCommandModel
}>()

const emit = defineEmits<{
	chatAddToFolder: []
	chatArchive: [active: boolean]
	chatJoin: []
	chatLeave: []
	chatMarkUnread: [active: boolean]
	chatMute: [active: boolean]
	chatRemoveFromFolder: []
	chatReassignFolders: []
	mediaDownload: []
	mediaSend: []
	messageDelete: []
	messageEdit: []
	messageForward: []
	messagePin: [active: boolean]
	messageReact: [active: boolean]
	messageReply: []
	messageRestore: []
	topicClose: [active: boolean]
	topicCreate: []
	topicParticipants: []
	topicRefresh: []
	topicSearch: []
	updateChatFolderId: [value: string]
	updateChatTargetFolderIds: [value: string]
	updateMediaBlobRef: [value: string]
	updateMediaBackupClass: [value: string]
	updateMediaCaption: [value: string]
	updateMediaDeclaredSize: [value: string]
	updateMediaFilename: [value: string]
	updateMediaKind: [value: string]
	updateMediaProviderFileId: [value: string]
	updateMediaReferenceIdHex: [value: string]
	updateMessageEmoji: [value: string]
	updateMessageRestoreReason: [value: string]
	updateMessageTargetChatId: [value: string]
	updateMessageText: [value: string]
	updateTopicId: [value: string]
	updateTopicSearchQuery: [value: string]
	updateTopicTitle: [value: string]
}>()
</script>

<template>
	<section class="telegram-command-workbench">
		<header>
			<span>Provider commands</span>
			<h2>Telegram actions</h2>
			<p>Accepted operations complete asynchronously and remain visible in operation receipts.</p>
		</header>
		<div class="telegram-command-workbench__grid">
			<TelegramMessageCommandPanel
				:model="message"
				@delete="emit('messageDelete')"
				@edit="emit('messageEdit')"
				@forward="emit('messageForward')"
				@pin="emit('messagePin', $event)"
				@react="emit('messageReact', $event)"
				@reply="emit('messageReply')"
				@restore="emit('messageRestore')"
				@update-emoji="emit('updateMessageEmoji', $event)"
				@update-restore-reason="emit('updateMessageRestoreReason', $event)"
				@update-target-chat-id="emit('updateMessageTargetChatId', $event)"
				@update-text="emit('updateMessageText', $event)"
			/>
			<TelegramChatCommandPanel
				:model="chat"
				@add-to-folder="emit('chatAddToFolder')"
				@archive="emit('chatArchive', $event)"
				@join="emit('chatJoin')"
				@leave="emit('chatLeave')"
				@mark-unread="emit('chatMarkUnread', $event)"
				@mute="emit('chatMute', $event)"
				@remove-from-folder="emit('chatRemoveFromFolder')"
				@reassign-folders="emit('chatReassignFolders')"
				@update-folder-id="emit('updateChatFolderId', $event)"
				@update-target-folder-ids="emit('updateChatTargetFolderIds', $event)"
			/>
			<TelegramTopicCommandPanel
				:model="topic"
				@close-topic="emit('topicClose', $event)"
				@create-topic="emit('topicCreate')"
				@refresh-participants="emit('topicParticipants')"
				@refresh-topics="emit('topicRefresh')"
				@search-messages="emit('topicSearch')"
				@update-provider-search-query="emit('updateTopicSearchQuery', $event)"
				@update-topic-id="emit('updateTopicId', $event)"
				@update-topic-title="emit('updateTopicTitle', $event)"
			/>
			<TelegramMediaCommandPanel
				:model="media"
				@download-file="emit('mediaDownload')"
				@send-media="emit('mediaSend')"
				@update-blob-ref="emit('updateMediaBlobRef', $event)"
				@update-backup-class="emit('updateMediaBackupClass', $event)"
				@update-caption="emit('updateMediaCaption', $event)"
				@update-declared-size="emit('updateMediaDeclaredSize', $event)"
				@update-filename="emit('updateMediaFilename', $event)"
				@update-media-kind="emit('updateMediaKind', $event)"
				@update-provider-file-id="emit('updateMediaProviderFileId', $event)"
				@update-reference-id-hex="emit('updateMediaReferenceIdHex', $event)"
			/>
		</div>
	</section>
</template>
