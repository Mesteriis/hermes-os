<script setup lang="ts">
import { Badge, Icon, ProviderIcon } from '@/shared/ui'
import type { CommunicationInboxItemModel } from './communicationDomainElements'
import { communicationInboxItemPresentation } from './communicationDomainElements'
import './communicationDomainElements.css'

defineProps<{
  items: readonly CommunicationInboxItemModel[]
}>()
</script>

<template>
	<section class="communication-workspace-panel communication-workspace-panel--inbox" aria-label="Communication list">
		<header class="communication-workspace-panel__header">
			<div class="communication-workspace-panel__title-row">
				<h2 class="communication-workspace-panel__title">Inbox</h2>
				<Badge variant="accent">All channels</Badge>
			</div>
			<p class="communication-workspace-panel__meta">Chats, groups and mail threads in one Communications surface.</p>
		</header>
		<div class="communication-workspace-panel__body">
			<div class="communication-inbox-list">
				<button
					v-for="item in items"
					:key="item.id"
					type="button"
					:class="[
						'communication-inbox-item',
						item.selected && 'communication-inbox-item--selected',
						communicationInboxItemPresentation(item).signal && 'communication-inbox-item--signal'
					]"
				>
					<div class="communication-inbox-item__top">
						<div class="communication-inbox-item__identity">
							<ProviderIcon
								:provider="communicationInboxItemPresentation(item).channelIcon"
								:label="communicationInboxItemPresentation(item).channelLabel"
							/>
							<div class="communication-inbox-item__title-block">
								<h3 class="communication-inbox-item__title">{{ item.title }}</h3>
								<p class="communication-inbox-item__meta">{{ communicationInboxItemPresentation(item).kindLabel }}</p>
							</div>
						</div>
						<span class="communication-inbox-item__meta">{{ item.timestamp }}</span>
					</div>
					<p class="communication-inbox-item__subtitle">{{ item.subtitle }}</p>
					<p class="communication-inbox-item__preview">{{ item.preview }}</p>
					<div class="communication-inbox-item__badges">
						<Badge :variant="communicationInboxItemPresentation(item).status.badgeTone">
							{{ communicationInboxItemPresentation(item).status.label }}
						</Badge>
						<Badge v-if="item.hasAttachments" variant="neutral">Files</Badge>
						<Badge v-if="item.muted" variant="neutral">Muted</Badge>
						<span v-if="item.unreadCount" class="communication-inbox-item__unread">{{ item.unreadCount }}</span>
						<Icon v-if="item.hasOpenAction" icon="tabler:alert-triangle" size="1rem" />
					</div>
				</button>
			</div>
		</div>
	</section>
</template>
