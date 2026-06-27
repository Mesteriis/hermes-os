<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from '../../../../../platform/i18n'
import Icon from '../../../../../shared/ui/Icon.vue'
import type {
	WhatsappWebMediaItem,
} from '../../../../../shared/communications/types/whatsapp'
import type { TelegramChatMember } from '../../../../../shared/communications/types/telegramMembers'
import type { AttachmentPreviewResponse } from '../../../types/attachments'
import type { CommunicationProviderConversation } from '../../../types/providerChannels'
import {
	isPreviewableMediaItem,
	mediaLabel,
	mediaMetaLabel,
	memberLabel,
	messagePreview as buildMessagePreview,
	type WhatsAppPanelMessage,
} from './WhatsAppCommunicationsPanel.helpers'

const props = defineProps<{
	selectedConversation: CommunicationProviderConversation | null
	isStatusFeedConversation: boolean
	isConversationUnread: boolean
	isConversationArchived: boolean
	isConversationMuted: boolean
	isConversationPinned: boolean
	participantCount: number
	editingMessage: WhatsAppPanelMessage | null
	editDraftText: string
	forwardingMessage: WhatsAppPanelMessage | null
	forwardTargetFilter: string
	forwardTargetConversationId: string
	forwardTargetConversations: CommunicationProviderConversation[]
	members: TelegramChatMember[]
	pinnedMessages: WhatsAppPanelMessage[]
	mediaItems: WhatsappWebMediaItem[]
	selectedMediaItem: WhatsappWebMediaItem | null
	mediaPreview: AttachmentPreviewResponse | null
	mediaPreviewError: string
	isMediaPreviewFetching: boolean
	isBusy: boolean
}>()

const emit = defineEmits<{
	(event: 'update:editDraftText', value: string): void
	(event: 'update:forwardTargetFilter', value: string): void
	(event: 'update:forwardTargetConversationId', value: string): void
	(event: 'confirm-edit'): void
	(event: 'cancel-edit'): void
	(event: 'confirm-forward'): void
	(event: 'cancel-forward'): void
	(event: 'select-media', item: WhatsappWebMediaItem): void
	(event: 'clear-media-preview'): void
	(event: 'jump-to-message', messageId: string): void
}>()

const { t } = useI18n()

const editDraftTextModel = computed({
	get: () => props.editDraftText,
	set: (value: string) => emit('update:editDraftText', value),
})
const forwardTargetFilterModel = computed({
	get: () => props.forwardTargetFilter,
	set: (value: string) => emit('update:forwardTargetFilter', value),
})
const forwardTargetConversationIdModel = computed({
	get: () => props.forwardTargetConversationId,
	set: (value: string) => emit('update:forwardTargetConversationId', value),
})

function messagePreview(message: { text?: string; body_text_preview?: string | null }): string {
	return buildMessagePreview(message, t)
}
</script>

<template>
	<aside class="panel detail-pane">
		<header class="provider-panel-header">
			<h2>{{ t('Details') }}</h2>
		</header>
		<div class="detail-scroll">
			<section class="detail-section">
				<dl class="detail-list">
					<div><dt>{{ t('Kind') }}</dt><dd>{{ isStatusFeedConversation ? t('status_feed') : (selectedConversation?.chat_kind ?? t('unknown')) }}</dd></div>
					<div><dt>{{ t('Unread') }}</dt><dd>{{ isConversationUnread ? t('Yes') : t('No') }}</dd></div>
					<div><dt>{{ t('Participants') }}</dt><dd>{{ participantCount }}</dd></div>
					<div><dt>{{ t('Archived') }}</dt><dd>{{ isConversationArchived ? t('Yes') : t('No') }}</dd></div>
					<div><dt>{{ t('Muted') }}</dt><dd>{{ isConversationMuted ? t('Yes') : t('No') }}</dd></div>
					<div><dt>{{ t('Pinned chat') }}</dt><dd>{{ isConversationPinned ? t('Yes') : t('No') }}</dd></div>
				</dl>
			</section>

			<section class="detail-section">
				<h3>{{ t('Edit draft') }}</h3>
				<div v-if="editingMessage" class="forward-panel">
					<p class="forward-panel-copy">
						{{ editingMessage.sender_display_name ?? editingMessage.sender }}:
						{{ messagePreview(editingMessage) }}
					</p>
					<label class="runtime-field">
						<span>{{ t('Edited text') }}</span>
						<textarea v-model="editDraftTextModel" rows="4" maxlength="4000" :placeholder="t('Update message text')" />
					</label>
					<div class="thread-actions">
						<button type="button" :disabled="isBusy || !editDraftTextModel.trim()" @click="emit('confirm-edit')">
							<Icon icon="tabler:check" width="14" height="14" />{{ t('Save edit') }}
						</button>
						<button type="button" :disabled="isBusy" @click="emit('cancel-edit')">
							<Icon icon="tabler:x" width="14" height="14" />{{ t('Cancel') }}
						</button>
					</div>
				</div>
				<div v-else class="empty-panel compact">{{ t('Choose Edit on a message to change it here.') }}</div>
			</section>

			<section class="detail-section">
				<h3>{{ t('Forward target') }}</h3>
				<div v-if="forwardingMessage" class="forward-panel">
					<p class="forward-panel-copy">
						{{ forwardingMessage.sender_display_name ?? forwardingMessage.sender }}:
						{{ messagePreview(forwardingMessage) }}
					</p>
					<label class="provider-search compact">
						<Icon icon="tabler:search" width="14" height="14" />
						<input v-model="forwardTargetFilterModel" type="search" :placeholder="t('Filter target conversations')" />
					</label>
					<div class="forward-target-list">
						<button
							v-for="conversation in forwardTargetConversations.slice(0, 10)"
							:key="conversation.conversation_id ?? conversation.provider_chat_id"
							type="button"
							class="provider-row compact"
							:class="{ active: forwardTargetConversationIdModel === conversation.provider_chat_id }"
							@click="forwardTargetConversationIdModel = conversation.provider_chat_id"
						>
							<strong>{{ conversation.title }}</strong>
							<span>{{ conversation.provider_chat_id }}</span>
						</button>
					</div>
					<div class="thread-actions">
						<button type="button" :disabled="isBusy || !forwardTargetConversationIdModel.trim()" @click="emit('confirm-forward')">
							<Icon icon="tabler:send-2" width="14" height="14" />{{ t('Forward here') }}
						</button>
						<button type="button" :disabled="isBusy" @click="emit('cancel-forward')">
							<Icon icon="tabler:x" width="14" height="14" />{{ t('Cancel') }}
						</button>
					</div>
				</div>
				<div v-else class="empty-panel compact">{{ t('Choose Forward on a message to pick a target conversation here.') }}</div>
			</section>

			<section class="detail-section">
				<h3>{{ t('Members') }}</h3>
				<ul v-if="members.length" class="detail-stack">
					<li v-for="member in members.slice(0, 8)" :key="member.provider_member_id">
						<strong>{{ memberLabel(member) }}</strong>
						<span>{{ member.role ?? member.status ?? t('member') }}</span>
					</li>
				</ul>
				<div v-else class="empty-panel compact">{{ t('No projected members yet.') }}</div>
			</section>

			<section class="detail-section">
				<h3>{{ t('Pinned messages') }}</h3>
				<ul v-if="pinnedMessages.length" class="detail-stack">
					<li v-for="message in pinnedMessages.slice(0, 5)" :key="message.message_id">
						<strong>{{ message.sender_display_name ?? message.sender }}</strong>
						<span>{{ messagePreview(message) }}</span>
						<button type="button" :disabled="isBusy" @click="emit('jump-to-message', message.message_id)">
							<Icon icon="tabler:arrow-up-right" width="14" height="14" />{{ t('Jump to message') }}
						</button>
					</li>
				</ul>
				<div v-else class="empty-panel compact">{{ t('No pinned messages.') }}</div>
			</section>

			<section class="detail-section">
				<h3>{{ t('Media') }}</h3>
				<ul v-if="mediaItems.length" class="detail-stack">
					<li v-for="item in mediaItems.slice(0, 6)" :key="`${item.message_id}:${item.provider_attachment_id ?? item.file_name}`">
						<strong>{{ mediaLabel(item) }}</strong>
						<span>{{ item.kind }}{{ item.mime_type ? ` · ${item.mime_type}` : '' }}</span>
						<button v-if="isPreviewableMediaItem(item)" type="button" :disabled="isBusy" @click="emit('select-media', item)">
							<Icon icon="tabler:photo-search" width="14" height="14" />{{ t('Preview media') }}
						</button>
						<button type="button" :disabled="isBusy" @click="emit('jump-to-message', item.message_id)">
							<Icon icon="tabler:arrow-up-right" width="14" height="14" />{{ t('Open source message') }}
						</button>
					</li>
				</ul>
				<div v-else class="empty-panel compact">{{ t('No projected media yet.') }}</div>
			</section>

			<section class="detail-section">
				<h3>{{ t('Media preview') }}</h3>
				<div v-if="selectedMediaItem" class="media-preview-panel">
					<div class="media-preview-header">
						<div>
							<strong>{{ mediaLabel(selectedMediaItem) }}</strong>
							<span>{{ mediaMetaLabel(selectedMediaItem) }}</span>
						</div>
						<button type="button" :disabled="isBusy" @click="emit('clear-media-preview')">
							<Icon icon="tabler:x" width="14" height="14" />{{ t('Close preview') }}
						</button>
					</div>
					<p v-if="isMediaPreviewFetching" class="media-preview-copy">{{ t('Loading safe media preview') }}</p>
					<p v-else-if="mediaPreviewError" class="media-preview-copy error">{{ mediaPreviewError }}</p>
					<div v-else-if="mediaPreview" class="media-preview-body">
						<img
							v-if="mediaPreview.preview_kind === 'image' && mediaPreview.data_url"
							class="media-preview-image"
							:src="mediaPreview.data_url"
							:alt="mediaLabel(selectedMediaItem)"
						/>
						<audio v-else-if="mediaPreview.preview_kind === 'audio' && mediaPreview.data_url" class="media-preview-media" controls preload="metadata" :src="mediaPreview.data_url" />
						<video v-else-if="mediaPreview.preview_kind === 'video' && mediaPreview.data_url" class="media-preview-media" controls preload="metadata" :src="mediaPreview.data_url" />
						<iframe v-else-if="mediaPreview.preview_kind === 'pdf' && mediaPreview.data_url" class="media-preview-document" :src="mediaPreview.data_url" :title="mediaLabel(selectedMediaItem)" />
						<pre v-else class="media-preview-text">{{ mediaPreview.text }}</pre>
						<p class="media-preview-copy">
							{{ mediaPreview.truncated ? t('Preview is truncated to the safe local limit.') : t('Preview loaded from the local attachment blob.') }}
						</p>
					</div>
					<p v-else class="media-preview-copy">{{ t('Select previewable media to open it here.') }}</p>
				</div>
				<div v-else class="empty-panel compact">{{ t('Select previewable media to open it here.') }}</div>
			</section>
		</div>
	</aside>
</template>
