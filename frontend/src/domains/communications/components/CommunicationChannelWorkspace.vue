<script setup lang="ts">
import { Badge, EntityIcon, Icon, MessageBubble } from '@/shared/ui'
import type { CommunicationChannelWorkspaceModel } from './communicationDomainElements'
import './communicationDomainElements.css'

defineProps<{
  workspace: CommunicationChannelWorkspaceModel
}>()
</script>

<template>
	<section class="communication-channel-workspace">
		<aside class="communication-channel-rail" aria-label="Channel rooms">
			<header class="communication-channel-rail__header">
				<h2 class="communication-workspace-panel__title">{{ workspace.title }}</h2>
				<p class="communication-workspace-panel__meta">{{ workspace.subtitle }}</p>
			</header>
			<div class="communication-channel-rail__rooms">
				<button
					v-for="room in workspace.rooms"
					:key="room.id"
					type="button"
					:class="['communication-channel-room', room.selected && 'communication-channel-room--selected']"
				>
					<span class="communication-channel-room__hash">#</span>
					<span class="communication-channel-room__body">
						<strong>{{ room.label }}</strong>
						<small>{{ room.description }}</small>
					</span>
					<span v-if="room.unreadCount" class="communication-inbox-item__unread">{{ room.unreadCount }}</span>
				</button>
			</div>
		</aside>

		<section class="communication-channel-stream" aria-label="Channel stream">
			<header class="communication-workspace-panel__header">
				<div class="communication-workspace-panel__title-row">
					<div>
						<h2 class="communication-conversation__title">#{{ workspace.activeRoomLabel }}</h2>
						<p class="communication-conversation__subtitle">Persistent channel history with threads and evidence.</p>
					</div>
					<Badge variant="accent">Workspace</Badge>
				</div>
			</header>
			<div class="communication-channel-stream__messages">
				<MessageBubble
					v-for="message in workspace.messages"
					:key="message.id"
					:author="message.author"
					:direction="message.direction"
					:meta="message.meta"
					:timestamp="message.timestamp"
					:tone="message.tone"
				>
					<p>{{ message.body }}</p>
				</MessageBubble>
			</div>
		</section>

		<aside class="communication-channel-inspector" aria-label="Channel inspector">
			<header class="communication-workspace-panel__header">
				<div class="communication-workspace-panel__title-row">
					<h2 class="communication-workspace-panel__title">Context</h2>
					<Icon icon="tabler:threads" size="1rem" />
				</div>
				<p class="communication-workspace-panel__meta">Pinned entities and thread candidates for the channel.</p>
			</header>
			<div class="communication-inspector">
				<section
					v-for="section in workspace.inspectorSections"
					:key="section.id"
					class="communication-inspector-section"
				>
					<h3 class="communication-inspector-section__title">{{ section.title }}</h3>
					<article
						v-for="item in section.items"
						:key="item.id"
						class="communication-inspector-entity"
					>
						<div class="communication-inspector-entity__identity">
							<EntityIcon :entity="item.entity" :label="item.title" />
							<div class="communication-inspector-entity__body">
								<h4 class="communication-inspector-entity__title">{{ item.title }}</h4>
								<p class="communication-inspector-entity__description">{{ item.description }}</p>
								<p class="communication-inspector-entity__evidence">{{ item.evidenceLabel }}</p>
							</div>
						</div>
					</article>
				</section>
			</div>
		</aside>
	</section>
</template>
