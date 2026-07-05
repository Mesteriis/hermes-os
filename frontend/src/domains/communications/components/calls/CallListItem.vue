<script setup lang="ts">
import { Badge, Icon } from '@/shared/ui'
import '../communicationDomainElements.css'
import {
  communicationCallKindIconName,
  communicationCallProviderIconName,
  communicationCallStateLabel,
  communicationCallStateTone,
  type CommunicationCallItemModel
} from '../communicationDomainElements'

defineProps<{
  item: CommunicationCallItemModel
}>()

const emit = defineEmits<{
  select: [item: CommunicationCallItemModel]
}>()
</script>

<template>
	<button
		type="button"
		:class="['communication-call-item', item.selected && 'communication-call-item--selected']"
		:aria-label="`${item.providerLabel}, ${item.title}, ${item.startedAtLabel}`"
		@click="emit('select', item)"
	>
		<span class="communication-call-item__lead">
			<span class="communication-call-item__avatar">{{ item.avatarLabel }}</span>
			<span class="communication-call-item__provider" :aria-label="item.providerLabel">
				<Icon :icon="communicationCallProviderIconName(item.providerKind)" size="0.75rem" />
			</span>
		</span>
		<span class="communication-call-item__body">
			<span class="communication-call-item__topline">
				<strong>{{ item.title }}</strong>
				<span>{{ item.startedAtLabel }}</span>
			</span>
			<span class="communication-call-item__subtitle">{{ item.subtitle }}</span>
			<span class="communication-call-item__summary">{{ item.summary }}</span>
			<span class="communication-call-item__meta">
				<Badge :variant="communicationCallStateTone(item.state)">{{ communicationCallStateLabel(item.state) }}</Badge>
				<span>{{ item.providerLabel }}</span>
				<span>{{ item.participantsLabel }}</span>
				<span>
					<Icon :icon="communicationCallKindIconName(item.kind)" size="0.85rem" />
					{{ item.durationLabel }}
				</span>
				<span v-if="item.recurrenceLabel">{{ item.recurrenceLabel }}</span>
				<span v-if="item.transcriptStateLabel">{{ item.transcriptStateLabel }}</span>
				<span v-if="item.recordingCount">
					<Icon icon="tabler:record-mail" size="0.85rem" />
					{{ item.recordingCount }}
				</span>
			</span>
		</span>
		<span v-if="item.unreadCount" class="communication-inbox-item__unread">{{ item.unreadCount }}</span>
	</button>
</template>
