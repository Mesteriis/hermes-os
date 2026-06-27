<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from '../../../../../platform/i18n'
import Icon from '../../../../../shared/ui/Icon.vue'
import type {
	WhatsappWebMediaItem,
} from '../../../../../shared/communications/types/whatsapp'
import { TELEGRAM_REACTION_PALETTE } from '../../../../../shared/communications/types/telegram'
import type { CommunicationProviderConversation } from '../../../types/providerChannels'
import {
	isPreviewableMediaItem,
	isStatusMessage,
	mediaLabel,
	mediaMetaLabel,
	mediaTime,
	messageContactCardSummary as buildMessageContactCardSummary,
	messageLinkPreview,
	messageLocationSummary as buildMessageLocationSummary,
	messageMentionNames,
	messageMetaFlags as buildMessageMetaFlags,
	messagePollSummary as buildMessagePollSummary,
	messageStickerSummary as buildMessageStickerSummary,
	messageSystemSummary as buildMessageSystemSummary,
	messageTime,
	reactionSummary,
	statusAuthorDetail,
	statusAuthorHeadline,
	statusDeletedSummary as buildStatusDeletedSummary,
	statusMediaCountLabel as buildStatusMediaCountLabel,
	statusMessageMediaItems as collectStatusMessageMediaItems,
	statusViewSummary as buildStatusViewSummary,
	type WhatsAppPanelMessage,
} from './WhatsAppCommunicationsPanel.helpers'

const props = defineProps<{
	selectedConversation: CommunicationProviderConversation | null
	browserMode: 'timeline' | 'media'
	selectedMessages: WhatsAppPanelMessage[]
	mediaItems: WhatsappWebMediaItem[]
	draftText: string
	emptyStateMessage: string
	isBusy: boolean
	isConversationUnread: boolean
	isConversationMuted: boolean
	isConversationArchived: boolean
	isConversationPinned: boolean
}>()

const emit = defineEmits<{
	(event: 'update:draftText', value: string): void
	(event: 'toggle-unread'): void
	(event: 'toggle-mute'): void
	(event: 'toggle-archive'): void
	(event: 'toggle-pin'): void
	(event: 'send-message'): void
	(event: 'set-message-ref', messageId: string, element: unknown): void
	(event: 'reply', message: WhatsAppPanelMessage): void
	(event: 'forward', message: WhatsAppPanelMessage): void
	(event: 'edit', message: WhatsAppPanelMessage): void
	(event: 'delete', message: WhatsAppPanelMessage): void
	(event: 'select-media', item: WhatsappWebMediaItem): void
	(event: 'jump-to-message', messageId: string): void
	(event: 'add-reaction', message: WhatsAppPanelMessage, reactionEmoji: string): void
	(event: 'remove-reaction', message: WhatsAppPanelMessage, reactionEmoji: string): void
}>()

const { t } = useI18n()

const draftTextModel = computed({
	get: () => props.draftText,
	set: (value: string) => emit('update:draftText', value),
})

function statusMessageMediaItems(message: WhatsAppPanelMessage): WhatsappWebMediaItem[] {
	return collectStatusMessageMediaItems(message, props.mediaItems)
}

function statusViewSummary(message: WhatsAppPanelMessage): string | null {
	return buildStatusViewSummary(message, t)
}

function statusDeletedSummary(message: WhatsAppPanelMessage): string | null {
	return buildStatusDeletedSummary(message, t)
}

function statusMediaCountLabel(message: WhatsAppPanelMessage): string | null {
	return buildStatusMediaCountLabel(message, props.mediaItems, t)
}

function messageMetaFlags(message: WhatsAppPanelMessage): string[] {
	return buildMessageMetaFlags(message, t)
}

function messagePollSummary(message: WhatsAppPanelMessage): string | null {
	return buildMessagePollSummary(message, t)
}

function messageLocationSummary(message: WhatsAppPanelMessage): string | null {
	return buildMessageLocationSummary(message, t)
}

function messageContactCardSummary(message: WhatsAppPanelMessage): string | null {
	return buildMessageContactCardSummary(message, t)
}

function messageStickerSummary(message: WhatsAppPanelMessage): string | null {
	return buildMessageStickerSummary(message, t)
}

function messageSystemSummary(message: WhatsAppPanelMessage): string | null {
	return buildMessageSystemSummary(message, t)
}
</script>

<template>
	<section class="panel chat-pane">
		<header class="provider-thread-header">
			<div>
				<h2>{{ selectedConversation?.title ?? t('No conversation selected') }}</h2>
				<p>{{ selectedConversation?.account_id ?? '' }}</p>
			</div>
			<div class="thread-actions">
				<button type="button" :disabled="isBusy || !selectedConversation?.conversation_id" @click="emit('toggle-unread')">
					<Icon :icon="isConversationUnread ? 'tabler:mail-opened' : 'tabler:mail'" width="14" height="14" />
					{{ isConversationUnread ? t('Mark read') : t('Mark unread') }}
				</button>
				<button type="button" :disabled="isBusy || !selectedConversation?.conversation_id" @click="emit('toggle-mute')">
					<Icon :icon="isConversationMuted ? 'tabler:volume' : 'tabler:volume-off'" width="14" height="14" />
					{{ isConversationMuted ? t('Unmute') : t('Mute') }}
				</button>
				<button type="button" :disabled="isBusy || !selectedConversation?.conversation_id" @click="emit('toggle-archive')">
					<Icon :icon="isConversationArchived ? 'tabler:archive-off' : 'tabler:archive'" width="14" height="14" />
					{{ isConversationArchived ? t('Unarchive') : t('Archive') }}
				</button>
				<button type="button" :disabled="isBusy || !selectedConversation?.conversation_id" @click="emit('toggle-pin')">
					<Icon :icon="isConversationPinned ? 'tabler:pinned-off' : 'tabler:pin'" width="14" height="14" />
					{{ isConversationPinned ? t('Unpin') : t('Pin') }}
				</button>
			</div>
		</header>
		<div class="message-scroll">
			<template v-if="browserMode === 'timeline'">
				<article
					v-for="message in selectedMessages"
					:key="message.message_id"
					class="message-bubble"
					:ref="(element) => emit('set-message-ref', message.message_id, element)"
				>
					<header>
						<strong>{{ message.sender_display_name ?? message.sender }}</strong>
						<time>{{ messageTime(message) }}</time>
					</header>
					<div v-if="messageMetaFlags(message).length" class="message-meta-flags">
						<span v-for="flag in messageMetaFlags(message)" :key="`${message.message_id}:${flag}`" class="message-meta-chip">
							{{ flag }}
						</span>
					</div>
					<p>{{ message.text }}</p>
					<ul v-if="messageMentionNames(message).length" class="message-meta-list">
						<li>
							<strong>{{ t('Mentions') }}</strong>
							<span>{{ messageMentionNames(message).join(', ') }}</span>
						</li>
					</ul>
					<ul v-if="isStatusMessage(message)" class="message-meta-list">
						<li v-if="statusAuthorHeadline(message)"><strong>{{ t('Status author') }}</strong><span>{{ statusAuthorHeadline(message) }}</span></li>
						<li v-if="statusAuthorDetail(message)"><strong>{{ t('Identity') }}</strong><span>{{ statusAuthorDetail(message) }}</span></li>
						<li v-if="statusViewSummary(message)"><strong>{{ t('Views') }}</strong><span>{{ statusViewSummary(message) }}</span></li>
						<li v-if="statusDeletedSummary(message)"><strong>{{ t('Lifecycle') }}</strong><span>{{ statusDeletedSummary(message) }}</span></li>
						<li v-if="statusMediaCountLabel(message)"><strong>{{ t('Status media') }}</strong><span>{{ statusMediaCountLabel(message) }}</span></li>
					</ul>
					<div v-if="messageLinkPreview(message)" class="message-meta-card">
						<strong>{{ messageLinkPreview(message)?.title ?? t('Link preview') }}</strong>
						<span>{{ messageLinkPreview(message)?.site ?? messageLinkPreview(message)?.url }}</span>
					</div>
					<ul
						v-if="messagePollSummary(message) || messageLocationSummary(message) || messageContactCardSummary(message) || messageStickerSummary(message) || messageSystemSummary(message)"
						class="message-meta-list"
					>
						<li v-if="messagePollSummary(message)"><strong>{{ t('Poll') }}</strong><span>{{ messagePollSummary(message) }}</span></li>
						<li v-if="messageLocationSummary(message)"><strong>{{ t('Location') }}</strong><span>{{ messageLocationSummary(message) }}</span></li>
						<li v-if="messageContactCardSummary(message)"><strong>{{ t('Contact') }}</strong><span>{{ messageContactCardSummary(message) }}</span></li>
						<li v-if="messageStickerSummary(message)"><strong>{{ t('Sticker') }}</strong><span>{{ messageStickerSummary(message) }}</span></li>
						<li v-if="messageSystemSummary(message)"><strong>{{ t('Event') }}</strong><span>{{ messageSystemSummary(message) }}</span></li>
					</ul>
					<div v-if="isStatusMessage(message) && statusMessageMediaItems(message).length" class="status-media-grid">
						<article
							v-for="item in statusMessageMediaItems(message)"
							:key="`${message.message_id}:${item.attachment_id ?? item.provider_attachment_id ?? item.file_name}`"
							class="status-media-card"
						>
							<strong>{{ mediaLabel(item) }}</strong>
							<span>{{ mediaMetaLabel(item) }}</span>
							<div class="status-media-actions">
								<button v-if="isPreviewableMediaItem(item)" type="button" :disabled="isBusy" @click="emit('select-media', item)">
									<Icon icon="tabler:photo-search" width="14" height="14" />{{ t('Preview media') }}
								</button>
								<button type="button" :disabled="isBusy" @click="emit('jump-to-message', item.message_id)">
									<Icon icon="tabler:arrow-up-right" width="14" height="14" />{{ t('Open source message') }}
								</button>
							</div>
						</article>
					</div>
					<div v-if="reactionSummary(message).length" class="message-reactions">
						<button
							v-for="reaction in reactionSummary(message)"
							:key="`${message.message_id}:${reaction.reaction_emoji}`"
							type="button"
							class="reaction-chip"
							:disabled="isBusy"
							@click="emit('remove-reaction', message, reaction.reaction_emoji)"
						>
							<span>{{ reaction.reaction_emoji }}</span>
							<span>{{ reaction.count }}</span>
						</button>
					</div>
					<footer>
						<button type="button" :disabled="isBusy || !draftTextModel.trim()" @click="emit('reply', message)">
							<Icon icon="tabler:message-reply" width="14" height="14" />{{ t('Reply') }}
						</button>
						<button type="button" :disabled="isBusy" @click="emit('forward', message)">
							<Icon icon="tabler:arrow-forward-up" width="14" height="14" />{{ t('Forward') }}
						</button>
						<button type="button" :disabled="isBusy" @click="emit('edit', message)">
							<Icon icon="tabler:edit" width="14" height="14" />{{ t('Edit') }}
						</button>
						<button type="button" :disabled="isBusy" @click="emit('delete', message)">
							<Icon icon="tabler:trash" width="14" height="14" />{{ t('Delete') }}
						</button>
					</footer>
					<div class="reaction-palette">
						<button
							v-for="reactionEmoji in TELEGRAM_REACTION_PALETTE.slice(0, 8)"
							:key="`${message.message_id}:palette:${reactionEmoji}`"
							type="button"
							class="reaction-chip add"
							:disabled="isBusy"
							@click="emit('add-reaction', message, reactionEmoji)"
						>
							{{ reactionEmoji }}
						</button>
					</div>
				</article>
			</template>
			<div v-else class="media-gallery">
				<article v-for="item in mediaItems" :key="`${item.message_id}:${item.provider_attachment_id ?? item.file_name}`" class="media-card">
					<header>
						<strong>{{ mediaLabel(item) }}</strong>
						<time>{{ mediaTime(item) }}</time>
					</header>
					<p>{{ mediaMetaLabel(item) }}</p>
					<footer>
						<button v-if="isPreviewableMediaItem(item)" type="button" :disabled="isBusy" @click="emit('select-media', item)">
							<Icon icon="tabler:photo-search" width="14" height="14" />{{ t('Preview media') }}
						</button>
						<button type="button" :disabled="isBusy" @click="emit('jump-to-message', item.message_id)">
							<Icon icon="tabler:arrow-up-right" width="14" height="14" />{{ t('Open source message') }}
						</button>
					</footer>
				</article>
			</div>
			<div v-if="(browserMode === 'timeline' && !selectedMessages.length) || (browserMode === 'media' && !mediaItems.length)" class="empty-panel">
				{{ emptyStateMessage }}
			</div>
		</div>
		<form class="provider-inline-form" @submit.prevent="emit('send-message')">
			<input v-model="draftTextModel" type="text" :placeholder="t('Write a message')" autocomplete="off" />
			<button type="submit" :disabled="isBusy || !selectedConversation || !draftTextModel.trim()">
				<Icon icon="tabler:send" width="16" height="16" />{{ t('Send') }}
			</button>
		</form>
	</section>
</template>
