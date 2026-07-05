<script setup lang="ts">
import { AttachmentChip, Badge, Button, Icon, MessageBubble } from '@/shared/ui'
import {
  communicationChannelProviderIconName,
  type CommunicationConversationMessageModel,
  type CommunicationChannelWorkspaceModel
} from '../communicationDomainElements'
import '../communicationDomainElements.css'

defineProps<{
  workspace: CommunicationChannelWorkspaceModel
}>()

function channelMessageAuthor(message: CommunicationConversationMessageModel): string | undefined {
  return message.direction === 'system' ? undefined : message.author
}

function channelMessageTimestamp(message: CommunicationConversationMessageModel): string | undefined {
  return message.direction === 'system' ? undefined : message.timestamp
}

function channelMessageMeta(message: CommunicationConversationMessageModel): string | undefined {
  return message.direction === 'system' ? undefined : message.meta
}
</script>

<template>
	<section class="communication-channel-stream" aria-label="Channel stream">
		<header class="communication-channel-stream__header">
			<div class="communication-channel-stream__identity">
				<span class="communication-channel-stream__provider">
					<Icon :icon="communicationChannelProviderIconName(workspace.activeProviderKind)" size="1.15rem" />
				</span>
				<div>
					<h2 class="communication-conversation__title">#{{ workspace.activeRoomLabel }}</h2>
					<p class="communication-conversation__subtitle">
						{{ workspace.activeProviderLabel }} · {{ workspace.activeAccountLabel }}
					</p>
				</div>
			</div>
			<Badge variant="accent">{{ workspace.activeTopicLabel }}</Badge>
			<p class="communication-channel-stream__description">{{ workspace.activeRoomDescription }}</p>
		</header>

		<nav class="communication-channel-topics" aria-label="Channel topics">
			<button
				v-for="topic in workspace.topics"
				:key="topic.id"
				type="button"
				:class="['communication-channel-topic', topic.selected && 'communication-channel-topic--selected']"
			>
				<span class="communication-channel-topic__label">{{ topic.label }}</span>
				<span class="communication-channel-topic__summary">{{ topic.summary }}</span>
				<Badge :variant="topic.tone ?? 'neutral'">{{ topic.messageCountLabel }}</Badge>
			</button>
		</nav>

		<div class="communication-channel-stream__messages" aria-label="Channel messages">
			<MessageBubble
				v-for="message in workspace.messages"
				:key="message.id"
				:author="channelMessageAuthor(message)"
				:direction="message.direction"
				:meta="channelMessageMeta(message)"
				:timestamp="channelMessageTimestamp(message)"
				:tone="message.tone"
			>
				<p>{{ message.body }}</p>
				<div v-if="message.attachments?.length" class="communication-channel-message__attachments">
					<AttachmentChip
						v-for="attachment in message.attachments"
						:key="attachment.id"
						:name="attachment.name"
						:meta="attachment.meta"
						:icon="attachment.icon"
						:tone="attachment.tone"
					/>
				</div>
			</MessageBubble>
		</div>

		<footer class="communication-channel-composer" aria-label="Channel composer">
			<div class="communication-channel-composer__tools" role="toolbar" aria-label="Channel composer tools">
				<Button
					v-for="capability in workspace.composerCapabilities"
					:key="capability.id"
					class="communication-channel-composer__tool hermes-icon-button"
					variant="outline"
					size="sm"
					:icon="capability.icon"
					:disabled="capability.disabled"
					:aria-label="capability.label"
					:title="capability.label"
				/>
			</div>
			<div class="communication-channel-composer__input-shell">
				<textarea
					class="communication-channel-composer__input"
					rows="2"
					:placeholder="workspace.composerPlaceholder"
					aria-label="Channel message"
				/>
				<Button
					class="communication-channel-composer__send hermes-icon-button"
					variant="default"
					size="sm"
					icon="tabler:send-2"
					aria-label="Send channel message"
					title="Send channel message"
				/>
			</div>
		</footer>
	</section>
</template>
