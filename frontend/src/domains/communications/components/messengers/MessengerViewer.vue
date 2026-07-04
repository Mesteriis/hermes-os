<script setup lang="ts">
import { useI18n } from '@/platform/i18n'
import { AttachmentChip, Badge, MessageBubble, MessageStatus, ProviderIcon } from '@/shared/ui'
import '../communicationDomainElements.css'
import type { MessengerConversationModel } from './messengerElements'
import {
  messengerChannelLabel,
  messengerChannelProviderIcon,
  messengerConversationKindLabel,
  messengerMessageAuthor,
  messengerMessageMeta,
  messengerMessageTimestamp,
  messengerWorkflowStatusPresentation
} from './messengerElements'
import MessengerRichEditor from './MessengerRichEditor.vue'

defineProps<{
  conversation: MessengerConversationModel
}>()

const { t } = useI18n()
</script>

<template>
	<section class="messenger-viewer" :aria-label="t('Open dialog')">
		<header class="messenger-viewer__header">
			<div class="messenger-viewer__identity">
				<ProviderIcon
					:provider="messengerChannelProviderIcon(conversation.channelKind)"
					:label="messengerChannelLabel(conversation.channelKind)"
					class="messenger-viewer__provider"
				/>
				<div class="messenger-viewer__title-group">
					<h2 class="messenger-viewer__title">{{ conversation.title }}</h2>
					<p class="messenger-viewer__subtitle">
						{{ messengerChannelLabel(conversation.channelKind) }} · {{ t(messengerConversationKindLabel(conversation.kind)) }}
					</p>
				</div>
			</div>
			<Badge :variant="messengerWorkflowStatusPresentation(conversation.workflowState).tone">
				{{ t(messengerWorkflowStatusPresentation(conversation.workflowState).label) }}
			</Badge>
		</header>

		<div class="messenger-viewer__facts" :aria-label="t('Dialog facts')">
			<span class="messenger-viewer__fact">{{ conversation.participantsLabel }}</span>
			<span v-if="conversation.lastSeenLabel" class="messenger-viewer__fact">{{ conversation.lastSeenLabel }}</span>
			<span
				v-for="fact in conversation.facts"
				:key="fact.label"
				:class="['messenger-viewer__fact', fact.tone && `messenger-viewer__fact--${fact.tone}`]"
			>
				{{ t(fact.label) }}: {{ fact.value }}
			</span>
		</div>

		<div class="messenger-viewer__messages">
			<MessageBubble
				v-for="message in conversation.messages"
				:key="message.id"
				:author="messengerMessageAuthor(message)"
				:direction="message.direction"
				:meta="messengerMessageMeta(message)"
				:timestamp="messengerMessageTimestamp(message)"
				:tone="message.tone"
				:selected="message.selected"
				:pending="message.pending"
			>
				<p>{{ message.body }}</p>
				<div v-if="message.attachments?.length" class="messenger-viewer__attachments">
					<AttachmentChip
						v-for="attachment in message.attachments"
						:key="attachment.id"
						:name="attachment.name"
						:meta="attachment.meta"
						:icon="attachment.icon"
						:tone="attachment.tone"
					/>
				</div>
				<template #footer>
					<MessageStatus v-if="message.direction === 'outbound'" status="delivered" />
				</template>
			</MessageBubble>
		</div>

		<footer class="messenger-viewer__composer">
			<MessengerRichEditor :conversation="conversation" />
		</footer>
	</section>
</template>
