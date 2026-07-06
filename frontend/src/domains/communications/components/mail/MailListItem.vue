<script setup lang="ts">
import { Badge, Icon, ProviderIcon, Tooltip } from '@/shared/ui'
import '../communicationDomainElements.css'
import type { MailListItemDensity, MailListItemModel } from './mailElements'
import {
  mailListItemAriaLabel,
  mailListItemAttachmentLabel,
  mailListItemHasSignal,
  mailListItemMarkerClass,
  mailListItemMarkerPresentation,
  mailListItemMarkers,
  mailListItemSourceKind,
  mailListItemStatus,
  mailListItemStatusClass
} from './mailElements'

const props = withDefaults(defineProps<{
  item: MailListItemModel
  density?: MailListItemDensity
}>(), {
  density: 'comfortable'
})

const emit = defineEmits<{
  select: [item: MailListItemModel]
}>()

function handleSelect(): void {
  emit('select', props.item)
}

function statusChipClass(item: MailListItemModel): string {
  if (item.workflowState !== 'new') return 'mail-list-item__status-chip'
  return 'mail-list-item__status-chip mail-list-item__status-chip--visible'
}
</script>

<template>
	<button
		type="button"
		:aria-label="mailListItemAriaLabel(item)"
		:class="[
			'mail-list-item',
			`mail-list-item--${density}`,
			item.selected && 'mail-list-item--selected',
			mailListItemHasSignal(item) && 'mail-list-item--signal'
		]"
		@click="handleSelect"
	>
		<div class="mail-list-item__main">
			<div class="mail-list-item__primary">
				<div class="mail-list-item__header">
					<div class="mail-list-item__sender-line">
						<ProviderIcon
							class="mail-list-item__source"
							:provider="mailListItemSourceKind(item)"
							:label="item.accountLabel"
							size="1rem"
						/>
						<div class="mail-list-item__sender-body">
							<strong class="mail-list-item__sender-name">{{ item.fromName }}</strong>
						</div>
					</div>
					<div class="mail-list-item__meta">
						<span v-if="item.unreadCount" class="mail-list-item__unread">{{ item.unreadCount }}</span>
						<span class="mail-list-item__time">{{ item.timestampLabel }}</span>
					</div>
				</div>
				<h3 class="mail-list-item__subject">{{ item.subject }}</h3>
				<div class="mail-list-item__summary">
					<p class="mail-list-item__snippet">{{ item.snippet }}</p>
					<div class="mail-list-item__signals" aria-label="List item signals">
						<span :class="['mail-list-item__status-dot', mailListItemStatusClass(item)]" aria-hidden="true" />
						<Badge
							:class="statusChipClass(item)"
							:variant="mailListItemStatus(item).badgeTone"
						>
							{{ mailListItemStatus(item).label }}
						</Badge>
						<span v-if="item.attachmentCount" class="mail-list-item__attachment">
							<Icon icon="tabler:paperclip" size="0.9rem" />
							<span>{{ item.attachmentCount }}</span>
							<span class="mail-list-item__attachment-label">{{ mailListItemAttachmentLabel(item) }}</span>
						</span>
						<div v-if="mailListItemMarkers(item).length" class="mail-list-item__marker-group" aria-label="Mail markers">
							<Tooltip
								v-for="marker in mailListItemMarkers(item)"
								:key="marker"
								:content="mailListItemMarkerPresentation(marker).label"
							>
								<template #trigger>
									<span
										:class="['mail-list-item__marker', mailListItemMarkerClass(marker)]"
										role="img"
										:aria-label="mailListItemMarkerPresentation(marker).label"
									>
										<Icon :icon="mailListItemMarkerPresentation(marker).icon" size="0.9rem" />
									</span>
								</template>
							</Tooltip>
						</div>
						<Icon v-if="item.hasOpenAction" class="mail-list-item__action-icon" icon="tabler:alert-triangle" size="0.95rem" />
					</div>
				</div>
			</div>
		</div>
	</button>
</template>
